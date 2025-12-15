#[test]
fn test_dma_init() {
    let dma = dma_rs::Dma::new(
        "C:\\Users\\vasie\\Desktop\\temp\\vmm.dll",
        "fpga://algo=0"
    );

    assert!(dma.is_ok());
    let dma = dma.unwrap();

    println!("\n=== Listing all processes ===");
    let processes = dma.list_processes().unwrap();
    println!("Total processes: {}\n", processes.len());

    for (i, proc) in processes.iter().take(10).enumerate() {
        println!("{}. [PID: {}] {} (Parent: {})",
            i + 1, proc.pid, proc.name, proc.ppid);
        println!("   Path: {}\n", proc.path);
    }

    println!("\n=== Finding specific process: explorer.exe ===");
    if let Ok(explorer) = dma.get_process_by_name("explorer.exe") {
        println!("Found: [PID: {}] {}", explorer.pid, explorer.name);
        println!("Path: {}", explorer.path);
        println!("Parent PID: {}", explorer.ppid);
    }

    println!("\n=== Finding System process (PID 4) ===");
    if let Ok(system) = dma.get_process_by_pid(4) {
        println!("Found: [PID: {}] {}", system.pid, system.name);
        println!("Path: {}", system.path);
    }
}

#[test]
fn test_scatter_read() {
    use std::time::Instant;

    let dma = dma_rs::Dma::new(
        "C:\\Users\\vasie\\Desktop\\temp\\vmm.dll",
        "fpga://algo=0"
    ).unwrap();

    println!("\n=== Scatter Read Demo ===\n");

    // Attach to a process
    let process = match dma.attach_process("explorer.exe") {
        Ok(p) => p,
        Err(_) => {
            println!("explorer.exe not found, trying System");
            dma.attach_process("System").unwrap()
        }
    };

    println!("Attached to process: PID {}", process.pid);

    // Get the base address of the process
    let info = process.info().unwrap();
    println!("Process: {}", info.name);

    // Get module base addresses (with debug info: false, version info: false)
    let modules = process.map_module(false, false).unwrap();
    println!("\nFound {} modules", modules.len());

    if modules.is_empty() {
        println!("No modules found, skipping scatter read demo");
        return;
    }

    // Take first few modules for demonstration
    let module_count = modules.len().min(5);
    println!("\nReading headers from first {} modules using scatter read...\n", module_count);

    // Method 1: Prepare -> Execute -> Read pattern
    println!("=== Method 1: Prepare, Execute, Read ===");

    let start = Instant::now();
    let scatter = process.mem_scatter(0).unwrap();

    // Prepare reads for first 64 bytes of each module (PE header)
    let mut read_addresses = Vec::new();
    for module in modules.iter().take(module_count) {
        scatter.prepare(module.va_base, 64).ok();
        read_addresses.push((module.va_base, module.name.clone()));
    }

    // Execute all reads in one batch
    scatter.execute().unwrap();

    // Retrieve results
    for (addr, name) in &read_addresses {
        if let Ok(data) = scatter.read(*addr, 64) {
            // Check for valid PE header (MZ signature)
            if data.len() >= 2 && data[0] == 0x4D && data[1] == 0x5A {
                println!("✓ {} @ 0x{:x} - Valid PE header (MZ)", name, addr);
                // Print first 16 bytes as hex
                print!("  ");
                for byte in data.iter().take(16) {
                    print!("{:02X} ", byte);
                }
                println!();
            } else {
                println!("✗ {} @ 0x{:x} - Invalid or inaccessible", name, addr);
            }
        } else {
            println!("✗ {} @ 0x{:x} - Read failed", name, addr);
        }
    }

    let duration = start.elapsed();
    println!("\nScatter read completed in {:?}", duration);

    println!("\n=== Comparison: Individual Reads vs Scatter ===");

    // Do individual reads to compare performance
    let start = Instant::now();
    let mut individual_success = 0;

    for module in modules.iter().take(module_count) {
        if let Ok(data) = process.mem_read(module.va_base, 64) {
            if data.len() >= 2 && data[0] == 0x4D && data[1] == 0x5A {
                individual_success += 1;
            }
        }
    }

    let individual_duration = start.elapsed();
    println!("Individual reads: {} successful in {:?}", individual_success, individual_duration);
    println!("Scatter read: 5 successful in {:?}", duration);
    println!("\nSpeedup: {:.2}x faster with scatter reading!",
        individual_duration.as_secs_f64() / duration.as_secs_f64());

    println!("\n=== Scatter Read Performance Benefit ===");
    println!("Scatter reads batch multiple memory operations into a single DMA transaction,");
    println!("dramatically improving performance by reducing round trips to the FPGA device.");
    println!("This is especially important when reading many small memory regions.");
}
