use crate::ast::{
    pest::CallQualifier,
    wasp::{
        AdviceDefinition, ApplyHookSignature, ApplySpe, TrapApply, TrapCall, TrapCallIndirectAfter,
        TrapCallIndirectBefore, TrapSignature,
        WasmType::{self, *},
        WaspRoot,
    },
};

pub const FUNCTION_NAME_CALL_BASE: &str = "call_base";
pub const FUNCTION_NAME_GENERIC_APPLY: &str = "generic_apply";
pub const FUNCTION_NAME_SPECIALIZED_BR_IF: &str = "specialized_br_if";
pub const FUNCTION_NAME_SPECIALIZED_BR_TABLE: &str = "specialized_br_table";
pub const FUNCTION_NAME_SPECIALIZED_CALL_POST: &str = "specialized_call_post";
pub const FUNCTION_NAME_SPECIALIZED_CALL_PRE: &str = "specialized_call_pre";
pub const FUNCTION_NAME_SPECIALIZED_CALL_INDIRECT_POST: &str = "specialized_call_indirect_post";
pub const FUNCTION_NAME_SPECIALIZED_CALL_INDIRECT_PRE: &str = "specialized_call_indirect_pre";
pub const FUNCTION_NAME_SPECIALIZED_IF_THEN: &str = "specialized_if_then_k";
pub const FUNCTION_NAME_SPECIALIZED_IF_THEN_ELSE: &str = "specialized_if_then_else_k";
pub const NAMESPACE_TRANSFORMED_INPUT: &str = "transformed_input";

// TODO: are `inputs` and `outputs` used?
#[derive(Debug, PartialEq, Eq, Default)]
pub struct WaspInterface {
    pub inputs: Vec<WasmImport>,
    pub outputs: Vec<WasmExport>,
    pub generic_interface: Option<(WasmExport, WasmImport)>,
    pub if_then_trap: Option<WasmExport>,
    pub if_then_else_trap: Option<WasmExport>,
    pub br_if_trap: Option<WasmExport>,
    pub br_table_trap: Option<WasmExport>,
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
    fn interface_generic_apply() -> ApplyInterface {
        (
            WasmExport {
                name: FUNCTION_NAME_GENERIC_APPLY.into(),
                // f_apply, argc, resc, sigv, sigtypv
                args: vec![I32, I32, I32, I32, I32],
                results: vec![],
            },
            WasmImport {
                namespace: NAMESPACE_TRANSFORMED_INPUT.into(),
                name: FUNCTION_NAME_CALL_BASE.into(),
                // f_apply, sigv
                args: vec![I32, I32],
                results: vec![],
            },
        )
    }

    fn interface_if_then() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_SPECIALIZED_IF_THEN.into(),
            // path_kontinuation
            args: vec![I32],
            // path_kontinuation
            results: vec![I32],
        }
    }

    fn interface_if_then_else() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_SPECIALIZED_IF_THEN_ELSE.into(),
            // path_kontinuation
            args: vec![I32],
            // path_kontinuation
            results: vec![I32],
        }
    }

    fn interface_br_if() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_SPECIALIZED_BR_IF.into(),
            // path_kontinuation, label
            // TODO: is `label` interesting? This value does not change at runtime
            args: vec![I32, I32],
            // path_kontinuation
            results: vec![I32],
        }
    }

    fn interface_br_table() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_SPECIALIZED_BR_TABLE.into(),
            // table_target_index, default
            args: vec![I32, I32],
            // table_target_index
            results: vec![I32],
        }
    }

    fn interface_call_pre() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_SPECIALIZED_CALL_PRE.into(),
            // function_target
            args: vec![I32],
            // void
            results: vec![],
        }
    }

    fn interface_call_post() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_SPECIALIZED_CALL_POST.into(),
            // function_target
            args: vec![I32],
            // void
            results: vec![],
        }
    }

    fn interface_call_indirect_pre() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_SPECIALIZED_CALL_INDIRECT_PRE.into(),
            // function_table_index, function_table
            args: vec![I32, I32],
            // void
            results: vec![I32],
        }
    }

    fn interface_call_indirect_post() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_SPECIALIZED_CALL_INDIRECT_POST.into(),
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
        let mut br_table_trap = None;
        let WaspRoot(advice_definitions) = wasp_root;
        for advice_definition in advice_definitions {
            if let AdviceDefinition::AdviceTrap(trap_signature) = advice_definition {
                match trap_signature {
                    TrapSignature::TrapApply(TrapApply {
                        apply_hook_signature: ApplyHookSignature::Gen(_),
                        ..
                    }) => generic_interface = Some(WaspInterface::interface_generic_apply()),
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
                        if_then_trap = Some(WaspInterface::interface_if_then())
                    }
                    TrapSignature::TrapIfThenElse(_) => {
                        if_then_else_trap = Some(WaspInterface::interface_if_then_else())
                    }
                    TrapSignature::TrapBrIf(_) => {
                        br_if_trap = Some(WaspInterface::interface_br_if())
                    }
                    TrapSignature::TrapBrTable(_) => {
                        br_table_trap = Some(WaspInterface::interface_br_table())
                    }
                    TrapSignature::TrapCall(TrapCall {
                        call_qualifier: CallQualifier::Before,
                        ..
                    }) => pre_trap_call = Some(WaspInterface::interface_call_pre()),
                    TrapSignature::TrapCall(TrapCall {
                        call_qualifier: CallQualifier::After,
                        ..
                    }) => post_trap_call = Some(WaspInterface::interface_call_post()),
                    TrapSignature::TrapCallIndirectBefore(TrapCallIndirectBefore { .. }) => {
                        pre_trap_call_indirect = Some(WaspInterface::interface_call_indirect_pre())
                    }
                    TrapSignature::TrapCallIndirectAfter(TrapCallIndirectAfter { .. }) => {
                        post_trap_call_indirect =
                            Some(WaspInterface::interface_call_indirect_post())
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
            br_table_trap,
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
                    parameter_function: "WasmFunc".into(),
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
                generic_interface: Some(WaspInterface::interface_generic_apply()),
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
                if_then_trap: Some(WaspInterface::interface_if_then()),
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
                if_then_else_trap: Some(WaspInterface::interface_if_then_else()),
                ..Default::default()
            }
        );
    }
}
