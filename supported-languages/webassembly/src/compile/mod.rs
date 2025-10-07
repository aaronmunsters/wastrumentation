pub mod compiler;
pub mod options;

use wastrumentation::compiler::SourceCodeBound;

#[derive(Debug, Clone)]
pub struct WebAssembly;

impl SourceCodeBound for WebAssembly {
    type DefaultCompilerOptions = options::CompilerOptions;
    type SourceCode = options::WebAssemblySource;
}
