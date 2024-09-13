use std::marker::PhantomData;

use crate::SourceCodeBound;

pub mod assemblyscript;
pub mod rust;

pub type WasmModule = Vec<u8>;
pub type CompilationResult<Language> = Result<WasmModule, CompilationError<Language>>;

#[derive(Debug)]
pub struct CompilationError<Language> {
    reason: String,
    language: PhantomData<Language>,
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

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::AssemblyScript;

    use super::*;

    #[test]
    fn test_debug() {
        assert_eq!(
            format!(
                "{:#?}",
                CompilationError::<AssemblyScript> {
                    language: PhantomData,
                    reason: ("reason".into())
                }
            ),
            indoc! { r#"
            CompilationError {
                reason: "reason",
                language: PhantomData<wastrumentation_instr_lib::AssemblyScript>,
            }"# }
        );
    }
}
