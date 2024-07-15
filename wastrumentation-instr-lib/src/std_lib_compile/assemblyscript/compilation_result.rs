use super::super::WasmModule;

pub type CompilationResult = Result<WasmModule, CompilationError>;

#[derive(Debug)]
pub struct CompilationError(pub String);

impl CompilationError {
    pub fn reason(&self) -> &str {
        let Self(reason) = self;
        reason.as_str()
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
