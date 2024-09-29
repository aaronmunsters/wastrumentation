use thiserror::Error;
use wasm_merge::Error as MergeError;

use crate::compiler::CompilationError;

#[derive(Debug, Error)]
pub enum Error<AnalysisLangauge, InstrumentationLanguage> {
    #[error("Compilation for analysis failed: {0}")]
    CompilationErrorAnalysis(CompilationError<AnalysisLangauge>),
    #[error("Compilation for instrumentation failed: {0}")]
    CompilationErrorInstrumentation(CompilationError<InstrumentationLanguage>),
    #[error("Merging modules failed: {0}")]
    MergeError(MergeError),
}
