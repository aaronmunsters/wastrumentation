pub mod compiler;
pub mod options;

use wastrumentation::compiler::SourceCodeBound;

// Languages
#[derive(Debug, Clone)]
pub struct AssemblyScript;

impl SourceCodeBound for AssemblyScript {
    type DefaultCompilerOptions = assemblyscript_compiler::options::CompilerOptions;
    type SourceCode = String;
}
