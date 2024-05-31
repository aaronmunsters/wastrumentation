use std::collections::HashSet;

use crate::ast::{
    pest::CallQualifier::{After, Before},
    wasp::{ApplyHookSignature, ApplySpe, Root, TrapCall, TrapSignature, WasmParameter, WasmType},
};

#[derive(Debug, PartialEq, Eq, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct JoinPoints {
    pub generic: bool,
    pub specialized: HashSet<SpecialisedJoinPoint>,
    pub if_then: bool,
    pub if_then_else: bool,
    pub br_if: bool,
    pub br_table: bool,
    pub call_pre: bool,
    pub call_post: bool,
    pub call_indirect_pre: bool,
    pub call_indirect_post: bool,
    pub block_pre: bool,
    pub block_post: bool,
    pub loop_pre: bool,
    pub loop_post: bool,
    pub select: bool,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct SpecialisedJoinPoint {
    result_types: Vec<WasmType>,
    argument_types: Vec<WasmType>,
}

impl JoinPoints {
    fn include(&mut self, join_point: JoinPoint) {
        match join_point {
            JoinPoint::Specialised(specialized_join_point) => {
                self.specialized.insert(specialized_join_point);
            }
            JoinPoint::Generic => self.generic = true,
            JoinPoint::IfThen => self.if_then = true,
            JoinPoint::IfThenElse => self.if_then_else = true,
            JoinPoint::BrIf => self.br_if = true,
            JoinPoint::CallPre => self.call_pre = true,
            JoinPoint::CallPost => self.call_post = true,
            JoinPoint::CallIndirectPre => self.call_indirect_pre = true,
            JoinPoint::CallIndirectPost => self.call_indirect_post = true,
            JoinPoint::TrapBrTable => self.br_table = true,
            JoinPoint::BlockBefore => self.block_pre = true,
            JoinPoint::BlockAfter => self.block_post = true,
            JoinPoint::LoopBefore => self.loop_pre = true,
            JoinPoint::LoopAfter => self.loop_post = true,
            JoinPoint::Select => self.select = true,
        };
    }
}

enum JoinPoint {
    Generic,
    Specialised(SpecialisedJoinPoint),
    BlockBefore,
    BlockAfter,
    LoopBefore,
    LoopAfter,
    Select,
    CallPre,
    CallPost,
    CallIndirectPre,
    CallIndirectPost,
    IfThen,
    IfThenElse,
    BrIf,
    TrapBrTable,
}

impl Root {
    #[must_use]
    pub fn join_points(&self) -> JoinPoints {
        let Self(advice_definitions) = self;
        let mut join_points = JoinPoints::default();
        for advice_definition in advice_definitions {
            match advice_definition {
                crate::ast::wasp::AdviceDefinition::AdviceGlobal(_) => {}
                crate::ast::wasp::AdviceDefinition::AdviceTrap(trap_signature) => {
                    join_points.include(trap_signature.join_point());
                }
            };
        }
        join_points
    }
}

impl TrapSignature {
    fn join_point(&self) -> JoinPoint {
        match self {
            TrapSignature::TrapApply(trap_apply) => trap_apply.apply_hook_signature.join_point(),
            TrapSignature::TrapIfThen(_) => JoinPoint::IfThen,
            TrapSignature::TrapIfThenElse(_) => JoinPoint::IfThenElse,
            TrapSignature::TrapBrIf(_) => JoinPoint::BrIf,
            TrapSignature::TrapCall(TrapCall {
                call_qualifier: Before,
                ..
            }) => JoinPoint::CallPre,
            TrapSignature::TrapCall(TrapCall {
                call_qualifier: After,
                ..
            }) => JoinPoint::CallPost,
            TrapSignature::TrapCallIndirectBefore(_) => JoinPoint::CallIndirectPre,
            TrapSignature::TrapCallIndirectAfter(_) => JoinPoint::CallIndirectPost,
            TrapSignature::TrapBrTable(_) => JoinPoint::TrapBrTable,
            TrapSignature::TrapBlockBefore(_) => JoinPoint::BlockBefore,
            TrapSignature::TrapBlockAfter(_) => JoinPoint::BlockAfter,
            TrapSignature::TrapLoopBefore(_) => JoinPoint::LoopBefore,
            TrapSignature::TrapLoopAfter(_) => JoinPoint::LoopAfter,
            TrapSignature::TrapSelect(_) => JoinPoint::Select,
        }
    }
}

