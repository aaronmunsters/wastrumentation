use wastrumentation::compiler::{
    CompilationError, CompilationResult, Compiles, DefaultCompilerOptions,
};

use super::{
    WebAssembly,
    options::{CompilerOptions, WebAssemblySource},
};

#[derive(Debug)]
pub struct Compiler;

impl DefaultCompilerOptions<WebAssembly> for CompilerOptions {
    fn default_for(source: WebAssemblySource) -> Self {
        Self { source }
    }
}

impl Compiles<WebAssembly> for Compiler {
    type CompilerOptions = CompilerOptions;
    type CompilerSetupError = ();

    fn setup_compiler() -> Result<Self, Self::CompilerSetupError> {
        Ok(Self)
    }

    fn compile(&self, compiler_options: &Self::CompilerOptions) -> CompilationResult<WebAssembly> {
        match &compiler_options.source {
            WebAssemblySource::Module(module) => Ok(module.clone()),
            WebAssemblySource::Wat(wat) => wat::parse_str(wat).map_err(|err| CompilationError {
                reason: err.to_string(),
                language: std::marker::PhantomData,
            }),
        }
    }
}
