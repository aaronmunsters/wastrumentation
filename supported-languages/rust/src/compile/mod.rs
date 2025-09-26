pub mod compiler;
pub mod options;

use wastrumentation::compiler::SourceCodeBound;

#[derive(Debug, Clone)]
pub struct Rust;

impl SourceCodeBound for Rust {
    type DefaultCompilerOptions = options::CompilerOptions;
    type SourceCode = options::RustSource;
}
