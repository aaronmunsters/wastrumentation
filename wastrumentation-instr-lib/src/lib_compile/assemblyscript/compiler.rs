use super::AssemblyScript;

pub use assemblyscript_compiler::compiler::Compiler;
use assemblyscript_compiler::{error::CompilerSetupError, options::CompilerOptions};

use wastrumentation::compiler::{CompilationError, CompilationResult, Compiles};

impl Compiles<AssemblyScript> for Compiler {
    type CompilerOptions = CompilerOptions;
    type CompilerSetupError = CompilerSetupError;

    fn setup_compiler() -> std::result::Result<Self, Self::CompilerSetupError> {
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
