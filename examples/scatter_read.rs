use dma_rs::Dma;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dma = Dma::new("fpga://algo=0")?;
    let process = dma.attach("explorer.exe")?;

    let modules = process.list_modules()?;
    let kernel32 = modules.iter()
        .find(|m| m.name.eq_ignore_ascii_case("kernel32.dll"))
        .ok_or("kernel32.dll not found")?;

    let base = kernel32.base;
    let num_reads = 1000;

    let start = Instant::now();
    for i in 0..num_reads {
        let _: u32 = process.read(base + i * 0x1000)?;
    }
    let normal_time = start.elapsed();

    let start = Instant::now();
    let mut scatter = process.scatter()?;

    for i in 0..num_reads {
        scatter.prepare_read(base + i * 0x1000, 4);
    }

    scatter.execute()?;

    for i in 0..num_reads {
        let _: u32 = scatter.read_as(base + i * 0x1000)?;
    }
    let scatter_time = start.elapsed();

    println!("Normal reads ({} iterations): {:?}", num_reads, normal_time);
    println!("Scatter reads ({} iterations): {:?}", num_reads, scatter_time);
    println!("Speedup: {:.2}x", normal_time.as_secs_f64() / scatter_time.as_secs_f64());

    Ok(())
}
