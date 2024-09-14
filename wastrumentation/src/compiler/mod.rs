use std::marker::PhantomData;

use crate::wasm_constructs::Signature;

#[derive(Debug, Clone)]
pub struct Library<Language: SourceCodeBound> {
    pub content: Language::SourceCode,
    pub language: PhantomData<Language>,
}

/// Trait declaring that Self has a default compiler & is associated with a source code type
pub trait SourceCodeBound
where
    Self: Sized,
{
    type DefaultCompiler: DefaultCompilerOptions<Self>;
    type SourceCode;
}

pub trait LibGeneratable
where
    Self: Sized + SourceCodeBound,
{
    fn generate_lib(signatures: &[Signature]) -> Library<Self>;
}

pub type WasmModule = Vec<u8>;
pub type CompilationResult<Language> = Result<WasmModule, CompilationError<Language>>;

#[derive(Debug)]
pub struct CompilationError<Language> {
    pub reason: String,
    pub language: PhantomData<Language>,
}

impl<Language> CompilationError<Language> {
    pub fn because(reason: String) -> Self {
        Self {
            reason,
            language: PhantomData,
        }
    }

    pub fn reason(&self) -> &str {
        self.reason.as_str()
    }
}

pub trait Compiles<Language: SourceCodeBound>
where
    Self: Sized,
{
    type CompilerOptions: DefaultCompilerOptions<Language>;

    fn setup_compiler() -> anyhow::Result<Self>;

    fn compile(&self, compiler_options: &Self::CompilerOptions) -> CompilationResult<Language>;
}

pub trait DefaultCompilerOptions<Language: SourceCodeBound> {
    fn default_for(library: Language::SourceCode) -> Self;
}