impl ApplyHookSignature {
    fn join_point(&self) -> JoinPoint {
        match self {
            ApplyHookSignature::Gen(_) => JoinPoint::Generic,
            ApplyHookSignature::Spe(apply_spe) => JoinPoint::Specialised(apply_spe.join_point()),
        }
    }
}

impl ApplySpe {
    fn join_point(&self) -> SpecialisedJoinPoint {
        let extract_types =
            |parameters: &Vec<_>| parameters.iter().map(WasmParameter::get_type).collect();

        SpecialisedJoinPoint {
            result_types: extract_types(&self.parameters_results),
            argument_types: extract_types(&self.parameters_arguments),
        }
    }
}

#[cfg(test)]
mod tests {
    use from_pest::FromPest;
    use indoc::indoc;
    use pest::Parser;

    use super::*;
    use crate::ast::pest::{Rule, WaspInput, WaspParser};

    fn get_joinpoints(wasp: &str) -> JoinPoints {
        let mut pest_parse = WaspParser::parse(Rule::wasp_input, wasp).unwrap();
        let wasp_input = WaspInput::from_pest(&mut pest_parse).expect("pest to input");
        let wasp_root = Root::try_from(wasp_input).unwrap();
        wasp_root.join_points()
    }

    #[test]
    fn test_empty_joinpoints() {
        assert_eq!(
            get_joinpoints(
                r#"
                (aspect
                    (global >>>GUEST>>>1+2+3+4+5<<<GUEST<<<))
                "#,
            ),
            JoinPoints::default()
        )
    }
    #[test]

    fn test_debug() {
        let joinpoints = get_joinpoints(
            r#"
            (aspect
                (advice apply (func    WasmFunction)
                              ((a I32) (b F32) (c I64))
                              ((d F64))
                    >>>GUEST>>>🟢<<<GUEST<<<))
            "#,
        );
        assert_eq!(
            format!("{joinpoints:#?}"),
            indoc! {r#"JoinPoints {
                generic: false,
                specialized: {
                    SpecialisedJoinPoint {
                        result_types: [
                            F64,
                        ],
                        argument_types: [
                            I32,
                            F32,
                            I64,
                        ],
                    },
                },
                if_then: false,
                if_then_else: false,
                br_if: false,
                br_table: false,
                call_pre: false,
                call_post: false,
                call_indirect_pre: false,
                call_indirect_post: false,
                block_pre: false,
                block_post: false,
                loop_pre: false,
                loop_post: false,
                select: false,
            }"#}
        )
    }

    #[test]
    fn test_specialized_apply() {
        assert_eq!(
            get_joinpoints(
                r#"
                (aspect
                    (advice apply (func    WasmFunction)
                                  ((a I32) (b F32) (c I64))
                                  ((d F64))
                        >>>GUEST>>>🟢<<<GUEST<<<))
                "#,
            ),
            JoinPoints {
                specialized: [SpecialisedJoinPoint {
                    result_types: [WasmType::F64].into(),
                    argument_types: [WasmType::I32, WasmType::F32, WasmType::I64].into(),
                }]
                .iter()
                .cloned()
                .collect(),
                ..Default::default()
            }
        )
    }

    #[test]
    fn test_generic_apply() {
        assert_eq!(
            get_joinpoints(
                r#"
                (aspect
                    (advice apply (func    WasmFunction)
                                  (args Args)
                                  (ress Results)
                        >>>GUEST>>>🟢<<<GUEST<<<))
                "#,
            ),
            JoinPoints {
                generic: true,
                ..Default::default()
            }
        )
    }

    #[test]
    fn test_if_then_else() {
        assert_eq!(
            get_joinpoints(
                r#"
                (aspect
                    (advice if_then_else (cond Condition)
                        >>>GUEST>>>🟢<<<GUEST<<<))
                "#,
            ),
            JoinPoints {
                if_then_else: true,
                ..Default::default()
            }
        )
    }

    #[test]
    fn test_br_if() {
        assert_eq!(
            get_joinpoints(
                r#"
                (aspect
                    (advice br_if (cond  Condition)
                                  (label Label)
                        >>>GUEST>>>⚪️<<<GUEST<<<))
                "#,
            ),
            JoinPoints {
                br_if: true,
                ..Default::default()
            }
        )
    }

    #[test]
    fn test_br_table() {
        assert_eq!(
            get_joinpoints(
                r#"
                (aspect
                    (advice br_table (target  Target)
                                    (default Default)
                        >>>GUEST>>>🏓<<<GUEST<<<))
                "#,
            ),
            JoinPoints {
                br_table: true,
                ..Default::default()
            }
        )
    }

