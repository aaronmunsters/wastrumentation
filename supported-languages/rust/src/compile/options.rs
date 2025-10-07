use std::path::PathBuf;

pub use rust_to_wasm_compiler::Profile;
pub use rust_to_wasm_compiler::WasiSupport;

#[derive(Debug, Clone)]
pub struct ManifestSource(pub String);
#[derive(Debug, Clone)]
pub struct RustSourceCode(pub String);

#[derive(Debug, Clone)]
pub enum RustSource {
    SourceCode(WasiSupport, ManifestSource, RustSourceCode),
    Manifest(WasiSupport, PathBuf),
}

pub struct CompilerOptions {
    pub source: RustSource,
    pub profile: Profile,
}
