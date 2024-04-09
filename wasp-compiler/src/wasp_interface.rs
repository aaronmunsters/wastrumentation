use crate::ast::{
    pest::CallQualifier,
    wasp::{
        AdviceDefinition, ApplyHookSignature, ApplySpe, TrapApply, TrapCall, TrapCallIndirect,
        TrapSignature,
        WasmType::{self, *},
        WaspRoot,
    },
};

// TODO: order such that uniqueness is last
pub const TRANSFORMED_INPUT_NS: &str = "transformed_input";
pub const GENERIC_APPLY_FUNCTION_NAME: &str = "generic_apply";
pub const SPECIALIZED_IF_THEN_FUNCTION_NAME: &str = "specialized_if_then_k";
pub const SPECIALIZED_IF_THEN_ELSE_FUNCTION_NAME: &str = "specialized_if_then_else_k";
pub const SPECIALIZED_BR_IF_FUNCTION_NAME: &str = "specialized_br_if";
pub const SPECIALIZED_CALL_PRE_FUNCTION_NAME: &str = "specialized_call_pre";
pub const SPECIALIZED_CALL_POST_FUNCTION_NAME: &str = "specialized_call_post";
pub const SPECIALIZED_CALL_INDIRECT_PRE_FUNCTION_NAME: &str = "specialized_call_indirect_pre";
pub const SPECIALIZED_CALL_INDIRECT_POST_FUNCTION_NAME: &str = "specialized_call_indirect_post";
pub const CALL_BASE: &str = "call_base";

