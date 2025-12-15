use dma_rs::{Dma, DmaError};

fn try_init_dma() -> Option<Dma<'static>> {
    Dma::new("fpga://algo=0").ok()
}

#[test]
fn test_dma_initialization() {
    match Dma::new("fpga://algo=0") {
        Ok(_) => println!("DMA initialized successfully"),
        Err(e) => println!("DMA not available (expected without hardware): {}", e),
    }
}

#[test]
fn test_list_processes() -> Result<(), DmaError> {
    let Some(dma) = try_init_dma() else {
        println!("Skipping test: DMA hardware not available");
        return Ok(());
    };

    let processes = dma.list_processes()?;

    assert!(!processes.is_empty(), "No processes found");
    println!("Found {} processes", processes.len());

    for proc in processes.iter().take(5) {
        println!("  PID: {:<6} Name: {}", proc.pid, proc.name);
    }

    Ok(())
}

#[test]
fn test_attach_process() -> Result<(), DmaError> {
    let Some(dma) = try_init_dma() else {
        println!("Skipping test: DMA hardware not available");
        return Ok(());
    };

    let process = dma.attach("explorer.exe")?;

    let info = process.info()?;
    println!("Attached to: {} (PID: {})", info.name, info.pid);

    assert_eq!(info.name, "explorer.exe");
    assert!(info.pid > 0);

    Ok(())
}

#[test]
fn test_list_modules() -> Result<(), DmaError> {
    let Some(dma) = try_init_dma() else {
        println!("Skipping test: DMA hardware not available");
        return Ok(());
    };

    let process = dma.attach("explorer.exe")?;

    let modules = process.list_modules()?;
    assert!(!modules.is_empty(), "No modules found");

    println!("Found {} modules", modules.len());
    for module in modules.iter().take(5) {
        println!("  {} @ 0x{:X}", module.name, module.base);
    }

    Ok(())
}

#[test]
fn test_memory_read() -> Result<(), DmaError> {
    let Some(dma) = try_init_dma() else {
        println!("Skipping test: DMA hardware not available");
        return Ok(());
    };

    let process = dma.attach("explorer.exe")?;

    let modules = process.list_modules()?;
    let module = modules.first()
        .ok_or_else(|| DmaError::ProcessError("No modules found".to_string()))?;

    let value: u64 = process.read(module.base)?;
    println!("Read u64 from 0x{:X}: 0x{:016X}", module.base, value);

    let bytes = process.read_bytes(module.base, 16)?;
    println!("Read {} bytes", bytes.len());
    assert_eq!(bytes.len(), 16);

    Ok(())
}

#[test]
fn test_module_operations() -> Result<(), DmaError> {
    let Some(dma) = try_init_dma() else {
        println!("Skipping test: DMA hardware not available");
        return Ok(());
    };

    let process = dma.attach("explorer.exe")?;

    let kernel32_base = process.module_base("kernel32.dll")?;
    println!("kernel32.dll base: 0x{:X}", kernel32_base);
    assert!(kernel32_base > 0);

    let create_file = process.proc_address("kernel32.dll", "CreateFileW")?;
    println!("CreateFileW: 0x{:X}", create_file);
    assert!(create_file > kernel32_base);

    Ok(())
}

#[test]
fn test_scatter_read() -> Result<(), DmaError> {
    let Some(dma) = try_init_dma() else {
        println!("Skipping test: DMA hardware not available");
        return Ok(());
    };

    let process = dma.attach("explorer.exe")?;

    let modules = process.list_modules()?;
    let module = modules.first()
        .ok_or_else(|| DmaError::ProcessError("No modules found".to_string()))?;

    let base = module.base;

    let mut scatter = process.scatter()?;

    scatter.prepare_read(base, 8);
    scatter.prepare_read(base + 0x1000, 8);
    scatter.prepare_read(base + 0x2000, 8);

    scatter.execute()?;

    let val1: u64 = scatter.read_as(base)?;
    let val2: u64 = scatter.read_as(base + 0x1000)?;
    let val3: u64 = scatter.read_as(base + 0x2000)?;

    println!("Scatter read results:");
    println!("  0x{:X}: 0x{:016X}", base, val1);
    println!("  0x{:X}: 0x{:016X}", base + 0x1000, val2);
    println!("  0x{:X}: 0x{:016X}", base + 0x2000, val3);

    Ok(())
}

#[test]
fn test_scatter_performance() -> Result<(), DmaError> {
    use std::time::Instant;

    let Some(dma) = try_init_dma() else {
        println!("Skipping test: DMA hardware not available");
        return Ok(());
    };

    let process = dma.attach("explorer.exe")?;

    let modules = process.list_modules()?;
    let module = modules.first()
        .ok_or_else(|| DmaError::ProcessError("No modules found".to_string()))?;

    let base = module.base;
    let iterations = 100;

    let start = Instant::now();
    for i in 0..iterations {
        let _: u64 = process.read(base + i * 0x1000)?;
    }
    let normal_duration = start.elapsed();

    let start = Instant::now();
    let mut scatter = process.scatter()?;

    for i in 0..iterations {
        scatter.prepare_read(base + i * 0x1000, 8);
    }

    scatter.execute()?;

    for i in 0..iterations {
        let _: u64 = scatter.read_as(base + i * 0x1000)?;
    }
    let scatter_duration = start.elapsed();

    println!("\nPerformance comparison ({} reads):", iterations);
    println!("  Normal: {:?}", normal_duration);
    println!("  Scatter: {:?}", scatter_duration);

    let speedup = normal_duration.as_secs_f64() / scatter_duration.as_secs_f64();
    println!("  Speedup: {:.2}x", speedup);

    assert!(speedup > 1.0, "Scatter should be faster than normal reads");

    Ok(())
}
