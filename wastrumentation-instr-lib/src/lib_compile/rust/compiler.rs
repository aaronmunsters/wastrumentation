use rust_to_wasm_compiler::{Profile, RustToWasmCompiler};
use wastrumentation::compiler::{
    CompilationError, CompilationResult, Compiles, DefaultCompilerOptions,
};

use super::{
    options::{CompilerOptions, ManifestSource, RustSource, RustSourceCode},
    Rust,
};

pub struct Compiler {
    compiler: RustToWasmCompiler,
}

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

    fn setup_compiler() -> anyhow::Result<Self> {
        Ok(Self {
            compiler: RustToWasmCompiler::new()?,
        })
    }

    fn compile(&self, compiler_options: &Self::CompilerOptions) -> CompilationResult<Rust> {
        match &compiler_options.source {
            RustSource::SourceCode(
                ManifestSource(manifest_source_code),
                RustSourceCode(rust_source_code),
            ) => self.compiler.compile_source(
                manifest_source_code,
                rust_source_code,
                compiler_options.profile,
            ),
            RustSource::Manifest(manifest_path) => self
                .compiler
                .compile(manifest_path, compiler_options.profile),
        }
        .map_err(|err| CompilationError::because(err.to_string()))
    }
}
