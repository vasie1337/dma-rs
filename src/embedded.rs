use std::path::PathBuf;
use std::sync::OnceLock;
use crate::error::DmaError;

static VMM_DLL: &[u8] = include_bytes!("../libs/vmm.dll");
static LEECHCORE_DLL: &[u8] = include_bytes!("../libs/leechcore.dll");
static FTD3XX_DLL: &[u8] = include_bytes!("../libs/FTD3XX.dll");

static EXTRACTED_VMM_PATH: OnceLock<String> = OnceLock::new();

pub fn get_vmm_dll_path() -> Result<&'static str, DmaError> {
    let path = EXTRACTED_VMM_PATH.get_or_init(|| {
        let dir = extract_dlls().expect("Failed to extract embedded DLLs");
        let vmm_path = dir.join("vmm.dll");
        vmm_path.to_string_lossy().to_string()
    });

    Ok(path.as_str())
}

fn extract_dlls() -> Result<PathBuf, DmaError> {
    let temp_dir = std::env::temp_dir().join("dma-rs-libs");

    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| DmaError::InitFailed(format!("Failed to create temp directory: {}", e)))?;

    let vmm_path = temp_dir.join("vmm.dll");
    let leechcore_path = temp_dir.join("leechcore.dll");
    let ftd3xx_path = temp_dir.join("FTD3XX.dll");

    if !vmm_path.exists() {
        std::fs::write(&vmm_path, VMM_DLL)
            .map_err(|e| DmaError::InitFailed(format!("Failed to extract vmm.dll: {}", e)))?;
    }

    if !leechcore_path.exists() {
        std::fs::write(&leechcore_path, LEECHCORE_DLL)
            .map_err(|e| DmaError::InitFailed(format!("Failed to extract leechcore.dll: {}", e)))?;
    }

    if !ftd3xx_path.exists() {
        std::fs::write(&ftd3xx_path, FTD3XX_DLL)
            .map_err(|e| DmaError::InitFailed(format!("Failed to extract FTD3XX.dll: {}", e)))?;
    }

    Ok(temp_dir)
}
