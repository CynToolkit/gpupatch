use std::fs;
use std::path::Path;
use std::error::Error;
use std::collections::HashMap;
// use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Architecture {
    PE32,
    PE32Plus,
}

#[derive(Debug)]
struct Export {
    ordinal: u32,
    name: Option<String>,
    rva: u32,
}

fn read_u16(data: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(data[offset..offset+2].try_into().unwrap())
}

fn read_u32(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(data[offset..offset+4].try_into().unwrap())
}

fn read_u64(data: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(data[offset..offset+8].try_into().unwrap())
}

fn write_u16(data: &mut [u8], offset: usize, val: u16) {
    data[offset..offset+2].copy_from_slice(&val.to_le_bytes());
}

fn write_u32(data: &mut [u8], offset: usize, val: u32) {
    data[offset..offset+4].copy_from_slice(&val.to_le_bytes());
}

fn write_u64(data: &mut [u8], offset: usize, val: u64) {
    data[offset..offset+8].copy_from_slice(&val.to_le_bytes());
}

fn align_to(val: u32, alignment: u32) -> u32 {
    if alignment == 0 { return val; }
    (val + alignment - 1) / alignment * alignment
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: nvpatch-rs <inputfile> [<outputfile>] [--disable]");
        return Ok(());
    }

    let input_path = &args[1];
    let mut output_path = input_path;
    let mut disable = false;
    
    if args.contains(&"--disable".to_string()) {
        disable = true;
    }
    
    for arg in args.iter().skip(2) {
        if !arg.starts_with("--") {
            output_path = arg;
            break;
        }
    }

    let bytes = fs::read(input_path)?;
    let patched = patch_pe(&bytes, disable, Path::new(input_path).file_name().and_then(|n| n.to_str()).unwrap_or("output.exe"))?;
    
    fs::write(output_path, patched)?;
    println!("Successfully patched {}", output_path);
    
    Ok(())
}

