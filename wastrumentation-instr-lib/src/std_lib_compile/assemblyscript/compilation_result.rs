use super::super::{CompilerResult as CompilerResultTrait, WasmModule};

pub type CompilationResult = Result<WasmModule, CompilationError>;

#[derive(Debug)]
pub struct CompilationError(pub String);

impl CompilerResultTrait for CompilationResult {
    fn module(&self) -> Result<WasmModule, String> {
        match self {
            Ok(module) => Ok(module.clone()),
            Err(CompilationError(reason)) => Err(reason.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug() {
        assert_eq!(
            format!("{:?}", CompilationError("reason".into())),
            r#"CompilationError("reason")"#
        );
    }
}
