# dma-rs

A Rust library for DMA access on Windows.

## Features

- **Process Management**: List, attach, and retrieve process information
- **Memory Operations**: Read and write process memory with type-safe APIs
- **Scatter Operations**: High-performance batch memory operations
- **Module Enumeration**: List loaded modules and resolve function addresses
- **Embedded DLLs**: All dependencies are embedded at compile-time
- **Clean API**: Type-safe, ergonomic Rust interface

## Quick Start

```rust
use dma_rs::Dma;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dma = Dma::new("fpga://algo=0")?;

    let process = dma.attach("explorer.exe")?;
    let modules = process.list_modules()?;

    if let Some(module) = modules.first() {
        let value: u64 = process.read(module.base)?;
        println!("Read: 0x{:X}", value);
    }

    Ok(())
}
```

## License

MIT