fn patch_pe(input: &[u8], disable: bool, filename: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut data = input.to_vec();
    let pe_ptr = read_u32(&data, 0x3C) as usize;
    if &data[pe_ptr..pe_ptr+4] != b"PE\0\0" {
        return Err("Not a valid PE file".into());
    }

    let coff_hdr = pe_ptr + 4;
    let num_sections = read_u16(&data, coff_hdr + 2);
    let optional_hdr_size = read_u16(&data, coff_hdr + 16) as usize;
    
    let optional_hdr = coff_hdr + 20;
    let magic = read_u16(&data, optional_hdr);
    
    let arch = match magic {
        0x10B => Architecture::PE32,
        0x20B => Architecture::PE32Plus,
        _ => return Err("Unknown PE magic".into()),
    };

    let (data_dirs_offset, size_of_image_offset, file_align, sect_align) = match arch {
        Architecture::PE32 => (
            optional_hdr + 96,
            optional_hdr + 28 + 28,
            read_u32(&data, optional_hdr + 28 + 8),
            read_u32(&data, optional_hdr + 28 + 4)
        ),
        Architecture::PE32Plus => (
            optional_hdr + 112,
            optional_hdr + 24 + 32,
            read_u32(&data, optional_hdr + 24 + 12),
            read_u32(&data, optional_hdr + 24 + 8)
        ),
    };

    let sections_offset = optional_hdr + optional_hdr_size;
    
    // Parse sections
    #[derive(Clone, Debug)]
    struct Section {
        _name: [u8; 8],
        v_size: u32,
        v_addr: u32,
        raw_size: u32,
        raw_ptr: u32,
        _chars: u32,
    }
    let mut sections = Vec::new();
    for i in 0..num_sections as usize {
        let off = sections_offset + i * 40;
        sections.push(Section {
            _name: data[off..off+8].try_into().unwrap(),
            v_size: read_u32(&data, off+8),
            v_addr: read_u32(&data, off+12),
            raw_size: read_u32(&data, off+16),
            raw_ptr: read_u32(&data, off+20),
            _chars: read_u32(&data, off+36),
        });
    }

    let rva_to_offset = |rva: u32| -> Option<usize> {
        for s in &sections {
            if rva >= s.v_addr && rva < s.v_addr + s.v_size {
                return Some((s.raw_ptr + (rva - s.v_addr)) as usize);
            }
        }
        None
    };

    // Check existing Export Directory Table
    let export_rva = read_u32(&data, data_dirs_offset);
    let export_size = read_u32(&data, data_dirs_offset + 4);
    
    let symbols = ["NvOptimusEnablement", "AmdPowerXpressRequestHighPerformance"];
    let mut exports: Vec<Export> = Vec::new();
    let mut ordinal_base = 1;
    let mut module_name = filename.to_string();

    if export_size > 0 {
        if let Some(export_tbl_off) = rva_to_offset(export_rva) {
            let _flags = read_u32(&data, export_tbl_off);
            let name_rva = read_u32(&data, export_tbl_off + 12);
            ordinal_base = read_u32(&data, export_tbl_off + 16);
            let addr_table_entries = read_u32(&data, export_tbl_off + 20);
            let num_name_pointers = read_u32(&data, export_tbl_off + 24);
            let eat_rva = read_u32(&data, export_tbl_off + 28);
            let npt_rva = read_u32(&data, export_tbl_off + 32);
            let ot_rva = read_u32(&data, export_tbl_off + 36);

            if let Some(n_off) = rva_to_offset(name_rva) {
                if let Some(end) = data[n_off..].iter().position(|&c| c == 0) {
                    module_name = String::from_utf8_lossy(&data[n_off..n_off+end]).to_string();
                }
            }

            // Read address table
            if let Some(eat_off) = rva_to_offset(eat_rva) {
                for i in 0..addr_table_entries {
                    let rva = read_u32(&data, eat_off + (i * 4) as usize);
                    exports.push(Export {
                        ordinal: ordinal_base + i,
                        name: None,
                        rva,
                    });
                }
            }

            // Read names and map to ordinals
            if let (Some(npt_off), Some(ot_off)) = (rva_to_offset(npt_rva), rva_to_offset(ot_rva)) {
                for i in 0..num_name_pointers {
                    let name_ptr_rva = read_u32(&data, npt_off + (i * 4) as usize);
                    let ord_idx = read_u16(&data, ot_off + (i * 2) as usize);
                    if let Some(n_off) = rva_to_offset(name_ptr_rva) {
                        if let Some(end) = data[n_off..].iter().position(|&c| c == 0) {
                            let name = String::from_utf8_lossy(&data[n_off..n_off+end]).to_string();
                            let final_ord = ordinal_base + ord_idx as u32;
                            if let Some(e) = exports.iter_mut().find(|e| e.ordinal == final_ord) {
                                e.name = Some(name);
                            }
                        }
                    }
                }
            }
        }
    }

    // Check if all desired symbols exist
    let mut existing_match = true;
    for s in symbols {
        if !exports.iter().any(|e| e.name.as_deref() == Some(s)) {
            existing_match = false;
            break;
        }
    }

    if existing_match && exports.len() > 0 {
        // Just patch existing values!
        for s in symbols {
            let entry = exports.iter().find(|e| e.name.as_deref() == Some(s)).unwrap();
            if let Some(off) = rva_to_offset(entry.rva) {
                let val: u32 = if disable { 0 } else { 1 };
                write_u32(&mut data, off, val);
            }
        }
        // Clear checksum
        let chk_off = match arch {
            Architecture::PE32 => optional_hdr + 28 + 36,
            Architecture::PE32Plus => optional_hdr + 24 + 40,
        };
        write_u32(&mut data, chk_off, 0);
        return Ok(data);
    }

    // Add to export list if not present
    let max_ord = exports.iter().map(|e| e.ordinal).max().unwrap_or(0);
    let mut next_ord = max_ord + 1;
    
    // We will construct new section
    let mut new_sect_buf = Vec::new();
    
    // 1. Write symbol values themselves (two 32-bit ints)
    let _base_val_rva_offset = 0; // relative to start of section
    let target_val: u32 = if disable { 0 } else { 1 };
    new_sect_buf.extend_from_slice(&target_val.to_le_bytes());
    new_sect_buf.extend_from_slice(&target_val.to_le_bytes());
    
    // Define where they go
    let mut _sym_rvas: Vec<u32> = Vec::new();
    
    // Create new export objects
    let last_s = sections.last().unwrap();
    let new_v_addr = align_to(last_s.v_addr + last_s.v_size, sect_align);
    
    for (i, s) in symbols.iter().enumerate() {
        if !exports.iter().any(|e| e.name.as_deref() == Some(*s)) {
            let rva = new_v_addr + (i as u32 * 4);
            exports.push(Export {
                ordinal: next_ord,
                name: Some(s.to_string()),
                rva,
            });
            next_ord += 1;
        }
    }

    // Re-sort by ordinal to match layout
    exports.sort_by_key(|e| e.ordinal);
    
    let export_dir_tbl_pos = new_sect_buf.len();
    // Placeholder for ExportDirectoryTable (40 bytes)
    new_sect_buf.resize(new_sect_buf.len() + 40, 0);
    
    // 2. Write Address Table
    let eat_pos = new_sect_buf.len();
    let min_ord = exports.iter().map(|e| e.ordinal).min().unwrap_or(1);
    let count_ord = exports.iter().map(|e| e.ordinal).max().unwrap_or(0) - min_ord + 1;
    let mut eat = vec![0u32; count_ord as usize];
    for e in &exports {
        let idx = (e.ordinal - min_ord) as usize;
        eat[idx] = e.rva;
    }
    for val in &eat {
        new_sect_buf.extend_from_slice(&val.to_le_bytes());
    }
    
    // 3. Write Strings
    let mut name_rvas = HashMap::new();
    for e in &exports {
        if let Some(name) = &e.name {
            name_rvas.insert(name.clone(), new_v_addr + new_sect_buf.len() as u32);
            new_sect_buf.extend_from_slice(name.as_bytes());
            new_sect_buf.push(0);
        }
    }
    let mod_name_rva = new_v_addr + new_sect_buf.len() as u32;
    new_sect_buf.extend_from_slice(module_name.as_bytes());
    new_sect_buf.push(0);
    
    // 4. Write Name Pointer Table (sorted by name)
    let npt_pos = new_sect_buf.len();
    let mut sorted_exports: Vec<_> = exports.iter().filter(|e| e.name.is_some()).collect();
    sorted_exports.sort_by_key(|e| e.name.as_ref().unwrap());
    
    for e in &sorted_exports {
        let rva = *name_rvas.get(e.name.as_ref().unwrap()).unwrap();
        new_sect_buf.extend_from_slice(&rva.to_le_bytes());
    }
    
    // 5. Write Ordinal Table
    let ot_pos = new_sect_buf.len();
    for e in &sorted_exports {
        let idx = (e.ordinal - min_ord) as u16;
        new_sect_buf.extend_from_slice(&idx.to_le_bytes());
    }
    
    let final_size = new_sect_buf.len();
    
    // Finalize ExportDirectoryTable
    let mut edt = vec![0u8; 40];
    write_u32(&mut edt, 4, 0xFFFFFFFF); // Timestamp
    write_u32(&mut edt, 12, mod_name_rva);
    write_u32(&mut edt, 16, min_ord);
    write_u32(&mut edt, 20, count_ord);
    write_u32(&mut edt, 24, sorted_exports.len() as u32);
    write_u32(&mut edt, 28, new_v_addr + eat_pos as u32);
    write_u32(&mut edt, 32, new_v_addr + npt_pos as u32);
    write_u32(&mut edt, 36, new_v_addr + ot_pos as u32);
    
    new_sect_buf[export_dir_tbl_pos..export_dir_tbl_pos+40].copy_from_slice(&edt);
    
    // Prepare output bytes
    let last_orig = sections.iter().max_by_key(|s| s.raw_ptr).unwrap();
    let last_orig_end = last_orig.raw_ptr + last_orig.raw_size;
    let has_extra = (last_orig_end as usize) < data.len();
    
    let new_raw_ptr = align_to(last_orig_end, file_align);
    let new_raw_size = align_to(final_size as u32, file_align);
    
    // Make new section header
    let mut new_hdr = vec![0u8; 40];
    new_hdr[0..8].copy_from_slice(b".nvpatch");
    write_u32(&mut new_hdr, 8, final_size as u32);
    write_u32(&mut new_hdr, 12, new_v_addr);
    write_u32(&mut new_hdr, 16, new_raw_size);
    write_u32(&mut new_hdr, 20, new_raw_ptr);
    write_u32(&mut new_hdr, 36, 0x40000040); // Read | Initialized
    
    // 1. Inject new section header
    let ins_pos = sections_offset + (num_sections as usize * 40);
    if ins_pos + 40 > sections[0].raw_ptr as usize {
        return Err("Not enough space for a new section header".into());
    }
    
    // Apply header modifications to working copy 'data'
    write_u16(&mut data, coff_hdr + 2, num_sections + 1);
    
    // Update SizeOfImage (Note: Original C# tool directly adds raw_size without aligning to section alignment)
    let cur_size_img = read_u32(&data, size_of_image_offset);
    write_u32(&mut data, size_of_image_offset, cur_size_img + new_raw_size);
    
    // Update SizeOfInitializedData in Standard Header (offset 8 from optional_hdr)
    let cur_init_size = read_u32(&data, optional_hdr + 8);
    write_u32(&mut data, optional_hdr + 8, cur_init_size + new_raw_size);
    
    // Clear CheckSum
    let chk_off = match arch {
        Architecture::PE32 => optional_hdr + 28 + 36,
        Architecture::PE32Plus => optional_hdr + 24 + 40,
    };
    write_u32(&mut data, chk_off, 0);

    // Update Export DataDirectory
    write_u32(&mut data, data_dirs_offset, new_v_addr + export_dir_tbl_pos as u32);
    write_u32(&mut data, data_dirs_offset + 4, 40 + eat_pos as u32 - export_dir_tbl_pos as u32 + 0); // approximation or exact
    // Wait, C# explicitly computes size
    write_u32(&mut data, data_dirs_offset + 4, final_size as u32 - export_dir_tbl_pos as u32);

    // Actually write the header into data
    data[ins_pos..ins_pos+40].copy_from_slice(&new_hdr);

    // Assemble final output!
    let mut out = Vec::new();
    // The portion up to last Original Section End
    let pivot = last_orig_end as usize;
    out.extend_from_slice(&data[0..pivot]);
    
    // Pad up to new raw ptr
    while out.len() < new_raw_ptr as usize {
        out.push(0);
    }
    
    let start_of_added = out.len();
    // Write new section bytes
    out.extend_from_slice(&new_sect_buf);
    // Pad section up to aligned size
    while out.len() < (new_raw_ptr + new_raw_size) as usize {
        out.push(0);
    }
    let added_bytes = (out.len() - start_of_added) as i64;
    
    if has_extra {
        let _old_len = out.len();
        out.extend_from_slice(&data[pivot..]);
        // Handle .NET Bundle Manifest
        update_net_bundle_manifest(&mut out, pivot, added_bytes);
    }

    Ok(out)
}

