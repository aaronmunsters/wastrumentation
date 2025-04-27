pub use asc_compiler_rs::options::CompilerOptions;
use wastrumentation::compiler::DefaultCompilerOptions;

use super::AssemblyScript;

impl DefaultCompilerOptions<AssemblyScript> for CompilerOptions {
    fn default_for(
        library: <AssemblyScript as wastrumentation::compiler::SourceCodeBound>::SourceCode,
    ) -> Self {
        CompilerOptions::default_for(library)
    }
}
