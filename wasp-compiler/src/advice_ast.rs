use crate::Rule;
use pest::Span;
use pest_ast::FromPest;

fn span_into_string(span: Span) -> String {
    span.as_str().to_string()
}

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::wasp_input))]
pub struct WaspInput {
    pub records: Wasp,
    _eoi: EOI,
}

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::wasp))]
pub struct Wasp {
    pub records: Vec<AdviceDefinition>,
}

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::advice_definition))]
pub enum AdviceDefinition {
    AdviceGlobal(AdviceGlobal),
    AdviceTrap(AdviceTrap),
}

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::advice_global))]
pub struct AdviceGlobal(#[pest_ast(inner(with(span_into_string)))] pub String);

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::advice_trap))]
pub struct AdviceTrap {
    pub trap_signature: TrapSignature,
}

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::trap_signature))]
pub enum TrapSignature {
    TrapApply(TrapApply),
}

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::trap_apply))]
pub struct TrapApply {
    pub apply_hook_signature: ApplyHookSignature,
    #[pest_ast(inner(with(span_into_string)))]
    pub body: String,
}

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::apply_hook_signature))]
pub enum ApplyHookSignature {
    ApplyGenHook(ApplyGenHook),
}

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::apply_gen_hook))]
pub struct ApplyGenHook {
    pub apply_formal_wasm_f: ApplyFormalWasmF,
    #[pest_ast(inner(with(span_into_string)))]
    pub args_identifier: String,
    #[pest_ast(inner(with(span_into_string)))]
    pub res_identifier: String,
}

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::apply_formal_wasm_f))]
pub struct ApplyFormalWasmF(#[pest_ast(inner(with(span_into_string)))] pub String);

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::EOI))]
struct EOI;

#[cfg(test)]
mod tests {
    use anyhow::Ok;
    use from_pest::FromPest;
    use pest::Parser;

    use crate::WaspParser;

    use super::*;

    #[test]
    fn aspect_global_empty() -> anyhow::Result<()> {
        let mut parse_tree = WaspParser::parse(Rule::wasp_input, "(aspect)")?;
        let WaspInput { .. } = WaspInput::from_pest(&mut parse_tree)?;
        Ok(())
    }

    #[test]
    fn aspect_global_only() -> anyhow::Result<()> {
        let expected = WaspInput {
            records: Wasp {
                records: vec![AdviceDefinition::AdviceGlobal(AdviceGlobal(
                    r#">>>GUEST>>> console.log("Hello world!") <<<GUEST<<<"#.into(),
                ))],
            },
            _eoi: EOI,
        };
        let mut parse_tree = WaspParser::parse(
            Rule::wasp_input,
            r#"
        (aspect
            (global >>>GUEST>>> console.log("Hello world!") <<<GUEST<<<))"#,
        )?;
        assert_eq!(WaspInput::from_pest(&mut parse_tree)?, expected);
        Ok(())
    }

    #[test]
    fn aspect_trap_apply_only() -> anyhow::Result<()> {
        let expected = WaspInput {
            records: Wasp {
                records: vec![AdviceDefinition::AdviceTrap(AdviceTrap {
                    trap_signature: TrapSignature::TrapApply(TrapApply {
                        apply_hook_signature: ApplyHookSignature::ApplyGenHook(ApplyGenHook {
                            apply_formal_wasm_f: ApplyFormalWasmF(String::from("func")),
                            args_identifier: String::from("args"),
                            res_identifier: String::from("results"),
                        }),
                        body: String::from(">>>GUEST>>>global_function_count++;<<<GUEST<<<"),
                    }),
                })],
            },
            _eoi: EOI,
        };
        let mut parse_tree = WaspParser::parse(
            Rule::wasp_input,
            r#"
    (aspect
        (advice apply (func    WasmFunction)
                    (args    Args)
                    (results Results)
                >>>GUEST>>>global_function_count++;<<<GUEST<<<))"#,
        )?;
        assert_eq!(WaspInput::from_pest(&mut parse_tree)?, expected);
        Ok(())
    }
}
