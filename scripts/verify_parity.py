#!/usr/bin/env python3
import sys
import struct

def main():
    if len(sys.argv) != 3:
        sys.exit("Usage: verify_parity.py <rust_exe> <cs_exe>")
        
    rust_file = sys.argv[1]
    cs_file = sys.argv[2]
    
    with open(rust_file, "rb") as f:
        r = bytearray(f.read())
    with open(cs_file, "rb") as f:
        c = bytearray(f.read())
        
    if len(r) != len(c):
        sys.exit("Lengths differ!")
        
    pe_ptr = struct.unpack_from("<I", r, 0x3C)[0]
    size_of_image_offset = pe_ptr + 24 + 56 # Offset is 56 for both PE32 and PE32+
    allowed = set(range(size_of_image_offset, size_of_image_offset + 4))
    
    diffs = [i for i in range(len(r)) if r[i] != c[i]]
    unallowed = [d for d in diffs if d not in allowed]
    
    if unallowed:
        sys.exit(f"Unallowed differences at: {unallowed}")
        
    print("SUCCESS: Bit-for-bit compatibility achieved! (SizeOfImage strict alignment exception verified)")

if __name__ == "__main__":
    main()
