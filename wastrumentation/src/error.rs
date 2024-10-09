use thiserror::Error;
use wasabi_wasm::{EncodeError, ParseError};
use wasm_merge::Error as MergeError;

use crate::{compiler::CompilationError, parse_nesting::LowToHighError};

#[derive(Debug, Error)]
pub enum Error<AnalysisLangauge, InstrumentationLanguage> {
    #[error("Compilation for analysis failed: {0}")]
    CompilationErrorAnalysis(CompilationError<AnalysisLangauge>),
    #[error("Compilation for instrumentation failed: {0}")]
    CompilationErrorInstrumentation(CompilationError<InstrumentationLanguage>),
    #[error("Merging modules failed: {0}")]
    MergeError(MergeError),
    #[error("Instrumentation failed: {0}")]
    InstrumentationError(InstrumentationError),
}

#[derive(thiserror::Error, Debug)]
pub enum InstrumentationError {
    #[error("attempt to instrument an `import` function")]
    ParseModuleError(ParseError),
    #[error("attempt to instrument inner code of an `import` function")]
    AttemptInnerInstrumentImport,
    #[error("low to high failed {low_to_high_err}")]
    LowToHighError { low_to_high_err: LowToHighError },
    #[error("Instrumentation Encode Error: {0}")]
    EncodeError(EncodeError),
}