fn update_net_bundle_manifest(buf: &mut Vec<u8>, _pivot: usize, offset: i64) {
    let sig: [u8; 32] = [
        0x8b, 0x12, 0x02, 0xb9, 0x6a, 0x61, 0x20, 0x38,
        0x72, 0x7b, 0x93, 0x02, 0x14, 0xd7, 0xa0, 0x32,
        0x13, 0xf5, 0xb9, 0xe6, 0xef, 0xae, 0x33, 0x18,
        0xee, 0x3b, 0x2d, 0xce, 0x24, 0xb3, 0x6a, 0xae,
    ];
    
    let mut sig_pos = None;
    for i in 0..(buf.len().saturating_sub(32)) {
        if &buf[i..i+32] == &sig {
            sig_pos = Some(i);
            break;
        }
    }
    
    if let Some(p) = sig_pos {
        let manifest_ptr_pos = p - 8;
        let manifest_pos = read_u64(buf, manifest_ptr_pos) as i64;
        if manifest_pos > 0 {
            // Shift pointers that reside AFTER the manifest pointer pos?
            // Wait, C# reads from original stream then writes back to file stream at modified position.
            
            // The manifest itself shifted by 'offset' because we inserted bytes at pivot.
            // Wait, let's be precise. All data after pivot shifted by `offset`.
            // Manifest was originally at manifest_pos. Now it is at manifest_pos + offset.
            let new_manifest_pos = manifest_pos + offset;
            write_u64(buf, manifest_ptr_pos, new_manifest_pos as u64);
            
            // Now read the contents of the manifest at new_manifest_pos and adjust offsets inside it
            let mut cur = new_manifest_pos as usize;
            
            let major = read_u32(buf, cur); cur += 4;
            let _minor = read_u32(buf, cur); cur += 4;
            let file_count = read_u32(buf, cur) as i32; cur += 4;
            
            // skip bundle id string (length prefixed)
            fn skip_str(buf: &[u8], c: &mut usize) {
                // it uses BinaryWriter.Write(string) which uses 7-bit encoded length
                let mut len = 0;
                let mut shift = 0;
                loop {
                    let b = buf[*c]; *c += 1;
                    len |= ((b & 0x7F) as usize) << shift;
                    if b & 0x80 == 0 { break; }
                    shift += 7;
                }
                *c += len;
            }
            skip_str(buf, &mut cur);
            
            if major >= 2 {
                // depsJsonOffset
                let val = read_u64(buf, cur);
                if val > 0 { write_u64(buf, cur, (val as i64 + offset) as u64); }
                cur += 8;
                cur += 8; // size

                // runtimeConfigJsonOffset
                let val = read_u64(buf, cur);
                if val > 0 { write_u64(buf, cur, (val as i64 + offset) as u64); }
                cur += 8;
                cur += 8; // size
                
                cur += 8; // flags
            }
            
            for _ in 0..file_count {
                // fileOffset
                let val = read_u64(buf, cur);
                if val > 0 { write_u64(buf, cur, (val as i64 + offset) as u64); }
                cur += 8;
                cur += 8; // fileSize
                
                if major >= 6 {
                    cur += 8; // compressedSize
                }
                cur += 1; // type
                skip_str(buf, &mut cur);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_mock_pe() -> Vec<u8> {
        // Create a minimal valid dummy PE layout to test the patcher logic.
        let mut data = vec![0u8; 1024];
        data[0] = b'M'; data[1] = b'Z';
        write_u32(&mut data, 0x3C, 128); // pe header at 128
        let pe = 128;
        data[pe] = b'P'; data[pe+1] = b'E';
        
        let coff = pe + 4;
        write_u16(&mut data, coff, 0x8664); // AMD64
        write_u16(&mut data, coff + 2, 1); // 1 section
        write_u16(&mut data, coff + 16, 240); // optional header size (big enough)
        
        let opt = coff + 20;
        write_u16(&mut data, opt, 0x20B); // PE32+
        write_u32(&mut data, opt + 24 + 8, 4096); // SectAlign
        write_u32(&mut data, opt + 24 + 12, 512); // FileAlign
        write_u32(&mut data, opt + 24 + 32, 8192); // SizeOfImage
        
        // Sections start at opt + 240 = 128 + 4 + 20 + 240 = 392
        let sect = opt + 240;
        data[sect] = b'.'; data[sect+1] = b't'; data[sect+2] = b'e'; data[sect+3] = b'x'; data[sect+4] = b't';
        write_u32(&mut data, sect+8, 100); // v_size
        write_u32(&mut data, sect+12, 4096); // v_addr
        write_u32(&mut data, sect+16, 512); // raw size
        write_u32(&mut data, sect+20, 512); // raw ptr
        
        // Make dummy file actually 1024 bytes large
        data.resize(1024, 0);
        data
    }

    #[test]
    fn test_patch_dummy() {
        let mock = create_mock_pe();
        let patched = patch_pe(&mock, false, "test.exe").expect("Patch failed");
        
        // Verify the new section exists!
        let pe_ptr = read_u32(&patched, 0x3C) as usize;
        let coff = pe_ptr + 4;
        assert_eq!(read_u16(&patched, coff + 2), 2); // Should have 2 sections now
        
        // Read Export directory rva
        let opt = coff + 20;
        let data_dirs = opt + 112;
        let ex_rva = read_u32(&patched, data_dirs);
        assert!(ex_rva > 0, "Export RVA must be set");
    }

    #[test]
    fn test_compare_binaries() {
        // This acts as a test function to compare two binary buffers or files
        let mock = create_mock_pe();
        let p1 = patch_pe(&mock, false, "app.exe").unwrap();
        let p2 = patch_pe(&mock, false, "app.exe").unwrap();
        
        assert_eq!(p1, p2, "Deterministic patching failed");
    }
}