    #[test]
    fn test_call_pre() {
        assert_eq!(
            get_joinpoints(
                r#"
                (aspect
                    (advice call before (f FunctionIndex)
                        >>>GUEST>>>🧐🏃<<<GUEST<<<))
                "#,
            ),
            JoinPoints {
                call_pre: true,
                ..Default::default()
            }
        )
    }

    #[test]
    fn test_call_post() {
        assert_eq!(
            get_joinpoints(
                r#"
                (aspect
                    (advice call after (f FunctionIndex)
                        >>>GUEST>>>🧐🏃<<<GUEST<<<))
                "#,
            ),
            JoinPoints {
                call_post: true,
                ..Default::default()
            }
        )
    }

    #[test]
    fn test_call_indirect_pre() {
        assert_eq!(
            get_joinpoints(
                r#"
                (aspect
                    (advice call_indirect before (table FunctionTable)
                                                 (index FunctionTableIndex)
                        >>>GUEST>>>🧐🏄<<<GUEST<<<))
                "#,
            ),
            JoinPoints {
                call_indirect_pre: true,
                ..Default::default()
            }
        )
    }

    #[test]
    fn test_call_indirect_post() {
        assert_eq!(
            get_joinpoints(
                r#"
                (aspect
                    (advice call_indirect after (table FunctionTable)
                        >>>GUEST>>>👀🏄<<<GUEST<<<))
                "#,
            ),
            JoinPoints {
                call_indirect_post: true,
                ..Default::default()
            }
        )
    }

    #[test]
    fn test_multiple() {
        assert_eq!(
            get_joinpoints(
                r#"
                (aspect
                    (advice apply (func    WasmFunction)
                                  (args    Args)
                                  (results Results)
                        >>>GUEST>>>🟡<<<GUEST<<<)
                    (advice apply (func    WasmFunction)
                                  ((a I32) (b F32) (c I64))
                                  ((d F64))
                        >>>GUEST>>>🟢<<<GUEST<<<)
                    (advice apply (func    WasmFunction)
                                  (Mut (a I32) (b F32))
                                  (Mut (c I64) (d F64))
                        >>>GUEST>>>🔵<<<GUEST<<<)
                    (advice if_then (cond Condition)
                        >>>GUEST>>>🟠<<<GUEST<<<)
                    (advice if_then_else (cond Condition)
                        >>>GUEST>>>🟣<<<GUEST<<<)
                    (advice br_if (cond  Condition)
                                  (label Label)
                        >>>GUEST>>>⚪️<<<GUEST<<<)
                    (advice br_table (target  Target)
                                     (default Default)
                        >>>GUEST>>>🏓<<<GUEST<<<)
                        (advice block before
                            >>>GUEST>>>🧐🧱<<<GUEST<<<)
                        (advice block after
                            >>>GUEST>>>👀🧱<<<GUEST<<<)
                        (advice loop before
                            >>>GUEST>>>🧐➰<<<GUEST<<<)
                        (advice loop after
                            >>>GUEST>>>👀➰<<<GUEST<<<)
                        (advice select (cond Condition)
                            >>>GUEST>>>🦂<<<GUEST<<<)
                    (advice call before (f FunctionIndex)
                        >>>GUEST>>>🧐🏃<<<GUEST<<<)
                    (advice call after (f FunctionIndex)
                        >>>GUEST>>>👀🏃<<<GUEST<<<)
                    (advice call_indirect before (table FunctionTable)
                                                 (index FunctionTableIndex)
                        >>>GUEST>>>🧐🏄<<<GUEST<<<)
                    (advice call_indirect after (table FunctionTable)
                        >>>GUEST>>>👀🏄<<<GUEST<<<)
                )
                "#,
            ),
            JoinPoints {
                generic: true,
                specialized: [
                    SpecialisedJoinPoint {
                        result_types: [WasmType::F64].into(),
                        argument_types: [WasmType::I32, WasmType::F32, WasmType::I64].into(),
                    },
                    SpecialisedJoinPoint {
                        result_types: [WasmType::I64, WasmType::F64].into(),
                        argument_types: [WasmType::I32, WasmType::F32].into(),
                    },
                ]
                .iter()
                .cloned()
                .collect(),
                if_then: true,
                if_then_else: true,
                br_if: true,
                br_table: true,
                call_pre: true,
                call_post: true,
                call_indirect_pre: true,
                call_indirect_post: true,
                block_post: true,
                block_pre: true,
                loop_post: true,
                loop_pre: true,
                select: true,
            }
        )
    }
}
