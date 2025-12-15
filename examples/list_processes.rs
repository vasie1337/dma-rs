use dma_rs::Dma;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dma = Dma::new("fpga://algo=0")?;

    let processes = dma.list_processes()?;

    println!("Found {} processes\n", processes.len());

    for proc in processes.iter().take(10) {
        println!("PID: {:<8} Name: {}", proc.pid, proc.name);
    }

    Ok(())
}
