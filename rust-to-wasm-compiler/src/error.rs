use cargo::CliError;
use std::io::Error as ErrorIO;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
/// Error kinds of what can go wrong when the compiler is initialized
#[error("Create custom abort file failed")]
pub struct CompilerSetupError(pub CliError);
impl<IntoCliError: Into<CliError>> From<IntoCliError> for CompilerSetupError {
    fn from(error: IntoCliError) -> Self {
        CompilerSetupError(error.into())
    }
}

#[derive(Debug, Error)]
/// Error kinds of what can go wrong when a compilation is invoked
pub enum CompilationError {
    #[error("Could not create temp working dir: {0}")]
    CreateTempWorkingDir(ErrorIO),
    #[error("Could not create temp cargo manifestation: {0}")]
    CreateTempCargoManifest(ErrorIO),
    #[error("Could not write to temp cargo manifestation: {0}")]
    WriteTempCargoManifest(ErrorIO),
    #[error("Could not create temp source directory: {0}")]
    CreateTempSourceDirectory(ErrorIO),
    #[error("Could not create temp library file: {0}")]
    CreateTempLibraryFile(ErrorIO),
    #[error("Could not write to temp library file: {0}")]
    WriteTempLibraryFile(ErrorIO),
    #[error("Cache files have been tampered with in {0}")]
    CacheFilesTamperedWith(PathBuf),
    #[error("Could not create workspace: {0:?}")]
    WorkspaceCreationFailed(CliError),
    #[error("Could not create compile options: {0:?}")]
    CompileOptionsCreationFailed(CliError),
    #[error("Could not select target architecture: {0:?}")]
    TargetCreationFailed(CliError),
    #[error("Compilation failed: {0:?}")]
    CompilationFailed(CliError),
    #[error("Expected single compilation result after cargo compilation")]
    ExpectedSingleCompilationResult,
    #[error("Expected exactly single compilation result after cargo compilation")]
    SingleCompilationResultNotPresent,
    #[error("Expected compilation unit output path: {0}")]
    CompilationResultOutputNotAccessible(ErrorIO),
}
