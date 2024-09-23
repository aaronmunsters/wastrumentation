#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum LowToHighError {
    #[error("failed type inference {type_error}")]
    TypeInference {
        type_error: wasabi_wasm::types::TypeError,
    },
    #[error("expected low level body to terminate in `End` bytecode")]
    BodyNonEndTermination,
    #[error("expected an `If` to precede an `Else` bytecode")]
    IfDidNotPrecedeElse,
    #[error("too many `End`s")]
    ExcessiveEnd,
    #[error("no parent for `End` bytecode")]
    EndWithoutParent,
    #[error("attempt to perform 'trivial' cast from low level to high level")]
    TrivialCastAttempt,
}
