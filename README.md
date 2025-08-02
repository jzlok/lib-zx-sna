# lib-zx-sna

A Rust library for handling ZX Spectrum snapshot files (.sna format).

## Overview

`lib-zx-sna` provides functionality to read, parse, and manipulate ZX Spectrum snapshot files. It supports both 48K and 128K snapshot formats, allowing you to:

- Load snapshots from files or binary data
- Access CPU registers and system state
- Read memory contents through memory mapping
- Handle both 48K and 128K ZX Spectrum configurations

## Features

- **Multi-format support**: Handles both 48K and 128K ZX Spectrum snapshots
- **Memory access**: Peek operations to read memory contents with proper bank mapping
- **CPU state**: Access to all CPU registers and system state information
- **Zero-copy design**: Efficient parsing without unnecessary data copying
- **Safe memory access**: Bounds checking and proper error handling

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
lib-zx-sna = "0.1.0"
```

## Usage

### Loading a snapshot from file

```rust
use lib_zx_sna::Snapshot;

// Load a 48K snapshot
let snapshot = Snapshot::from_file("game48k.sna");

// Load a 128K snapshot
let snapshot = Snapshot::from_file("game128k.sna");
```

### Loading a snapshot from binary data

```rust
use lib_zx_sna::Snapshot;

let binary_data = std::fs::read("snapshot.sna").unwrap();
let snapshot = Snapshot::from_bin(&binary_data);
```

### Accessing CPU registers

```rust
let snapshot = Snapshot::from_file("game.sna");

// Access main registers
println!("AF: {:04X}", snapshot.header.af);
println!("BC: {:04X}", snapshot.header.bc);
println!("DE: {:04X}", snapshot.header.de);
println!("HL: {:04X}", snapshot.header.hl);

// Access alternate registers
println!("AF': {:04X}", snapshot.header.af_prime);
println!("BC': {:04X}", snapshot.header.bc_prime);

// Access index registers
println!("IX: {:04X}", snapshot.header.ix);
println!("IY: {:04X}", snapshot.header.iy);

// Access stack pointer and other registers
println!("SP: {:04X}", snapshot.header.sp);
println!("I: {:02X}", snapshot.header.i);
println!("R: {:02X}", snapshot.header.r);

// System state
println!("Interrupt mode: {}", snapshot.header.int_mode);
println!("Border color: {}", snapshot.header.border_color);
```

### Reading memory

```rust
let snapshot = Snapshot::from_file("game.sna");

// Read a byte from memory
let value = snapshot.peek(0x5000);
println!("Value at 0x5000: {:02X}", value);

// Read a word (16-bit value) from memory
let word_value = snapshot.peek_word(0x5000);
println!("Word at 0x5000: {:04X}", word_value);
```

### Handling 128K snapshots

```rust
let snapshot = Snapshot::from_file("game128k.sna");

// Check if it's a 128K snapshot
match snapshot.snapshot_type {
    lib_zx_sna::SnapshotType::Snapshot128 => {
        if let Some(ext) = &snapshot.extension {
            println!("Program Counter: {:04X}", ext.pc);
            println!("7FFD Register: {:02X}", ext.x7ffd);
            println!("TR-DOS state: {:02X}", ext.tr_dos);
        }
    }
    lib_zx_sna::SnapshotType::Snapshot48 => {
        println!("This is a 48K snapshot");
    }
}
```

peek and poke worked on the memory mapped into the writeable portion of the lower 64k of Spectrum memory.  Switch banks by writing to port 0x7ffd through the following function:
```rust
    snapshot.write_0x7ffd(bank as u8);
```

You can also peek and poke directly into the banked memory:
```rust
    let value = snapshot.bank_peek(bank, address);  // where address is in the range 0 to 0x3FFF
    snapshot.bank_poke(bank, address, value);       // writes the value into the bank at the address between 0 and 0x3FFF
```

There are also bank_peek_word and bank_poke_word

## Memory Layout

### 48K Snapshots
- Bank 0: 0x4000-0x7FFF (16K)
- Bank 1: 0x8000-0xBFFF (16K) 
- Bank 2: 0xC000-0xFFFF (16K)

### 128K Snapshots
The library handles the complex 128K memory banking automatically. Memory is organized into 8 banks of 16K each, with proper mapping based on the 7FFD register value.

## File Format

The library supports the standard ZX Spectrum .sna file format:

- **48K snapshots**: 49,179 bytes (27 byte header + 48K memory)
- **128K snapshots**: As per above + 4 byte extension + however many additional banks there are (without duplicating 2, 5 or anything mapped into 0xC000-0xFFFF)

## Examples

The repository includes example snapshot files:
- `48k.sna` - Example 48K snapshot
- `128k.sna` - Example 128K snapshot

## Testing

Run the test suite with:

```bash
cargo test
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.  See the TODO items:

## TODO

- [ ] Saving a .sna file.
- [ ] Tests for banked_peek and banked_poke

## References

- [ZX Spectrum .sna file format specification](https://worldofspectrum.org/faq/reference/formats.htm#Snapshot)
- [ZX Spectrum technical documentation](https://worldofspectrum.org/faq/reference/z80reference.htm)
- [ZX Spectrum Memory Maps](http://www.breakintoprogram.co.uk/hardware/computers/zx-spectrum/memory-map)

## License

lib-zx-sna is Copyright (c) 2025 Jez Sherlock

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

https://opensource.org/license/mit
