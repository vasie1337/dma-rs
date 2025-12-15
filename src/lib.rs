use memprocfs::{LeechCore, Vmm};

pub mod error;

pub fn init() -> Result<(), error::DmaError> {
    let leechcore_existing = match LeechCore::new(
        "C:\\temp\\leechcore.dll",
        "file://C:\\temp\\dump.dmp",
        LeechCore::LC_CONFIG_PRINTF_ENABLED,
    ) {
        Ok(leechcore) => leechcore,
        Err(e) => return Err(error::DmaError::InitFailed(e.to_string())),
    };

    let args = ["-printf", "-v", "-waitinitialize"].to_vec();

    let vmm = match Vmm::new_from_leechcore(&leechcore_existing, &args) {
        Ok(vmm) => vmm,
        Err(e) => return Err(error::DmaError::InitFailed(e.to_string())),
    };

    let virtualmachine_all = match vmm.map_virtual_machine() {
        Ok(virtualmachine_all) => virtualmachine_all,
        Err(e) => return Err(error::DmaError::InitFailed(e.to_string())),
    };

    for virtualmachine in &*virtualmachine_all {
        println!("{virtualmachine}");
    }

    Ok(())
}
