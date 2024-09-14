pub mod assemblyscript;
pub mod rust;

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use indoc::indoc;
    use wastrumentation::compiler::CompilationError;

    use crate::AssemblyScript;

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
