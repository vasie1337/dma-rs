// test the dma-rs library

#[test]
fn test_dma_init() {
    let result = dma_rs::init();
    println!("{:?}", result);
    assert!(result.is_ok());
}
