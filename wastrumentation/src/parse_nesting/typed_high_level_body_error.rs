use wasabi_wasm::types::TypeError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LowToHighError {
    TypeInference(TypeError),
    BodyNonEndTermination,
    IfDidNotPrecedeElse,
    ExcessiveEnd,
    EndWithoutParent,
    TrivialCastAttempt,
}

const ERROR_MSG_EXCESSIVE_END: &str = "Too many `End`s";
const ERROR_MSG_END_WTHT_PRNT: &str = "`End` has no parent body";
const ERROR_MSG_IF_NO_BFR_ELS: &str = "Expected an `If` to precede an `Else`";
const ERROR_MSG_BDY_NON_END_T: &str = "Expected low level body to terminate in `End`";
const ERROR_MSG_TRIV_CAST_ATT: &str = "Attempt to perform 'trivial' cast from Low to High";

impl std::fmt::Display for LowToHighError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LowToHighError::ExcessiveEnd => write!(f, "{ERROR_MSG_EXCESSIVE_END}"),
            LowToHighError::EndWithoutParent => write!(f, "{ERROR_MSG_END_WTHT_PRNT}"),
            LowToHighError::TrivialCastAttempt => write!(f, "{ERROR_MSG_TRIV_CAST_ATT}"),
            LowToHighError::IfDidNotPrecedeElse => write!(f, "{ERROR_MSG_IF_NO_BFR_ELS}"),
            LowToHighError::BodyNonEndTermination => write!(f, "{ERROR_MSG_BDY_NON_END_T}"),
            LowToHighError::TypeInference(e) => {
                write!(f, "Type Inference Error: {e}")
            }
        }
    }
}

impl std::error::Error for LowToHighError {}
