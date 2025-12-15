use dma_rs::Dma;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dma = Dma::new("fpga://algo=0")?;

    let process = dma.attach("explorer.exe")?;
    let info = process.info()?;

    println!("Attached to {} (PID: {})", info.name, info.pid);
    println!("Path: {}\n", info.path);

    let modules = process.list_modules()?;
    println!("Loaded modules: {}", modules.len());

    for module in modules.iter().take(5) {
        println!("  {} @ 0x{:X} (size: 0x{:X})",
            module.name, module.base, module.size);
    }

    if let Some(kernel32) = modules.iter().find(|m| m.name.eq_ignore_ascii_case("kernel32.dll")) {
        println!("\nkernel32.dll base: 0x{:X}", kernel32.base);

        let bytes = process.read_bytes(kernel32.base, 64)?;
        println!("First 64 bytes:");
        for (i, chunk) in bytes.chunks(16).enumerate() {
            print!("  {:04X}: ", i * 16);
            for byte in chunk {
                print!("{:02X} ", byte);
            }
            println!();
        }
    }

    Ok(())
}
