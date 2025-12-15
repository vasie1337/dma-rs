use thiserror::Error;

#[derive(Error, Debug)]
pub enum DmaError {
    #[error("Initialization failed: {0}")]
    InitFailed(String),

    #[error("Process error: {0}")]
    ProcessError(String),

    #[error("Memory error: {0}")]
    MemoryError(String),

    #[error("Module error: {0}")]
    ModuleError(String),
}
