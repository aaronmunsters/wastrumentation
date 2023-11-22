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
    _eoi: EndOfInput,
}

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::wasp))]
pub struct Wasp(Vec<AdviceDefinition>);

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
pub struct AdviceTrap(TrapSignature);

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
    GenHook(ApplyGenHook),
    GenIntro(ApplyGenIntro),
    GenInter(ApplyGenInter),
    SpeIntro(ApplySpeIntro),
    SpeInter(ApplySpeInter),
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
#[pest_ast(rule(Rule::apply_gen_intro))]
pub struct ApplyGenIntro {
    pub apply_formal_wasm_f: ApplyFormalWasmF,
    #[pest_ast(inner(with(span_into_string)))]
    pub args_identifier: String,
    #[pest_ast(inner(with(span_into_string)))]
    pub res_identifier: String,
}

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::apply_gen_inter))]
pub struct ApplyGenInter {
    pub apply_formal_wasm_f: ApplyFormalWasmF,
    #[pest_ast(inner(with(span_into_string)))]
    pub args_identifier: String,
    #[pest_ast(inner(with(span_into_string)))]
    pub res_identifier: String,
}

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::apply_spe_intro))]
pub struct ApplySpeIntro {
    pub apply_formal_wasm_f: ApplyFormalWasmF,
    pub formal_arguments_arguments: Vec<ApplyFormalArgument>,
    pub formal_arguments_results: Vec<ApplyFormalResult>,
}

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::apply_spe_inter))]
pub struct ApplySpeInter {
    pub apply_formal_wasm_f: ApplyFormalWasmF,
    pub formal_arguments_arguments: Vec<ApplyFormalArgument>,
    pub formal_arguments_results: Vec<ApplyFormalResult>,
}

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::apply_formal_wasm_f))]
pub struct ApplyFormalWasmF(#[pest_ast(inner(with(span_into_string)))] pub String);

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::apply_formal_argument))]
pub struct ApplyFormalArgument(TypedArgument);

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::apply_formal_result))]
pub struct ApplyFormalResult(TypedArgument);

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::typed_argument))]
pub struct TypedArgument {
    #[pest_ast(inner(with(span_into_string)))]
    identifier: String,
    #[pest_ast(inner(with(span_into_string)))]
    wasm_type: String,
}

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::EOI))]
struct EndOfInput;

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
            records: Wasp(vec![AdviceDefinition::AdviceGlobal(AdviceGlobal(
                r#">>>GUEST>>> console.log("Hello world!") <<<GUEST<<<"#.into(),
            ))]),
            _eoi: EndOfInput,
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
    fn aspect_trap_apply_hook() -> anyhow::Result<()> {
        let expected = WaspInput {
            records: Wasp(vec![AdviceDefinition::AdviceTrap(AdviceTrap(
                TrapSignature::TrapApply(TrapApply {
                    apply_hook_signature: ApplyHookSignature::GenHook(ApplyGenHook {
                        apply_formal_wasm_f: ApplyFormalWasmF(String::from("func")),
                        args_identifier: String::from("args"),
                        res_identifier: String::from("results"),
                    }),
                    body: String::from(">>>GUEST>>>global_function_count++;<<<GUEST<<<"),
                }),
            ))]),
            _eoi: EndOfInput,
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

    #[test]
    fn aspect_test_apply_spe_intro() -> anyhow::Result<()> {
        let expected = WaspInput {
            records: Wasp(vec![AdviceDefinition::AdviceTrap(AdviceTrap(
                TrapSignature::TrapApply(TrapApply {
                    apply_hook_signature: ApplyHookSignature::SpeIntro(ApplySpeIntro {
                        apply_formal_wasm_f: ApplyFormalWasmF("func".into()),
                        formal_arguments_arguments: vec![
                            ApplyFormalArgument(TypedArgument {
                                identifier: "a".into(),
                                wasm_type: "I32".into(),
                            }),
                            ApplyFormalArgument(TypedArgument {
                                identifier: "b".into(),
                                wasm_type: "I32".into(),
                            }),
                        ],
                        formal_arguments_results: vec![
                            ApplyFormalResult(TypedArgument {
                                identifier: "c".into(),
                                wasm_type: "F32".into(),
                            }),
                            ApplyFormalResult(TypedArgument {
                                identifier: "d".into(),
                                wasm_type: "F32".into(),
                            }),
                        ],
                    }),
                    body: ">>>GUEST>>>[🐇], [🔍], [🪖]<<<GUEST<<<".into(),
                }),
            ))]),
            _eoi: EndOfInput,
        };

        let mut parse_tree = WaspParser::parse(
            Rule::wasp_input,
            r#"
    (aspect
        (advice apply (func    WasmFunction)
                      ((a I32) (b I32))
                      ((c F32) (d F32))
                >>>GUEST>>>[🐇], [🔍], [🪖]<<<GUEST<<<))"#,
        )?;
        assert_eq!(WaspInput::from_pest(&mut parse_tree)?, expected);
        Ok(())
    }

    #[test]
    fn aspect_trap_applies() -> anyhow::Result<()> {
        let expected = WaspInput {
            records: Wasp(vec![
                AdviceDefinition::AdviceTrap(AdviceTrap(TrapSignature::TrapApply(TrapApply {
                    apply_hook_signature: ApplyHookSignature::GenHook(ApplyGenHook {
                        apply_formal_wasm_f: ApplyFormalWasmF(String::from("func")),
                        args_identifier: String::from("args"),
                        res_identifier: String::from("results"),
                    }),
                    body: String::from(">>>GUEST>>>[🐇], [🔍], [🙆‍]<<<GUEST<<<"),
                }))),
                AdviceDefinition::AdviceTrap(AdviceTrap(TrapSignature::TrapApply(TrapApply {
                    apply_hook_signature: ApplyHookSignature::GenIntro(ApplyGenIntro {
                        apply_formal_wasm_f: ApplyFormalWasmF(String::from("func")),
                        args_identifier: String::from("args"),
                        res_identifier: String::from("results"),
                    }),
                    body: String::from(">>>GUEST>>>[🐌], [🔍], [🙆‍]<<<GUEST<<<"),
                }))),
                AdviceDefinition::AdviceTrap(AdviceTrap(TrapSignature::TrapApply(TrapApply {
                    apply_hook_signature: ApplyHookSignature::GenInter(ApplyGenInter {
                        apply_formal_wasm_f: ApplyFormalWasmF(String::from("func")),
                        args_identifier: String::from("args"),
                        res_identifier: String::from("results"),
                    }),
                    body: String::from(">>>GUEST>>>[🐌], [📝], [🙆‍]<<<GUEST<<<"),
                }))),
                AdviceDefinition::AdviceTrap(AdviceTrap(TrapSignature::TrapApply(TrapApply {
                    apply_hook_signature: ApplyHookSignature::SpeIntro(ApplySpeIntro {
                        apply_formal_wasm_f: ApplyFormalWasmF(String::from("func")),
                        formal_arguments_arguments: vec![
                            ApplyFormalArgument(TypedArgument {
                                identifier: "a".into(),
                                wasm_type: "I32".into(),
                            }),
                            ApplyFormalArgument(TypedArgument {
                                identifier: "b".into(),
                                wasm_type: "I32".into(),
                            }),
                        ],
                        formal_arguments_results: vec![
                            ApplyFormalResult(TypedArgument {
                                identifier: "c".into(),
                                wasm_type: "F32".into(),
                            }),
                            ApplyFormalResult(TypedArgument {
                                identifier: "d".into(),
                                wasm_type: "F32".into(),
                            }),
                        ],
                    }),
                    body: String::from(">>>GUEST>>>[🐇], [🔍], [🪖]<<<GUEST<<<"),
                }))),
                AdviceDefinition::AdviceTrap(AdviceTrap(TrapSignature::TrapApply(TrapApply {
                    apply_hook_signature: ApplyHookSignature::SpeInter(ApplySpeInter {
                        apply_formal_wasm_f: ApplyFormalWasmF(String::from("func")),
                        formal_arguments_arguments: vec![
                            ApplyFormalArgument(TypedArgument {
                                identifier: "a".into(),
                                wasm_type: "I32".into(),
                            }),
                            ApplyFormalArgument(TypedArgument {
                                identifier: "b".into(),
                                wasm_type: "I32".into(),
                            }),
                        ],
                        formal_arguments_results: vec![
                            ApplyFormalResult(TypedArgument {
                                identifier: "c".into(),
                                wasm_type: "F32".into(),
                            }),
                            ApplyFormalResult(TypedArgument {
                                identifier: "d".into(),
                                wasm_type: "F32".into(),
                            }),
                        ],
                    }),
                    body: String::from(">>>GUEST>>>[🐇], [📝], [🪖]<<<GUEST<<<"),
                }))),
            ]),
            _eoi: EndOfInput,
        };

        let mut parse_tree = WaspParser::parse(
            Rule::wasp_input,
            r#"
    (aspect
        (advice apply (func    WasmFunction)
                      (args    Args)
                      (results Results)
                >>>GUEST>>>[🐇], [🔍], [🙆‍]<<<GUEST<<<)
        (advice apply (func    WasmFunction)
                      (args    DynArgs)
                      (results DynResults)
                >>>GUEST>>>[🐌], [🔍], [🙆‍]<<<GUEST<<<)
        (advice apply (func    WasmFunction)
                      (args    DynMutArgs)
                      (results DynMutResults)
                >>>GUEST>>>[🐌], [📝], [🙆‍]<<<GUEST<<<)
        (advice apply (func    WasmFunction)
                      ((a I32) (b I32))
                      ((c F32) (d F32))
                >>>GUEST>>>[🐇], [🔍], [🪖]<<<GUEST<<<)
        (advice apply (func    WasmFunction)
                      (Mut (a I32) (b I32))
                      (Mut (c F32) (d F32))
                >>>GUEST>>>[🐇], [📝], [🪖]<<<GUEST<<<)
    )"#,
        )?;
        assert_eq!(WaspInput::from_pest(&mut parse_tree)?, expected);
        Ok(())
    }
}
