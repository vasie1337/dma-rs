#[test]
fn test_dma_init() {
    let dma = dma_rs::Dma::new("fpga://algo=0");

    assert!(dma.is_ok());
    let dma = dma.unwrap();

    let processes = dma.list_processes().unwrap();
    println!("Total processes: {}\n", processes.len());
}
