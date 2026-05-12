use std::fs;
use std::path::Path;
use std::error::Error;
use gpupatch_core::patch_pe;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let mut is_interactive = false;
    
    let input_path = if args.len() < 2 {
        is_interactive = true;
        println!("╔══════════════════════════════════════════════╗");
        println!("║            GPU Performance Patcher           ║");
        println!("╚══════════════════════════════════════════════╝");
        println!("\nInstructions:");
        println!("▶ Drag & Drop a .exe file directly into this window.");
        println!("▶ Or type the path to the executable manually.\n");
        print!("Target File: ");
        use std::io::Write;
        std::io::stdout().flush()?;
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let trimmed = input.trim().trim_matches('\"').trim_matches('\'').to_string();
        if trimmed.is_empty() {
            println!("❌ Aborted: No input provided.");
            return Ok(());
        }
        trimmed
    } else {
        args[1].clone()
    };

    let mut output_path = input_path.clone();
    let mut disable = false;
    
    if args.contains(&"--disable".to_string()) {
        disable = true;
    }
    
    for arg in args.iter().skip(2) {
        if !arg.starts_with("--") {
            output_path = arg.clone();
            break;
        }
    }

    let bytes = match fs::read(&input_path) {
        Ok(b) => b,
        Err(e) => {
            println!("❌ Error reading file '{}': {}", input_path, e);
            if is_interactive {
                println!("\nPress Enter to exit...");
                let mut _b = String::new();
                std::io::stdin().read_line(&mut _b)?;
            }
            return Err(e.into());
        }
    };
    
    let patch_result = patch_pe(
        &bytes, 
        disable, 
        Path::new(&input_path).file_name().and_then(|n| n.to_str()).unwrap_or("output.exe")
    );

    match patch_result {
        Ok(patched) => {
            fs::write(&output_path, patched)?;
            println!("\n✨ SUCCESS: Successfully patched -> {}", output_path);
            println!("🚀 The executable is now forced to high-performance GPU mode.");
        }
        Err(e) => {
            println!("\n❌ PATCH FAILED: {}", e);
        }
    }
    
    if is_interactive || args.len() == 2 {
        println!("\nPress Enter to close...");
        let mut _final_input = String::new();
        std::io::stdin().read_line(&mut _final_input)?;
    }
    
    Ok(())
}
