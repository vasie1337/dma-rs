use thiserror::Error;

#[derive(Error, Debug)]
pub enum DmaError {
    #[error("Initialization failed: {0}")]
    InitFailed(String),
}