// TODO: are `inputs` and `outputs` used?
#[derive(Debug, PartialEq, Eq, Default)]
pub struct WaspInterface {
    pub inputs: Vec<WasmImport>,
    pub outputs: Vec<WasmExport>,
    pub generic_interface: Option<(WasmExport, WasmImport)>,
    pub if_then_trap: Option<WasmExport>,
    pub if_then_else_trap: Option<WasmExport>,
    pub br_if_trap: Option<WasmExport>,
    pub pre_trap_call: Option<WasmExport>,
    pub pre_trap_call_indirect: Option<WasmExport>,
    pub post_trap_call: Option<WasmExport>,
    pub post_trap_call_indirect: Option<WasmExport>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct WasmImport {
    pub namespace: String,
    pub name: String,
    pub args: Vec<WasmType>,
    pub results: Vec<WasmType>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct WasmExport {
    pub name: String,
    pub args: Vec<WasmType>,
    pub results: Vec<WasmType>,
}

type ApplyInterface = (WasmExport, WasmImport);

impl WaspInterface {
    // TODO: naming - move uniqueness to end
    fn generic_apply_interface() -> ApplyInterface {
        (
            WasmExport {
                name: GENERIC_APPLY_FUNCTION_NAME.into(),
                // f_apply, argc, resc, sigv, sigtypv
                args: vec![I32, I32, I32, I32, I32],
                results: vec![],
            },
            WasmImport {
                namespace: TRANSFORMED_INPUT_NS.into(),
                name: CALL_BASE.into(),
                // f_apply, sigv
                args: vec![I32, I32],
                results: vec![],
            },
        )
    }

    fn if_then_interface() -> WasmExport {
        WasmExport {
            name: SPECIALIZED_IF_THEN_FUNCTION_NAME.into(),
            // path_kontinuation
            args: vec![I32],
            // path_kontinuation
            results: vec![I32],
        }
    }

    fn if_then_else_interface() -> WasmExport {
        WasmExport {
            name: SPECIALIZED_IF_THEN_ELSE_FUNCTION_NAME.into(),
            // path_kontinuation
            args: vec![I32],
            // path_kontinuation
            results: vec![I32],
        }
    }

    fn br_if_interface() -> WasmExport {
        WasmExport {
            name: SPECIALIZED_BR_IF_FUNCTION_NAME.into(),
            // path_kontinuation, label
            // TODO: is `label` interesting? This value does not change at runtime
            args: vec![I32, I32],
            // path_kontinuation
            results: vec![I32],
        }
    }

    fn call_pre_interface() -> WasmExport {
        WasmExport {
            name: SPECIALIZED_CALL_PRE_FUNCTION_NAME.into(),
            // function_target
            args: vec![I32],
            // void
            results: vec![],
        }
    }

    fn call_post_interface() -> WasmExport {
        WasmExport {
            name: SPECIALIZED_CALL_POST_FUNCTION_NAME.into(),
            // function_target
            args: vec![I32],
            // void
            results: vec![],
        }
    }

    fn call_indirect_pre_interface() -> WasmExport {
        WasmExport {
            name: SPECIALIZED_CALL_INDIRECT_PRE_FUNCTION_NAME.into(),
            // function_table_index, function_table
            args: vec![I32, I32],
            // void
            results: vec![I32],
        }
    }

    fn call_indirect_post_interface() -> WasmExport {
        WasmExport {
            name: SPECIALIZED_CALL_INDIRECT_POST_FUNCTION_NAME.into(),
            // function_table
            args: vec![I32],
            // void
            results: vec![],
        }
    }
}

impl From<&WaspRoot> for WaspInterface {
    fn from(wasp_root: &WaspRoot) -> Self {
        let mut generic_interface = None;
        let mut wasm_imports: Vec<WasmImport> = Vec::new();
        let mut wasm_exports: Vec<WasmExport> = Vec::new();
        let mut if_then_trap: Option<WasmExport> = None;
        let mut if_then_else_trap: Option<WasmExport> = None;
        let mut br_if_trap: Option<WasmExport> = None;
        let mut pre_trap_call: Option<WasmExport> = None;
        let mut pre_trap_call_indirect: Option<WasmExport> = None;
        let mut post_trap_call: Option<WasmExport> = None;
        let mut post_trap_call_indirect: Option<WasmExport> = None;
        let WaspRoot(advice_definitions) = wasp_root;
        for advice_definition in advice_definitions {
            if let AdviceDefinition::AdviceTrap(trap_signature) = advice_definition {
                match trap_signature {
                    TrapSignature::TrapApply(TrapApply {
                        apply_hook_signature: ApplyHookSignature::Gen(_),
                        ..
                    }) => generic_interface = Some(WaspInterface::generic_apply_interface()),
                    TrapSignature::TrapApply(TrapApply {
                        apply_hook_signature:
                            ApplyHookSignature::Spe(ApplySpe {
                                mutable_signature,
                                parameters_arguments,
                                parameters_results,
                                ..
                            }),
                        ..
                    }) => {
                        wasm_imports.push(WasmImport::for_extern_call_base(
                            *mutable_signature,
                            parameters_arguments,
                            parameters_results,
                        ));
                        wasm_exports.push(WasmExport::for_exported_apply_trap(
                            *mutable_signature,
                            parameters_arguments,
                            parameters_results,
                        ));
                    }
                    TrapSignature::TrapIfThen(_) => {
                        if_then_trap = Some(WaspInterface::if_then_interface())
                    }
                    TrapSignature::TrapIfThenElse(_) => {
                        if_then_else_trap = Some(WaspInterface::if_then_else_interface())
                    }
                    TrapSignature::TrapBrIf(_) => {
                        br_if_trap = Some(WaspInterface::br_if_interface())
                    }
                    TrapSignature::TrapCall(TrapCall {
                        call_qualifier: CallQualifier::Before,
                        ..
                    }) => pre_trap_call = Some(WaspInterface::call_pre_interface()),
                    TrapSignature::TrapCall(TrapCall {
                        call_qualifier: CallQualifier::After,
                        ..
                    }) => post_trap_call = Some(WaspInterface::call_post_interface()),
                    TrapSignature::TrapCallIndirect(TrapCallIndirect {
                        call_qualifier: CallQualifier::Before,
                        ..
                    }) => {
                        pre_trap_call_indirect = Some(WaspInterface::call_indirect_pre_interface())
                    }
                    TrapSignature::TrapCallIndirect(TrapCallIndirect {
                        call_qualifier: CallQualifier::After,
                        ..
                    }) => {
                        post_trap_call_indirect =
                            Some(WaspInterface::call_indirect_post_interface())
                    }
                }
            };
        }
        Self {
            inputs: wasm_imports,
            outputs: wasm_exports,
            generic_interface,
            if_then_trap,
            if_then_else_trap,
            br_if_trap,
            pre_trap_call,
            pre_trap_call_indirect,
            post_trap_call,
            post_trap_call_indirect,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::wasp::{
        ApplyGen, BranchFormalCondition, GenericTarget, TrapIfThen, TrapIfThenElse, WasmParameter,
    };

    use super::*;

    #[test]
    fn test_debug() {
        let wasm_import = WasmImport {
            namespace: "namespace".into(),
            name: "name".into(),
            args: vec![WasmType::I32],
            results: vec![WasmType::F32],
        };
        assert_eq!(
            format!("{wasm_import:?}"),
            r#"WasmImport { namespace: "namespace", name: "name", args: [I32], results: [F32] }"#
        );

        let wasm_import = WasmExport {
            name: "name".into(),
            args: vec![WasmType::I32],
            results: vec![WasmType::F32],
        };
        assert_eq!(
            format!("{wasm_import:?}"),
            r#"WasmExport { name: "name", args: [I32], results: [F32] }"#
        );
    }

    #[test]
    fn test_generation_empty() {
        // empty wasp root generates empty interface
        let wasp_root: WaspRoot = WaspRoot(vec![]);
        let wasp_interface = WaspInterface::from(&wasp_root);
        assert_eq!(wasp_interface, WaspInterface::default());
    }

    #[test]
    fn test_generation_global_only() {
        let wasp_root: WaspRoot = WaspRoot(vec![AdviceDefinition::AdviceGlobal(
            "global functionality".into(),
        )]);
        let wasp_interface = WaspInterface::from(&wasp_root);
        assert_eq!(wasp_interface, WaspInterface::default());
    }

    #[test]
    fn test_generation_generic() {
        let wasp_root = WaspRoot(vec![AdviceDefinition::AdviceTrap(
            TrapSignature::TrapApply(TrapApply {
                apply_hook_signature: ApplyHookSignature::Gen(ApplyGen {
                    generic_means: GenericTarget::Dynamic,
                    parameter_apply: "WasmFunc".into(),
                    parameter_arguments: "WasmArgs".into(),
                    parameter_results: "WasmResults".into(),
                }),
                body: "trap body".into(),
            }),
        )]);
        let wasp_interface = WaspInterface::from(&wasp_root);

        assert_eq!(
            wasp_interface,
            WaspInterface {
                generic_interface: Some(WaspInterface::generic_apply_interface()),
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_generation_specialized() {
        let wasp_root = WaspRoot(vec![AdviceDefinition::AdviceTrap(
            TrapSignature::TrapApply(TrapApply {
                apply_hook_signature: ApplyHookSignature::Spe(ApplySpe {
                    mutable_signature: true,
                    apply_parameter: "WasmFunc".into(),
                    parameters_arguments: vec![WasmParameter {
                        identifier: "a".into(),
                        identifier_type: WasmType::I32,
                    }],
                    parameters_results: vec![WasmParameter {
                        identifier: "b".into(),
                        identifier_type: WasmType::F32,
                    }],
                }),
                body: "trap body".into(),
            }),
        )]);
        let wasp_interface = WaspInterface::from(&wasp_root);

        assert_eq!(
            wasp_interface,
            WaspInterface {
                inputs: vec![WasmImport {
                    namespace: "transformed_input".into(),
                    name: "call_base_mut_args_i32_ress_f32".into(),
                    args: vec![I32],
                    results: vec![F32]
                }],
                outputs: vec![WasmExport {
                    name: "apply_func_mut_args_i32_ress_f32".into(),
                    args: vec![I32],
                    results: vec![F32]
                }],
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_generation_if_then() {
        let wasp_root = WaspRoot(vec![AdviceDefinition::AdviceTrap(
            TrapSignature::TrapIfThen(TrapIfThen {
                branch_formal_condition: BranchFormalCondition("condition".into()),
                body: "trap body".into(),
            }),
        )]);
        let wasp_interface = WaspInterface::from(&wasp_root);

        assert_eq!(
            wasp_interface,
            WaspInterface {
                if_then_trap: Some(WaspInterface::if_then_interface()),
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_generation_if_then_else() {
        let wasp_root = WaspRoot(vec![AdviceDefinition::AdviceTrap(
            TrapSignature::TrapIfThenElse(TrapIfThenElse {
                branch_formal_condition: BranchFormalCondition("condition".into()),
                body: "trap body".into(),
            }),
        )]);
        let wasp_interface = WaspInterface::from(&wasp_root);

        assert_eq!(
            wasp_interface,
            WaspInterface {
                if_then_else_trap: Some(WaspInterface::if_then_else_interface()),
                ..Default::default()
            }
        );
    }
}
