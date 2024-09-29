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
    type DefaultCompilerOptions: DefaultCompilerOptions<Self>;
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
    type CompilerSetupError;

    fn setup_compiler() -> Result<Self, Self::CompilerSetupError>;

    fn compile(&self, compiler_options: &Self::CompilerOptions) -> CompilationResult<Language>;
}

pub trait DefaultCompilerOptions<Language: SourceCodeBound> {
    fn default_for(library: Language::SourceCode) -> Self;
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use indoc::indoc;
    use wastrumentation::compiler::CompilationError;

    #[derive(Debug)]
    struct ExampleLanguage();

    #[test]
    fn test_debug() {
        let compilation_error = CompilationError::<ExampleLanguage> {
            language: PhantomData,
            reason: ("reason".into()),
        };

        let module_path = module_path!();

        let expectation = format! { indoc! { r#"
        CompilationError {{
            reason: "reason",
            language: PhantomData<{module_path}::ExampleLanguage>,
        }}"# }, module_path = module_path };

        assert_eq!(format!("{compilation_error:#?}",), expectation);
    }
}
