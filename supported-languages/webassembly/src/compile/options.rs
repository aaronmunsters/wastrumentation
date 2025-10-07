#[derive(Debug, Clone)]
pub enum WebAssemblySource {
    Module(Vec<u8>),
    Wat(String),
}

pub struct CompilerOptions {
    pub source: WebAssemblySource,
}
