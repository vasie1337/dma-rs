use dma_rs::Dma;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dma = Dma::new("fpga://algo=0")?;
    let process = dma.attach("explorer.exe")?;

    let kernel32_base = process.module_base("kernel32.dll")?;
    println!("kernel32.dll base: 0x{:X}", kernel32_base);

    let functions = [
        "CreateFileW",
        "ReadFile",
        "WriteFile",
        "GetProcAddress",
        "LoadLibraryW",
    ];

    for func_name in &functions {
        match process.proc_address("kernel32.dll", func_name) {
            Ok(addr) => {
                let offset = addr - kernel32_base;
                println!("{}+0x{:X} = 0x{:X}", func_name, offset, addr);
            }
            Err(e) => println!("Failed to find {}: {}", func_name, e),
        }
    }

    Ok(())
}
