use rust_to_wasm_compiler::{CompilerSetupError, Profile, RustToWasmCompiler};
use wastrumentation::compiler::{
    CompilationError, CompilationResult, Compiles, DefaultCompilerOptions,
};

use super::{
    Rust,
    options::{CompilerOptions, ManifestSource, RustSource, RustSourceCode},
};

pub struct Compiler {
    compiler: RustToWasmCompiler,
}

// TODO: for tests suite, would be nice to cover Profile::Release & Profile::Dev
impl DefaultCompilerOptions<Rust> for CompilerOptions {
    fn default_for(source: RustSource) -> Self {
        Self {
            profile: Profile::Release,
            source,
        }
    }
}

impl Compiles<Rust> for Compiler {
    type CompilerOptions = CompilerOptions;
    type CompilerSetupError = CompilerSetupError;

    fn setup_compiler() -> Result<Self, Self::CompilerSetupError> {
        Ok(Self {
            compiler: RustToWasmCompiler::new()?,
        })
    }

    fn compile(&self, compiler_options: &Self::CompilerOptions) -> CompilationResult<Rust> {
        match &compiler_options.source {
            RustSource::SourceCode(
                wasi_support,
                ManifestSource(manifest_source_code),
                RustSourceCode(rust_source_code),
            ) => self.compiler.compile_source(
                *wasi_support,
                manifest_source_code,
                rust_source_code,
                compiler_options.profile,
            ),
            RustSource::Manifest(wasi_support, manifest_path) => {
                self.compiler
                    .compile(*wasi_support, manifest_path, compiler_options.profile)
            }
        }
        .map_err(|err| CompilationError::because(err.to_string()))
    }
}
