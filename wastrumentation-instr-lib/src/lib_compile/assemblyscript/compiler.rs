use anyhow::Result;

use super::AssemblyScript;

pub use assemblyscript_compiler::compiler::Compiler;
use assemblyscript_compiler::options::CompilerOptions;

use wastrumentation::compiler::{CompilationError, CompilationResult, Compiles};

impl Compiles<AssemblyScript> for Compiler {
    type CompilerOptions = CompilerOptions;

    fn setup_compiler() -> Result<Self> {
        Self::new()
    }

    fn compile(
        &self,
        compiler_options: &Self::CompilerOptions,
    ) -> CompilationResult<AssemblyScript> {
        self.compile(compiler_options)
            .map_err(|err| CompilationError::because(err.to_string()))
    }
}
