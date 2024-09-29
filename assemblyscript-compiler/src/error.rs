use std::io::Error as ErrorIO;
use thiserror::Error;

#[derive(Debug, Error)]
/// Error kinds of what can go wrong when the compiler is initialized
pub enum CompilerSetupError {
    #[error("Create custom abort file failed: {0}")]
    CustomAbortFileCreation(ErrorIO),
    #[error("Write custom abort file failed: {0}")]
    CustomAbortFileWrite(ErrorIO),
    #[error("Npm init failed: {0}")]
    NpmInitFailed(ErrorIO),
    #[error("Npm install failed: {0}")]
    NpmInstallFailed(ErrorIO),
}

#[derive(Debug, Error)]
/// Error kinds of what can go wrong when a compilation is invoked
pub enum CompilationError {
    #[error("Could not create temp input file: {0}")]
    CreateTempInputFile(ErrorIO),
    #[error("Could not write source code to temp input file: {0}")]
    WriteSourceCodeToTempInputFile(ErrorIO),
    #[error("Could not flush source code to temp input file: {0}")]
    FlushSourceCodeToTempInputFile(ErrorIO),
    #[error("Could not create temp output file: {0}")]
    CreateTempOutputFile(ErrorIO),
    #[error("Could not execute compilation command: {0}")]
    ExecuteCompilationCommand(ErrorIO),
    #[error("AssemblyScript compilation failed: {0}")]
    AssemblyScriptCompilationFailed(String),
    #[error("Could not read result from compiled output: {0}")]
    ReadResultFromCompiledOutput(ErrorIO),
}
