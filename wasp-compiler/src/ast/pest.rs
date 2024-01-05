use pest::Span;
use pest_ast::FromPest;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "wasp.pest"]
pub struct WaspParser;

fn span_into_string(span: Span) -> String {
    span.as_str().to_string()
}

fn drop_guest_delimiter(guest_code: String) -> String {
    guest_code
        .strip_prefix(">>>GUEST>>>")
        .unwrap()
        .strip_suffix("<<<GUEST<<<")
        .unwrap()
        .to_string()
}

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::wasp_input))]
pub struct WaspInput {
    pub records: Wasp,
    _eoi: EndOfInput,
}

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::wasp))]
pub struct Wasp(pub Vec<AdviceDefinition>);

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::advice_definition))]
pub enum AdviceDefinition {
    AdviceGlobal(AdviceGlobal),
    AdviceTrap(AdviceTrap),
}

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::advice_global))]
pub struct AdviceGlobal(
    #[pest_ast(inner(with(span_into_string), with(drop_guest_delimiter)))] pub String,
);

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::advice_trap))]
pub struct AdviceTrap(pub TrapSignature);

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::trap_signature))]
pub enum TrapSignature {
    TrapApply(TrapApply),
}

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::trap_apply))]
pub struct TrapApply {
    pub apply_hook_signature: ApplyHookSignature,
    #[pest_ast(inner(with(span_into_string), with(drop_guest_delimiter)))]
    pub body: String,
}

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::apply_hook_signature))]
pub enum ApplyHookSignature {
    Gen(ApplyGen),
    SpeInter(ApplySpeInter),
    SpeIntro(ApplySpeIntro),
}

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::apply_gen))]
pub struct ApplyGen {
    pub apply_formal_wasm_f: ApplyFormalWasmF,
    pub apply_formal_argument: ApplyFormalArgument,
    pub apply_formal_result: ApplyFormalResult,
}

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::apply_spe_inter))]
pub struct ApplySpeInter {
    pub apply_formal_wasm_f: ApplyFormalWasmF,
    pub formal_arguments_arguments: Vec<ApplyFormalArgument>,
    pub formal_arguments_results: Vec<ApplyFormalResult>,
}

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::apply_spe_intro))]
pub struct ApplySpeIntro {
    pub apply_formal_wasm_f: ApplyFormalWasmF,
    pub formal_arguments_arguments: Vec<ApplyFormalArgument>,
    pub formal_arguments_results: Vec<ApplyFormalResult>,
}

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::apply_formal_wasm_f))]
pub struct ApplyFormalWasmF(#[pest_ast(inner(with(span_into_string)))] pub String);

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::apply_formal_argument))]
pub struct ApplyFormalArgument(pub TypedArgument);

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::apply_formal_result))]
pub struct ApplyFormalResult(pub TypedArgument);

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::typed_argument))]
pub struct TypedArgument {
    #[pest_ast(inner(with(span_into_string)))]
    pub identifier: String,
    #[pest_ast(inner(with(span_into_string)))]
    pub type_identifier: String,
}

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::EOI))]
struct EndOfInput;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WaspParser;
    use from_pest::FromPest;
    use pest::Parser;

    #[test]
    #[should_panic]
    fn fail_parse() {
        let mut parse_tree = WaspParser::parse(Rule::wasp_input, "(aspect)").unwrap();
        let WaspInput { records, _eoi } = WaspInput::from_pest(&mut parse_tree).unwrap();
    }

    #[test]
    fn aspect_global_empty() {
        let mut parse_tree = WaspParser::parse(Rule::wasp_input, "(aspect)").unwrap();
        let WaspInput { .. } = WaspInput::from_pest(&mut parse_tree).unwrap();
    }

    #[test]
    fn aspect_global_only() {
        let expected = WaspInput {
            records: Wasp(vec![AdviceDefinition::AdviceGlobal(AdviceGlobal(
                r#" console.log("Hello world!") "#.into(),
            ))]),
            _eoi: EndOfInput,
        };
        let mut parse_tree = WaspParser::parse(
            Rule::wasp_input,
            r#"
        (aspect
            (global >>>GUEST>>> console.log("Hello world!") <<<GUEST<<<))"#,
        )
        .unwrap();
        assert_eq!(WaspInput::from_pest(&mut parse_tree).unwrap(), expected);
    }

    #[test]
    fn aspect_trap_apply_hook() {
        let expected = WaspInput {
            records: Wasp(vec![AdviceDefinition::AdviceTrap(AdviceTrap(
                TrapSignature::TrapApply(TrapApply {
                    apply_hook_signature: ApplyHookSignature::Gen(ApplyGen {
                        apply_formal_wasm_f: ApplyFormalWasmF(String::from("func")),
                        apply_formal_argument: ApplyFormalArgument(TypedArgument {
                            identifier: "args".into(),
                            type_identifier: "Args".into(),
                        }),
                        apply_formal_result: ApplyFormalResult(TypedArgument {
                            identifier: "results".into(),
                            type_identifier: "Results".into(),
                        }),
                    }),
                    body: String::from("global_function_count++;"),
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
        )
        .unwrap();
        assert_eq!(WaspInput::from_pest(&mut parse_tree).unwrap(), expected);
    }

    #[test]
    fn aspect_test_apply_spe_intro() {
        let expected = WaspInput {
            records: Wasp(vec![AdviceDefinition::AdviceTrap(AdviceTrap(
                TrapSignature::TrapApply(TrapApply {
                    apply_hook_signature: ApplyHookSignature::SpeIntro(ApplySpeIntro {
                        apply_formal_wasm_f: ApplyFormalWasmF("func".into()),
                        formal_arguments_arguments: vec![
                            ApplyFormalArgument(TypedArgument {
                                identifier: "a".into(),
                                type_identifier: "I32".into(),
                            }),
                            ApplyFormalArgument(TypedArgument {
                                identifier: "b".into(),
                                type_identifier: "I32".into(),
                            }),
                        ],
                        formal_arguments_results: vec![
                            ApplyFormalResult(TypedArgument {
                                identifier: "c".into(),
                                type_identifier: "F32".into(),
                            }),
                            ApplyFormalResult(TypedArgument {
                                identifier: "d".into(),
                                type_identifier: "F32".into(),
                            }),
                        ],
                    }),
                    body: "[🐇], [🔍], [🪖]".into(),
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
        )
        .unwrap();
        assert_eq!(WaspInput::from_pest(&mut parse_tree).unwrap(), expected);
    }

    #[test]
    fn aspect_test_apply_spe_inter() {
        let expected = WaspInput {
            records: Wasp(vec![AdviceDefinition::AdviceTrap(AdviceTrap(
                TrapSignature::TrapApply(TrapApply {
                    apply_hook_signature: ApplyHookSignature::SpeInter(ApplySpeInter {
                        apply_formal_wasm_f: ApplyFormalWasmF("func".into()),
                        formal_arguments_arguments: vec![
                            ApplyFormalArgument(TypedArgument {
                                identifier: "a".into(),
                                type_identifier: "I32".into(),
                            }),
                            ApplyFormalArgument(TypedArgument {
                                identifier: "b".into(),
                                type_identifier: "I32".into(),
                            }),
                        ],
                        formal_arguments_results: vec![
                            ApplyFormalResult(TypedArgument {
                                identifier: "c".into(),
                                type_identifier: "F32".into(),
                            }),
                            ApplyFormalResult(TypedArgument {
                                identifier: "d".into(),
                                type_identifier: "F32".into(),
                            }),
                        ],
                    }),
                    body: "[🐇], [🔍], [🪖]".into(),
                }),
            ))]),
            _eoi: EndOfInput,
        };

        let mut parse_tree = WaspParser::parse(
            Rule::wasp_input,
            r#"
    (aspect
        (advice apply (func    WasmFunction)
                      (Mut (a I32) (b I32))
                      (Mut (c F32) (d F32))
                >>>GUEST>>>[🐇], [🔍], [🪖]<<<GUEST<<<))"#,
        )
        .unwrap();
        assert_eq!(WaspInput::from_pest(&mut parse_tree).unwrap(), expected);
    }

    #[test]
    fn aspect_trap_applies() {
        let expected = WaspInput {
            records: Wasp(vec![
                AdviceDefinition::AdviceTrap(AdviceTrap(TrapSignature::TrapApply(TrapApply {
                    apply_hook_signature: ApplyHookSignature::Gen(ApplyGen {
                        apply_formal_wasm_f: ApplyFormalWasmF(String::from("func")),
                        apply_formal_argument: ApplyFormalArgument(TypedArgument {
                            identifier: "args".into(),
                            type_identifier: "Args".into(),
                        }),
                        apply_formal_result: ApplyFormalResult(TypedArgument {
                            identifier: "results".into(),
                            type_identifier: "Results".into(),
                        }),
                    }),
                    body: String::from("[🐇], [🔍], [🙆‍]"),
                }))),
                AdviceDefinition::AdviceTrap(AdviceTrap(TrapSignature::TrapApply(TrapApply {
                    apply_hook_signature: ApplyHookSignature::Gen(ApplyGen {
                        apply_formal_wasm_f: ApplyFormalWasmF(String::from("func")),
                        apply_formal_argument: ApplyFormalArgument(TypedArgument {
                            identifier: "args".into(),
                            type_identifier: "DynArgs".into(),
                        }),
                        apply_formal_result: ApplyFormalResult(TypedArgument {
                            identifier: "results".into(),
                            type_identifier: "DynResults".into(),
                        }),
                    }),
                    body: String::from("[🐌], [🔍], [🙆‍]"),
                }))),
                AdviceDefinition::AdviceTrap(AdviceTrap(TrapSignature::TrapApply(TrapApply {
                    apply_hook_signature: ApplyHookSignature::Gen(ApplyGen {
                        apply_formal_wasm_f: ApplyFormalWasmF(String::from("func")),
                        apply_formal_argument: ApplyFormalArgument(TypedArgument {
                            identifier: "args".into(),
                            type_identifier: "MutDynArgs".into(),
                        }),
                        apply_formal_result: ApplyFormalResult(TypedArgument {
                            identifier: "results".into(),
                            type_identifier: "MutDynResults".into(),
                        }),
                    }),
                    body: String::from("[🐌], [📝], [🙆‍]"),
                }))),
                AdviceDefinition::AdviceTrap(AdviceTrap(TrapSignature::TrapApply(TrapApply {
                    apply_hook_signature: ApplyHookSignature::SpeIntro(ApplySpeIntro {
                        apply_formal_wasm_f: ApplyFormalWasmF("func".into()),
                        formal_arguments_arguments: vec![
                            ApplyFormalArgument(TypedArgument {
                                identifier: "a".into(),
                                type_identifier: "I32".into(),
                            }),
                            ApplyFormalArgument(TypedArgument {
                                identifier: "b".into(),
                                type_identifier: "I32".into(),
                            }),
                        ],
                        formal_arguments_results: vec![
                            ApplyFormalResult(TypedArgument {
                                identifier: "c".into(),
                                type_identifier: "F32".into(),
                            }),
                            ApplyFormalResult(TypedArgument {
                                identifier: "d".into(),
                                type_identifier: "F32".into(),
                            }),
                        ],
                    }),
                    body: String::from("[🐇], [🔍], [🪖]"),
                }))),
                AdviceDefinition::AdviceTrap(AdviceTrap(TrapSignature::TrapApply(TrapApply {
                    apply_hook_signature: ApplyHookSignature::SpeInter(ApplySpeInter {
                        apply_formal_wasm_f: ApplyFormalWasmF("func".into()),
                        formal_arguments_arguments: vec![
                            ApplyFormalArgument(TypedArgument {
                                identifier: "a".into(),
                                type_identifier: "I32".into(),
                            }),
                            ApplyFormalArgument(TypedArgument {
                                identifier: "b".into(),
                                type_identifier: "I32".into(),
                            }),
                        ],
                        formal_arguments_results: vec![
                            ApplyFormalResult(TypedArgument {
                                identifier: "c".into(),
                                type_identifier: "F32".into(),
                            }),
                            ApplyFormalResult(TypedArgument {
                                identifier: "d".into(),
                                type_identifier: "F32".into(),
                            }),
                        ],
                    }),
                    body: String::from("[🐇], [📝], [🪖]"),
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
                      (args    MutDynArgs)
                      (results MutDynResults)
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
        )
        .unwrap();
        assert_eq!(WaspInput::from_pest(&mut parse_tree).unwrap(), expected);
    }

    const CORRECT_PROGRAM: &'static str = r#"
        (aspect
            (advice apply (func    WasmFunction)
                          (args    Args)
                          (results Results)
                >>>GUEST>>>🔴<<<GUEST<<<)
            (advice apply (func    WasmFunction)
                          (args    DynArgs)
                          (results DynResults)
                >>>GUEST>>>🟠<<<GUEST<<<)
            (advice apply (func    WasmFunction)
                          (args    MutDynArgs)
                          (results MutDynResults)
                >>>GUEST>>>🟡<<<GUEST<<<)
            (advice apply (func    WasmFunction)
                          ((a I32) (b F32))
                          ((c I64) (d F64))
                >>>GUEST>>>🟢<<<GUEST<<<)
            (advice apply (func    WasmFunction)
                          (Mut (a I32) (b F32))
                          (Mut (c I64) (d F64))
                >>>GUEST>>>🔵<<<GUEST<<<)
            (global >>>GUEST>>>🟣<<<GUEST<<<))"#;

    #[test]
    fn test_debug() {
        let mut parse_tree = WaspParser::parse(Rule::wasp_input, CORRECT_PROGRAM).unwrap();
        let wasp_input = WaspInput::from_pest(&mut parse_tree).unwrap();
        assert_eq!(
            format!("{wasp_input:?}"),
            "WaspInput { \
                records: Wasp([\
                    AdviceTrap(AdviceTrap(TrapApply(TrapApply { \
                        apply_hook_signature: Gen(ApplyGen { \
                            apply_formal_wasm_f: ApplyFormalWasmF(\"func\"), \
                            apply_formal_argument: ApplyFormalArgument(TypedArgument { \
                                identifier: \"args\", \
                                type_identifier: \"Args\" \
                            }), \
                            apply_formal_result: ApplyFormalResult(TypedArgument { \
                                identifier: \"results\", \
                                type_identifier: \"Results\" \
                            }) \
                        }), \
                        body: \"🔴\" \
                    }))), \
                    AdviceTrap(AdviceTrap(TrapApply(TrapApply { \
                        apply_hook_signature: Gen(ApplyGen { \
                            apply_formal_wasm_f: ApplyFormalWasmF(\"func\"), \
                            apply_formal_argument: ApplyFormalArgument(TypedArgument { \
                                identifier: \"args\", \
                                type_identifier: \"DynArgs\" \
                            }), \
                            apply_formal_result: ApplyFormalResult(TypedArgument { \
                                identifier: \"results\", \
                                type_identifier: \"DynResults\" \
                            }) \
                        }), \
                        body: \"🟠\" \
                    }))), \
                    AdviceTrap(AdviceTrap(TrapApply(TrapApply { \
                        apply_hook_signature: Gen(ApplyGen { \
                            apply_formal_wasm_f: ApplyFormalWasmF(\"func\"), \
                            apply_formal_argument: ApplyFormalArgument(TypedArgument { \
                                identifier: \"args\", \
                                type_identifier: \"MutDynArgs\" \
                            }), \
                            apply_formal_result: ApplyFormalResult(TypedArgument { \
                                identifier: \"results\", \
                                type_identifier: \"MutDynResults\" \
                            }) \
                        }), \
                        body: \"🟡\" \
                    }))), \
                    AdviceTrap(AdviceTrap(TrapApply(TrapApply { \
                        apply_hook_signature: SpeIntro(ApplySpeIntro { \
                            apply_formal_wasm_f: ApplyFormalWasmF(\"func\"), \
                            formal_arguments_arguments: [\
                                ApplyFormalArgument(TypedArgument { \
                                    identifier: \"a\", \
                                    type_identifier: \"I32\" \
                                }), \
                                ApplyFormalArgument(TypedArgument { \
                                    identifier: \"b\", \
                                    type_identifier: \"F32\" \
                                })\
                            ], \
                            formal_arguments_results: [\
                                ApplyFormalResult(TypedArgument { \
                                    identifier: \"c\", \
                                    type_identifier: \"I64\" \
                                }), \
                                ApplyFormalResult(TypedArgument { \
                                    identifier: \"d\", \
                                    type_identifier: \"F64\" \
                                })\
                            ] \
                        }), \
                        body: \"🟢\" \
                    }))), \
                    AdviceTrap(AdviceTrap(TrapApply(TrapApply { \
                        apply_hook_signature: SpeInter(ApplySpeInter { \
                            apply_formal_wasm_f: ApplyFormalWasmF(\"func\"), \
                            formal_arguments_arguments: [\
                                ApplyFormalArgument(TypedArgument { \
                                    identifier: \"a\", \
                                    type_identifier: \"I32\" \
                                }), \
                                ApplyFormalArgument(TypedArgument { \
                                    identifier: \"b\", \
                                    type_identifier: \"F32\" \
                                })\
                            ], \
                            formal_arguments_results: [\
                                ApplyFormalResult(TypedArgument { \
                                    identifier: \"c\", \
                                    type_identifier: \"I64\" \
                                }), \
                                ApplyFormalResult(TypedArgument { \
                                    identifier: \"d\", \
                                    type_identifier: \"F64\" \
                                })\
                            ] \
                        }), \
                        body: \"🔵\" \
                    }))), \
                    AdviceGlobal(AdviceGlobal(\"🟣\"))\
                ]), \
                _eoi: EndOfInput \
            }"
        );
    }
}
