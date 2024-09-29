use from_pest::{ConversionError, Void};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Parameters must be unique, got: {0:?}")]
    NonUniqueParameters(Vec<String>),
    #[error("Formal parameters must both be either high-level, dynamic or mutably dynamic (got: args {0}, for ress {1}).",)]
    IncorrectArgsRessType(String, String),
    #[error("Duplicate parameter accross arguments and results: {0}.")]
    DuplicateArgsRessParameter(String),
    #[error("Duplicate parameter: {0}.")]
    DuplicateParameter(String),
    #[error("Provided type {unsupported} unsupported, supported here: {supported:?}")]
    UnsupportedIdentifierType {
        unsupported: String,
        supported: Vec<String>,
    },
    #[error("Conversion error (pest) failed: {0}")]
    ConversionError(ConversionError<Void>),
    #[error("Pest error: {0}")]
    PestError(String), // The actual error would fit here too, but is too large
}
