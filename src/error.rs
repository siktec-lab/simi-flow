use crate::prelude::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SimiError {
    #[error("Algorithm error: {0}")]
    AlgorithmError(String),

    #[error("Preprocessing error: {0}")]
    PreprocessError(String),

    #[error("Router configuration error: {0}")]
    RouterError(String),

    #[error("Batch processing error: {0}")]
    BatchError(String),
}
