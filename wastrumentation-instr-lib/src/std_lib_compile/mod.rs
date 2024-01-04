pub mod assemblyscript;
pub type WasmModule = Vec<u8>;

pub trait CompilerOptions {
    fn source_code(&self) -> Vec<u8>;

    fn compile(&self) -> Box<dyn CompilerResult>;
}

pub trait CompilerResult {
    fn module(&self) -> Result<WasmModule, String>;
}
