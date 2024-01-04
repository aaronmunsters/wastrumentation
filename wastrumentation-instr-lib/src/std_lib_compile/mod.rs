pub mod assemblyscript;

pub trait CompilerOptions {
    fn source_code(&self) -> Vec<u8>;

    fn compile(&self) -> Box<dyn CompilerResult>;
}

pub trait CompilerResult {
    fn module(&self) -> Result<WasmModule, String>;
}

#[derive(Debug, Clone)]
pub enum WasmModule {
    Binary(Vec<u8>),
    Text(String),
}

impl WasmModule {
    pub fn unwrap_binary(&self) -> &Vec<u8> {
        match self {
            Self::Binary(binary) => binary,
            _ => panic!("called `WasmModule::unwrap_binary()` on a non-binary value"),
        }
    }
    pub fn unwrap_text(&self) -> &str {
        match self {
            Self::Text(text) => text,
            _ => panic!("called `WasmModule::unwrap_text()` on a non-text value"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug() {
        assert_eq!(format!("{:?}", WasmModule::Binary(vec![])), "Binary([])");
        assert_eq!(
            format!("{:?}", WasmModule::Text("source".into())),
            r#"Text("source")"#
        );
    }

    #[test]
    #[should_panic(expected = "called `WasmModule::unwrap_text()` on a non-text value")]
    fn test_binary_panic_unwrap() {
        WasmModule::Binary(vec![]).unwrap_text();
    }

    #[test]
    #[should_panic(expected = "called `WasmModule::unwrap_binary()` on a non-binary value")]
    fn test_text_panic_unwrap() {
        WasmModule::Text("source".into()).unwrap_binary();
    }
}
