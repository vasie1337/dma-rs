use dma_rs::Dma;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dma = Dma::new("fpga://algo=0")?;
    let process = dma.attach("explorer.exe")?;

    let modules = process.list_modules()?;
    let module = modules.first()
        .ok_or("No modules found")?;

    println!("Reading from {} @ 0x{:X}", module.name, module.base);

    let value_u32: u32 = process.read(module.base)?;
    println!("u32 value: 0x{:08X}", value_u32);

    let value_u64: u64 = process.read(module.base)?;
    println!("u64 value: 0x{:016X}", value_u64);

    let bytes = process.read_bytes(module.base, 16)?;
    print!("Bytes: ");
    for byte in &bytes {
        print!("{:02X} ", byte);
    }
    println!();

    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    struct DosHeader {
        e_magic: u16,
        e_cblp: u16,
        e_cp: u16,
        e_crlc: u16,
    }

    let dos_header: DosHeader = process.read(module.base)?;
    println!("\nDOS Header:");
    println!("  Magic: 0x{:04X} ({}{})",
        dos_header.e_magic,
        (dos_header.e_magic as u8) as char,
        (dos_header.e_magic >> 8) as u8 as char
    );

    Ok(())
}
