use std::collections::HashSet;

use crate::ast::wasp::{
    ApplyHookSignature, ApplySpe, TrapSignature, WasmParameter, WasmType, WaspRoot,
};

#[derive(Debug, PartialEq, Eq, Default)]
pub struct JoinPoints {
    pub generic: bool,
    pub specialized: HashSet<SpecialisedJoinPoint>,
    pub if_then_else: bool,
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
            JoinPoint::IfThenElse => self.if_then_else = true,
        };
    }
}

enum JoinPoint {
    Generic,
    Specialised(SpecialisedJoinPoint),
    IfThenElse,
}

impl WaspRoot {
    pub fn join_points(&self) -> JoinPoints {
        let Self(advice_definitions) = self;
        let mut join_points = JoinPoints::default();
        for advice_definition in advice_definitions {
            match advice_definition {
                crate::ast::wasp::AdviceDefinition::AdviceGlobal(_) => {}
                crate::ast::wasp::AdviceDefinition::AdviceTrap(trap_signature) => {
                    join_points.include(trap_signature.join_point())
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
            TrapSignature::TrapIfThenElse(_) => JoinPoint::IfThenElse,
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
    use pest::Parser;

    use super::*;
    use crate::ast::pest::{Rule, WaspInput, WaspParser};

    fn get_joinpoints(wasp: &str) -> JoinPoints {
        let mut pest_parse = WaspParser::parse(Rule::wasp_input, wasp).unwrap();
        let wasp_input = WaspInput::from_pest(&mut pest_parse).expect("pest to input");
        let wasp_root = WaspRoot::try_from(wasp_input).unwrap();
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
            format!("{joinpoints:?}"),
            "\
        JoinPoints { \
            generic: false, \
            specialized: {\
                SpecialisedJoinPoint { \
                    result_types: [F64], \
                    argument_types: [I32, F32, I64] \
                }\
            }, \
            if_then_else: false \
        }"
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
                generic: false,
                specialized: [SpecialisedJoinPoint {
                    result_types: [WasmType::F64].into(),
                    argument_types: [WasmType::I32, WasmType::F32, WasmType::I64].into(),
                }]
                .iter()
                .cloned()
                .collect(),
                if_then_else: false,
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
                specialized: HashSet::new(),
                if_then_else: false,
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
                generic: false,
                specialized: HashSet::new(),
                if_then_else: true,
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
                    (advice if_then_else (cond Condition)
                        >>>GUEST>>>🟣<<<GUEST<<<)
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
                if_then_else: true,
            }
        )
    }
}
