use std::io::Error as ErrorIO;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Could not create temp input file: {0}")]
    TempInputFileCreationFailed(ErrorIO),
    #[error("Could not write to temp input file: {0}")]
    TempInputFileWriteFailed(ErrorIO),
    #[error("Could not create temp output file: {0}")]
    TempOutputFileCreationFailed(ErrorIO),
    #[error("Could not write to temp output file: {0}")]
    TempOutputFileWriteFailed(ErrorIO),
    #[error("Merge execution failed: {0}")]
    MergeExecutionFailed(ErrorIO),
    #[error("Merge execution failed with std-err: {0}")]
    MergeExecutionFailedReason(String),
    #[error("Could not read result from written output: {0}")]
    ReadFromOutputFileFailed(ErrorIO),
}
