use crate::ast::{
    pest::CallQualifier,
    wasp::{
        AdviceDefinition, ApplyHookSignature, ApplySpe, Root, TrapApply, TrapBlockAfter,
        TrapBlockBefore, TrapCall, TrapCallIndirectAfter, TrapCallIndirectBefore, TrapLoopAfter,
        TrapLoopBefore, TrapSelect, TrapSignature,
        WasmType::{self, I32},
    },
};

pub const FUNCTION_NAME_BLOCK_PRE: &str = "block_pre";
pub const FUNCTION_NAME_BLOCK_POST: &str = "block_post";
pub const FUNCTION_NAME_CALL_BASE: &str = "call_base";
pub const FUNCTION_NAME_GENERIC_APPLY: &str = "generic_apply";
pub const FUNCTION_NAME_LOOP_PRE: &str = "loop_pre";
pub const FUNCTION_NAME_LOOP_POST: &str = "loop_post";
pub const FUNCTION_NAME_SELECT: &str = "specialized_select";
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
    pub generic_interface: Option<(WasmExport, WasmImport)>,
    pub if_then_trap: Option<WasmExport>,
    pub if_then_else_trap: Option<WasmExport>,
    pub br_if_trap: Option<WasmExport>,
    pub br_table_trap: Option<WasmExport>,
    pub pre_trap_call: Option<WasmExport>,
    pub pre_trap_call_indirect: Option<WasmExport>,
    pub post_trap_call: Option<WasmExport>,
    pub post_trap_call_indirect: Option<WasmExport>,
    pub pre_block: Option<WasmExport>,
    pub post_block: Option<WasmExport>,
    pub pre_loop: Option<WasmExport>,
    pub post_loop: Option<WasmExport>,
    pub select: Option<WasmExport>,
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

    fn interface_block_pre() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_BLOCK_PRE.into(),
            args: vec![],
            results: vec![],
        }
    }
    fn interface_block_post() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_BLOCK_POST.into(),
            args: vec![],
            results: vec![],
        }
    }
    fn interface_loop_pre() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_LOOP_PRE.into(),
            args: vec![],
            results: vec![],
        }
    }
    fn interface_loop_post() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_LOOP_POST.into(),
            args: vec![],
            results: vec![],
        }
    }
    fn interface_select() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_SELECT.into(),
            // condition
            args: vec![I32],
            // kontinuation
            results: vec![I32],
        }
    }
}

impl From<&Root> for WaspInterface {
    fn from(wasp_root: &Root) -> Self {
        let mut wasp_interface = Self::default();
        let Root(advice_definitions) = wasp_root;
        for advice_definition in advice_definitions {
            if let AdviceDefinition::AdviceTrap(trap_signature) = advice_definition {
                match trap_signature {
                    TrapSignature::TrapApply(TrapApply {
                        apply_hook_signature: ApplyHookSignature::Gen(_),
                        ..
                    }) => {
                        wasp_interface.generic_interface =
                            Some(WaspInterface::interface_generic_apply());
                    }
                    TrapSignature::TrapApply(TrapApply {
                        apply_hook_signature:
                            ApplyHookSignature::Spe(ApplySpe {
                                mutable_signature: _,
                                parameters_arguments: _,
                                parameters_results: _,
                                ..
                            }),
                        ..
                    }) => todo!(),
                    TrapSignature::TrapIfThen(_) => {
                        wasp_interface.if_then_trap = Some(WaspInterface::interface_if_then());
                    }
                    TrapSignature::TrapIfThenElse(_) => {
                        wasp_interface.if_then_else_trap =
                            Some(WaspInterface::interface_if_then_else());
                    }
                    TrapSignature::TrapBrIf(_) => {
                        wasp_interface.br_if_trap = Some(WaspInterface::interface_br_if());
                    }
                    TrapSignature::TrapBrTable(_) => {
                        wasp_interface.br_table_trap = Some(WaspInterface::interface_br_table());
                    }
                    TrapSignature::TrapCall(TrapCall {
                        call_qualifier: CallQualifier::Before,
                        ..
                    }) => wasp_interface.pre_trap_call = Some(WaspInterface::interface_call_pre()),
                    TrapSignature::TrapCall(TrapCall {
                        call_qualifier: CallQualifier::After,
                        ..
                    }) => {
                        wasp_interface.post_trap_call = Some(WaspInterface::interface_call_post());
                    }
                    TrapSignature::TrapCallIndirectBefore(TrapCallIndirectBefore { .. }) => {
                        wasp_interface.pre_trap_call_indirect =
                            Some(WaspInterface::interface_call_indirect_pre());
                    }
                    TrapSignature::TrapCallIndirectAfter(TrapCallIndirectAfter { .. }) => {
                        wasp_interface.post_trap_call_indirect =
                            Some(WaspInterface::interface_call_indirect_post());
                    }
                    TrapSignature::TrapBlockBefore(TrapBlockBefore { .. }) => {
                        wasp_interface.pre_block = Some(WaspInterface::interface_block_pre());
                    }
                    TrapSignature::TrapBlockAfter(TrapBlockAfter { .. }) => {
                        wasp_interface.post_block = Some(WaspInterface::interface_block_post());
                    }
                    TrapSignature::TrapLoopBefore(TrapLoopBefore { .. }) => {
                        wasp_interface.pre_loop = Some(WaspInterface::interface_loop_pre());
                    }
                    TrapSignature::TrapLoopAfter(TrapLoopAfter { .. }) => {
                        wasp_interface.post_loop = Some(WaspInterface::interface_loop_post());
                    }
                    TrapSignature::TrapSelect(TrapSelect { .. }) => {
                        wasp_interface.select = Some(WaspInterface::interface_select());
                    }
                }
            };
        }
        wasp_interface
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::wasp::{
        ApplyGen, BranchFormalCondition, GenericTarget, TrapIfThen, TrapIfThenElse,
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
        let wasp_root: Root = Root(vec![]);
        let wasp_interface = WaspInterface::from(&wasp_root);
        assert_eq!(wasp_interface, WaspInterface::default());
    }

    #[test]
    fn test_generation_global_only() {
        let wasp_root: Root = Root(vec![AdviceDefinition::AdviceGlobal(
            "global functionality".into(),
        )]);
        let wasp_interface = WaspInterface::from(&wasp_root);
        assert_eq!(wasp_interface, WaspInterface::default());
    }

    #[test]
    fn test_generation_generic() {
        let wasp_root = Root(vec![AdviceDefinition::AdviceTrap(
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
    fn test_generation_if_then() {
        let wasp_root = Root(vec![AdviceDefinition::AdviceTrap(
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
        let wasp_root = Root(vec![AdviceDefinition::AdviceTrap(
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
