use crate::ast::pest::{
    self as pest_ast, ApplyFormalArgument, ApplyFormalResult, ApplyFormalWasmF, ApplySpeInter,
    ApplySpeIntro,
};
use anyhow::anyhow;
use std::collections::HashSet;

const ARGS_HIGHLEVEL: &str = "Args";
const ARGS_DYNAMIC: &str = "DynArgs";
const ARGS_DYNAMIC_MUT: &str = "MutDynArgs";
const RESS_HIGHLEVEL: &str = "Results";
const RESS_DYNAMIC: &str = "DynResults";
const RESS_DYNAMIC_MUT: &str = "MutDynResults";

const I32_STR: &str = "I32";
const F32_STR: &str = "F32";
const I64_STR: &str = "I64";
const F64_STR: &str = "F64";

#[derive(Debug, PartialEq, Eq)]
pub struct WaspRoot(pub Vec<AdviceDefinition>);

#[derive(Debug, PartialEq, Eq)]
pub enum AdviceDefinition {
    AdviceGlobal(String),
    AdviceTrap(TrapSignature),
}

#[derive(Debug, PartialEq, Eq)]
pub enum TrapSignature {
    TrapApply(TrapApply),
    TrapIfThenElse(TrapIfThenElse),
}

#[derive(Debug, PartialEq, Eq)]
pub struct TrapApply {
    pub apply_hook_signature: ApplyHookSignature,
    pub body: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ApplyHookSignature {
    Gen(ApplyGen),
    Spe(ApplySpe),
}

#[derive(Debug, PartialEq, Eq)]
pub enum GenericTarget {
    HighLevel,
    Dynamic,
    MutableDynamic,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ApplyGen {
    pub generic_means: GenericTarget,
    pub parameter_apply: String,
    pub parameter_arguments: String,
    pub parameter_results: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ApplySpe {
    pub mutable_signature: bool,
    pub apply_parameter: String,
    pub parameters_arguments: Vec<WasmParameter>,
    pub parameters_results: Vec<WasmParameter>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TrapIfThenElse {
    pub if_then_else_hook_signature: IfThenElseHookSignature,
    pub body: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct IfThenElseHookSignature {
    pub parameter_condition: String,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum WasmType {
    I32,
    F32,
    I64,
    F64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct WasmParameter {
    pub identifier: String,
    pub identifier_type: WasmType,
}

impl WasmParameter {
    pub fn get_type(&self) -> WasmType {
        self.identifier_type
    }
}

impl WaspRoot {
    pub fn instruments_generic_apply(&self) -> bool {
        let Self(advice_definitions) = self;
        advice_definitions
            .iter()
            .any(|advice_definition: &AdviceDefinition| {
                matches!(
                    advice_definition,
                    AdviceDefinition::AdviceTrap(TrapSignature::TrapApply(TrapApply {
                        apply_hook_signature: ApplyHookSignature::Gen(_),
                        ..
                    }))
                )
            })
    }

    pub fn instruments_if_then_else(&self) -> bool {
        let Self(advice_definitions) = self;
        advice_definitions
            .iter()
            .any(|advice_definition: &AdviceDefinition| {
                matches!(
                    advice_definition,
                    AdviceDefinition::AdviceTrap(TrapSignature::TrapIfThenElse { .. })
                )
            })
    }
}

impl TryFrom<pest_ast::WaspInput> for WaspRoot {
    type Error = anyhow::Error;

    fn try_from(pest_wasp_input: pest_ast::WaspInput) -> Result<Self, Self::Error> {
        // TODO: determine what should happen if a specialization is defined more than once?
        // 1. Throw an error?
        // 2. Append them, in order of definition?
        // 3. Allow more specific joinpoint definitions?
        //      Difficulty here is to ensure that the joinpoint definitions are mutually exclusive when input program is not known aot
        let pest_ast::Wasp(pest_advice_definitions) = pest_wasp_input.records;
        let mut advice_definitions = Vec::with_capacity(pest_advice_definitions.len());
        for advice_definition in pest_advice_definitions {
            advice_definitions.push(AdviceDefinition::try_from(advice_definition)?);
        }
        Ok(WaspRoot(advice_definitions))
    }
}

impl TryFrom<pest_ast::AdviceDefinition> for AdviceDefinition {
    type Error = anyhow::Error;

    fn try_from(pest_advice_definition: pest_ast::AdviceDefinition) -> Result<Self, Self::Error> {
        match pest_advice_definition {
            pest_ast::AdviceDefinition::AdviceGlobal(pest_ast::AdviceGlobal(definition)) => {
                Ok(AdviceDefinition::AdviceGlobal(definition))
            }
            pest_ast::AdviceDefinition::AdviceTrap(pest_ast::AdviceTrap(trap_signature)) => Ok(
                AdviceDefinition::AdviceTrap(TrapSignature::try_from(trap_signature)?),
            ),
        }
    }
}

impl TryFrom<pest_ast::TrapSignature> for TrapSignature {
    type Error = anyhow::Error;

    fn try_from(pest_trap_signature: pest_ast::TrapSignature) -> Result<Self, Self::Error> {
        match pest_trap_signature {
            pest_ast::TrapSignature::TrapApply(pest_ast::TrapApply {
                apply_hook_signature,
                body,
            }) => Ok(TrapSignature::TrapApply(TrapApply {
                apply_hook_signature: ApplyHookSignature::try_from(apply_hook_signature)?,
                body,
            })),
            pest_ast::TrapSignature::TrapIfThenElse(pest_ast::TrapIfThenElse {
                if_then_else_hook_signature,
                body,
            }) => Ok(TrapSignature::TrapIfThenElse(TrapIfThenElse {
                if_then_else_hook_signature: IfThenElseHookSignature::from(
                    if_then_else_hook_signature,
                ),
                body,
            })),
        }
    }
}

impl From<pest_ast::IfThenElseHookSignature> for IfThenElseHookSignature {
    fn from(past_if_then_else_hook_signature: pest_ast::IfThenElseHookSignature) -> Self {
        let pest_ast::IfThenElseFormalCondition(parameter_condition) =
            past_if_then_else_hook_signature.if_then_else_formal_condition;
        IfThenElseHookSignature {
            parameter_condition,
        }
    }
}

impl TryFrom<pest_ast::ApplyHookSignature> for ApplyHookSignature {
    type Error = anyhow::Error;

    fn try_from(
        pest_apply_hook_signature: pest_ast::ApplyHookSignature,
    ) -> Result<Self, Self::Error> {
        match pest_apply_hook_signature {
            pest_ast::ApplyHookSignature::Gen(pest_apply_gen) => {
                Ok(ApplyHookSignature::Gen(ApplyGen::try_from(pest_apply_gen)?))
            }
            pest_ast::ApplyHookSignature::SpeInter(pest_apply_spe_inter) => Ok(
                ApplyHookSignature::Spe(ApplySpe::try_from(pest_apply_spe_inter)?),
            ),
            pest_ast::ApplyHookSignature::SpeIntro(pest_apply_spe_intro) => Ok(
                ApplyHookSignature::Spe(ApplySpe::try_from(pest_apply_spe_intro)?),
            ),
        }
    }
}

impl TryFrom<pest_ast::ApplyGen> for ApplyGen {
    type Error = anyhow::Error;

    fn try_from(pest_apply_gen: pest_ast::ApplyGen) -> Result<Self, Self::Error> {
        let pest_ast::ApplyGen {
            apply_formal_wasm_f: ApplyFormalWasmF(parameter_apply),
            apply_formal_argument,
            apply_formal_result,
        } = pest_apply_gen;
        let ApplyFormalArgument(formal_argument) = apply_formal_argument;
        let ApplyFormalResult(formal_result) = apply_formal_result;
        let generic_means: GenericTarget = match (
                    formal_argument.type_identifier.as_str(),
                    formal_result.type_identifier.as_str(),
                ) {
                    (ARGS_HIGHLEVEL, RESS_HIGHLEVEL) => GenericTarget::HighLevel,
                    (ARGS_DYNAMIC, RESS_DYNAMIC) => GenericTarget::Dynamic,
                    (ARGS_DYNAMIC_MUT, RESS_DYNAMIC_MUT) => GenericTarget::MutableDynamic,
                    (args, ress) => return Err(
                        anyhow!(
                            "Formal parameters must both be either high-level, dynamic or mutably dynamic (got: args {}, for ress {}).",
                            args,
                            ress
                        )
                    ),
                };

        let mut parameters = HashSet::with_capacity(3);
        parameters.insert(&parameter_apply);
        parameters.insert(&formal_argument.identifier);
        parameters.insert(&formal_result.identifier);
        if parameters.len() != 3 {
            return Err(anyhow!(
                "Parameters must be unique, got: {}, {}, {}.",
                &parameter_apply,
                &formal_argument.identifier,
                &formal_result.identifier
            ));
        }

        Ok(ApplyGen {
            generic_means,
            parameter_apply,
            parameter_arguments: formal_argument.identifier,
            parameter_results: formal_result.identifier,
        })
    }
}

impl TryFrom<pest_ast::ApplySpeInter> for ApplySpe {
    type Error = anyhow::Error;

    fn try_from(pest_apply_spe_inter: pest_ast::ApplySpeInter) -> Result<Self, Self::Error> {
        let ApplySpeInter {
            apply_formal_wasm_f: ApplyFormalWasmF(apply_parameter),
            formal_arguments_arguments,
            formal_arguments_results,
        } = pest_apply_spe_inter;
        let WasmParameterVec(parameters_arguments) =
            WasmParameterVec::try_from(formal_arguments_arguments)?;
        let WasmParameterVec(parameters_results) =
            WasmParameterVec::try_from(formal_arguments_results)?;
        WasmParameterVec::distinct_arguments(&parameters_arguments, &parameters_results)?;
        Ok(ApplySpe {
            mutable_signature: false,
            apply_parameter,
            parameters_arguments,
            parameters_results,
        })
    }
}

impl TryFrom<pest_ast::ApplySpeIntro> for ApplySpe {
    type Error = anyhow::Error;

    fn try_from(pest_apply_spe_intro: pest_ast::ApplySpeIntro) -> Result<Self, Self::Error> {
        let ApplySpeIntro {
            apply_formal_wasm_f: ApplyFormalWasmF(apply_parameter),
            formal_arguments_arguments,
            formal_arguments_results,
        } = pest_apply_spe_intro;
        let WasmParameterVec(parameters_arguments) =
            WasmParameterVec::try_from(formal_arguments_arguments)?;
        let WasmParameterVec(parameters_results) =
            WasmParameterVec::try_from(formal_arguments_results)?;
        WasmParameterVec::distinct_arguments(&parameters_arguments, &parameters_results)?;
        Ok(ApplySpe {
            mutable_signature: true,
            apply_parameter,
            parameters_arguments,
            parameters_results,
        })
    }
}

struct WasmParameterVec(Vec<WasmParameter>);
impl WasmParameterVec {
    fn distinct_arguments(
        parameters_1: &[WasmParameter],
        parameters_2: &[WasmParameter],
    ) -> anyhow::Result<()> {
        let mut parameters: HashSet<String> = HashSet::with_capacity(parameters_1.len() - 1);
        for parameter in parameters_1.iter().chain(parameters_2.iter()) {
            if parameters.contains(parameter.identifier.as_str()) {
                return Err(anyhow!(
                    "Duplicate paramater accross arguments and results: {}.",
                    parameter.identifier
                ));
            }
            parameters.insert(parameter.identifier.to_string());
        }
        Ok(())
    }
}

impl TryFrom<Vec<pest_ast::TypedArgument>> for WasmParameterVec {
    type Error = anyhow::Error;

    fn try_from(pest_typed_arguments: Vec<pest_ast::TypedArgument>) -> Result<Self, Self::Error> {
        let mut wasm_type_vec: Vec<WasmParameter> = Vec::with_capacity(pest_typed_arguments.len());
        let mut arguments_identifiers: HashSet<String> =
            HashSet::with_capacity(pest_typed_arguments.len());
        for typed_argument in pest_typed_arguments {
            let pest_ast::TypedArgument {
                identifier,
                type_identifier,
            } = typed_argument;
            let identifier_type = match type_identifier.as_str() {
                I32_STR => WasmType::I32,
                F32_STR => WasmType::F32,
                I64_STR => WasmType::I64,
                F64_STR => WasmType::F64,
                unsupported_type => {
                    return Err(anyhow!(
                    "Provided type {} unsupported, use one of following instead: {}, {}, {} & {}.",
                    unsupported_type,
                    I32_STR,
                    F32_STR,
                    I64_STR,
                    F64_STR
                ))
                }
            };
            if arguments_identifiers.contains(&identifier) {
                return Err(anyhow!("Duplicate parameter found: {}", &identifier));
            }
            arguments_identifiers.insert(identifier.clone());
            wasm_type_vec.push(WasmParameter {
                identifier,
                identifier_type,
            })
        }
        Ok(WasmParameterVec(wasm_type_vec))
    }
}

impl TryFrom<Vec<pest_ast::ApplyFormalArgument>> for WasmParameterVec {
    type Error = anyhow::Error;

    fn try_from(
        pest_apply_formal_arguments: Vec<pest_ast::ApplyFormalArgument>,
    ) -> Result<Self, Self::Error> {
        let typed_arguments: Vec<pest_ast::TypedArgument> = pest_apply_formal_arguments
            .into_iter()
            .map(|pest_apply_formal_argument| pest_apply_formal_argument.0)
            .collect();
        typed_arguments.try_into()
    }
}

impl TryFrom<Vec<pest_ast::ApplyFormalResult>> for WasmParameterVec {
    type Error = anyhow::Error;

    fn try_from(
        pest_apply_formal_results: Vec<pest_ast::ApplyFormalResult>,
    ) -> Result<Self, Self::Error> {
        let typed_arguments: Vec<pest_ast::TypedArgument> = pest_apply_formal_results
            .into_iter()
            .map(|pest_apply_formal_argument| pest_apply_formal_argument.0)
            .collect();
        typed_arguments.try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ast, Rule, WaspParser};
    use from_pest::FromPest;
    use pest::Parser;

    const CORRECT_PROGRAM: &str = r#"
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
            (global >>>GUEST>>>🟣<<<GUEST<<<)
            (advice if_then_else (cond Condition) >>>GUEST>>>🧂<<<GUEST<<<))"#;

    fn program_to_wasp_root(program: &str) -> anyhow::Result<WaspRoot> {
        let mut pest_parse = WaspParser::parse(Rule::wasp_input, program).unwrap();
        let wasp_input = ast::pest::WaspInput::from_pest(&mut pest_parse).unwrap();
        let wasp_root = WaspRoot::try_from(wasp_input)?;
        Ok(wasp_root)
    }

    #[test]
    fn should_convert_success_ast() {
        assert_eq!(
            program_to_wasp_root(CORRECT_PROGRAM).unwrap(),
            WaspRoot(vec![
                AdviceDefinition::AdviceTrap(TrapSignature::TrapApply(TrapApply {
                    apply_hook_signature: ApplyHookSignature::Gen(ApplyGen {
                        generic_means: GenericTarget::HighLevel,
                        parameter_apply: "func".into(),
                        parameter_arguments: "args".into(),
                        parameter_results: "results".into()
                    }),
                    body: "🔴".into()
                })),
                AdviceDefinition::AdviceTrap(TrapSignature::TrapApply(TrapApply {
                    apply_hook_signature: ApplyHookSignature::Gen(ApplyGen {
                        generic_means: GenericTarget::Dynamic,
                        parameter_apply: "func".into(),
                        parameter_arguments: "args".into(),
                        parameter_results: "results".into()
                    }),
                    body: "🟠".into()
                })),
                AdviceDefinition::AdviceTrap(TrapSignature::TrapApply(TrapApply {
                    apply_hook_signature: ApplyHookSignature::Gen(ApplyGen {
                        generic_means: GenericTarget::MutableDynamic,
                        parameter_apply: "func".into(),
                        parameter_arguments: "args".into(),
                        parameter_results: "results".into()
                    }),
                    body: "🟡".into()
                })),
                AdviceDefinition::AdviceTrap(TrapSignature::TrapApply(TrapApply {
                    apply_hook_signature: ApplyHookSignature::Spe(ApplySpe {
                        mutable_signature: true,
                        apply_parameter: "func".into(),
                        parameters_arguments: vec![
                            WasmParameter {
                                identifier: "a".into(),
                                identifier_type: WasmType::I32
                            },
                            WasmParameter {
                                identifier: "b".into(),
                                identifier_type: WasmType::F32
                            }
                        ],
                        parameters_results: vec![
                            WasmParameter {
                                identifier: "c".into(),
                                identifier_type: WasmType::I64
                            },
                            WasmParameter {
                                identifier: "d".into(),
                                identifier_type: WasmType::F64
                            }
                        ]
                    }),
                    body: "🟢".into()
                })),
                AdviceDefinition::AdviceTrap(TrapSignature::TrapApply(TrapApply {
                    apply_hook_signature: ApplyHookSignature::Spe(ApplySpe {
                        mutable_signature: false,
                        apply_parameter: "func".into(),
                        parameters_arguments: vec![
                            WasmParameter {
                                identifier: "a".into(),
                                identifier_type: WasmType::I32
                            },
                            WasmParameter {
                                identifier: "b".into(),
                                identifier_type: WasmType::F32
                            }
                        ],
                        parameters_results: vec![
                            WasmParameter {
                                identifier: "c".into(),
                                identifier_type: WasmType::I64
                            },
                            WasmParameter {
                                identifier: "d".into(),
                                identifier_type: WasmType::F64
                            }
                        ]
                    }),
                    body: "🔵".into()
                })),
                AdviceDefinition::AdviceGlobal("🟣".into()),
                AdviceDefinition::AdviceTrap(TrapSignature::TrapIfThenElse(TrapIfThenElse {
                    if_then_else_hook_signature: IfThenElseHookSignature {
                        parameter_condition: "cond".into()
                    },
                    body: "🧂".into()
                }))
            ])
        )
    }

    #[test]
    fn test_debug() {
        let wasp_root = program_to_wasp_root(CORRECT_PROGRAM).unwrap();
        let formatted = format!("{wasp_root:?}");
        for guest_code in ["🔴", "🟠", "🟡", "🟢", "🔵", "🟣", "🧂"] {
            assert!(formatted.contains(guest_code));
        }
    }

    #[test]
    fn test_errors_incorrect_parameters() {
        let outcomes = [
            ("(args Args          )", "(results DynResults )", "Formal parameters must both be either high-level, dynamic or mutably dynamic (got: args Args, for ress DynResults)."),
            ("(    (a FOO)        )", "(    (b I32)        )", "Provided type FOO unsupported, use one of following instead: I32, F32, I64 & F64."),
            ("(    (a I32) (a I32))", "(    (c I32)        )", "Duplicate parameter found: a"),
            ("(    (a I32)        )", "(    (c I32) (c I32))", "Duplicate parameter found: c"),
            ("(    (a I32)        )", "(    (a I32)        )", "Duplicate paramater accross arguments and results: a."),
            ("(Mut (a I32) (a I32))", "(Mut (c I32)        )", "Duplicate parameter found: a"),
            ("(Mut (a I32)        )", "(Mut (c I32) (c I64))", "Duplicate parameter found: c"),
            ("(Mut (a I32)        )", "(Mut (a I32)        )", "Duplicate paramater accross arguments and results: a."),
        ];

        for (parameter_arguments, parameter_results, message) in outcomes {
            let program = format!(
                "(aspect 
                    (advice apply (func WasmFunction) {} {}
                        >>>GUEST>>>🟢<<<GUEST<<<))",
                parameter_arguments, parameter_results
            );
            assert_eq!(
                program_to_wasp_root(program.as_str())
                    .unwrap_err()
                    .to_string()
                    .as_str(),
                message
            );
        }
    }
    #[test]
    fn test_errors_incorrect_parameters_duplicate() {
        let program: String =
            "(aspect (advice apply (a WasmFunction) (a Args) (a Results) >>>GUEST>>>🟢<<<GUEST<<<))".into();
        assert_eq!(
            program_to_wasp_root(&program)
                .unwrap_err()
                .to_string()
                .as_str(),
            "Parameters must be unique, got: a, a, a."
        );
    }

    #[test]
    fn test_wasm_type() {
        let x = WasmType::I32;
        let y = x;
        assert_eq!(format!("{x}, {y}"), "i32, i32");

        // assert cloning behavior works
        let e = &x;
        assert_eq!(*e, e.clone());
    }
}
