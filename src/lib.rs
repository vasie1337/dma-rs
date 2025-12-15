use memprocfs::Vmm;

pub mod error;

// C:\\Users\\vasie\\Desktop\\temp\\dump.dmp

pub fn init() -> Result<(), error::DmaError> {
    let args = [
        "-printf",
        "-v",
        "-waitinitialize",
        "-device",
        "C:\\Users\\vasie\\Desktop\\temp\\dump.dmp",
    ]
    .to_vec();
    if let Ok(vmm) = Vmm::new("C:\\Users\\vasie\\Desktop\\temp\\vmm.dll", &args) {
        println!("VMM initialized");
        let processes = match vmm.process_list() {
            Ok(processes) => processes,
            Err(e) => return Err(error::DmaError::InitFailed(e.to_string())),
        };
        for process in &*processes {
            println!("{process}");
        }
        Ok(())
    } else {
        Err(error::DmaError::InitFailed(
            "Failed to initialize VMM".to_string(),
        ))
    }
}
