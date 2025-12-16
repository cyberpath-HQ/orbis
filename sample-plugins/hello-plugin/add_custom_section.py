#!/usr/bin/env python3
"""Add a custom section to a WASM file."""
import sys
import struct

def encode_leb128(value):
    """Encode an unsigned integer as LEB128."""
    result = []
    while True:
        byte = value & 0x7F
        value >>= 7
        if value != 0:
            byte |= 0x80
        result.append(byte)
        if value == 0:
            break
    return bytes(result)

def add_custom_section(wasm_bytes, name, data):
    """Add a custom section to WASM binary."""
    # WASM magic and version
    magic = wasm_bytes[:4]
    version = wasm_bytes[4:8]
    
    if magic != b'\x00asm':
        raise ValueError("Invalid WASM file")
    
    # Build custom section
    name_bytes = name.encode('utf-8')
    section_content = encode_leb128(len(name_bytes)) + name_bytes + data
    
    # Section ID 0 = custom section
    custom_section = bytes([0]) + encode_leb128(len(section_content)) + section_content
    
    # Insert after version (at position 8)
    return magic + version + custom_section + wasm_bytes[8:]

if __name__ == '__main__':
    if len(sys.argv) != 6 or sys.argv[2] != '-s' or sys.argv[4] != '-o':
        print("Usage: add_custom_section.py <input.wasm> -s <section_name> -o <output.wasm>")
        sys.exit(1)
    
    input_file = sys.argv[1]
    section_name = sys.argv[3]
    output_file = sys.argv[5]
    
    # Read manifest from stdin
    manifest_data = sys.stdin.read().encode('utf-8')
    
    # Read WASM file
    with open(input_file, 'rb') as f:
        wasm_bytes = f.read()
    
    # Add custom section
    result = add_custom_section(wasm_bytes, section_name, manifest_data)
    
    # Write output
    with open(output_file, 'wb') as f:
        f.write(result)
    
    print(f"Added '{section_name}' custom section to {output_file}")
