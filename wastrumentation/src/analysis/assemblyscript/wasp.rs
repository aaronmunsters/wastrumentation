use wasp_compiler::ast::{
    pest::CallQualifier,
    wasp::{
        AdviceDefinition, ApplyHookSignature, ApplySpe, Root, TrapApply, TrapBlockAfter,
        TrapBlockBefore, TrapCall, TrapCallIndirectAfter, TrapCallIndirectBefore, TrapLoopAfter,
        TrapLoopBefore, TrapSelect, TrapSignature,
    },
};

use super::super::AnalysisInterface;

impl From<&Root> for AnalysisInterface {
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
                            Some(AnalysisInterface::interface_generic_apply());
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
                        wasp_interface.if_then_trap = Some(AnalysisInterface::interface_if_then());
                    }
                    TrapSignature::TrapIfThenElse(_) => {
                        wasp_interface.if_then_else_trap =
                            Some(AnalysisInterface::interface_if_then_else());
                    }
                    TrapSignature::TrapBrIf(_) => {
                        wasp_interface.br_if_trap = Some(AnalysisInterface::interface_br_if());
                    }
                    TrapSignature::TrapBrTable(_) => {
                        wasp_interface.br_table_trap =
                            Some(AnalysisInterface::interface_br_table());
                    }
                    TrapSignature::TrapCall(TrapCall {
                        call_qualifier: CallQualifier::Before,
                        ..
                    }) => {
                        wasp_interface.pre_trap_call = Some(AnalysisInterface::interface_call_pre())
                    }
                    TrapSignature::TrapCall(TrapCall {
                        call_qualifier: CallQualifier::After,
                        ..
                    }) => {
                        wasp_interface.post_trap_call =
                            Some(AnalysisInterface::interface_call_post());
                    }
                    TrapSignature::TrapCallIndirectBefore(TrapCallIndirectBefore { .. }) => {
                        wasp_interface.pre_trap_call_indirect =
                            Some(AnalysisInterface::interface_call_indirect_pre());
                    }
                    TrapSignature::TrapCallIndirectAfter(TrapCallIndirectAfter { .. }) => {
                        wasp_interface.post_trap_call_indirect =
                            Some(AnalysisInterface::interface_call_indirect_post());
                    }
                    TrapSignature::TrapBlockBefore(TrapBlockBefore { .. }) => {
                        wasp_interface.pre_block = Some(AnalysisInterface::interface_block_pre());
                    }
                    TrapSignature::TrapBlockAfter(TrapBlockAfter { .. }) => {
                        wasp_interface.post_block = Some(AnalysisInterface::interface_block_post());
                    }
                    TrapSignature::TrapLoopBefore(TrapLoopBefore { .. }) => {
                        wasp_interface.pre_loop = Some(AnalysisInterface::interface_loop_pre());
                    }
                    TrapSignature::TrapLoopAfter(TrapLoopAfter { .. }) => {
                        wasp_interface.post_loop = Some(AnalysisInterface::interface_loop_post());
                    }
                    TrapSignature::TrapSelect(TrapSelect { .. }) => {
                        wasp_interface.select = Some(AnalysisInterface::interface_select());
                    }
                }
            };
        }
        wasp_interface
    }
}

#[cfg(test)]
mod tests {
    use wasp_compiler::ast::wasp::{
        ApplyGen, BranchFormalCondition, GenericTarget, TrapIfThen, TrapIfThenElse,
    };

    use crate::analysis::{WasmExport, WasmImport, WasmType};

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
        let wasp_interface = AnalysisInterface::from(&wasp_root);
        assert_eq!(wasp_interface, AnalysisInterface::default());
    }

    #[test]
    fn test_generation_global_only() {
        let wasp_root: Root = Root(vec![AdviceDefinition::AdviceGlobal(
            "global functionality".into(),
        )]);
        let wasp_interface = AnalysisInterface::from(&wasp_root);
        assert_eq!(wasp_interface, AnalysisInterface::default());
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
        let wasp_interface = AnalysisInterface::from(&wasp_root);

        assert_eq!(
            wasp_interface,
            AnalysisInterface {
                generic_interface: Some(AnalysisInterface::interface_generic_apply()),
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
        let wasp_interface = AnalysisInterface::from(&wasp_root);

        assert_eq!(
            wasp_interface,
            AnalysisInterface {
                if_then_trap: Some(AnalysisInterface::interface_if_then()),
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
        let wasp_interface = AnalysisInterface::from(&wasp_root);

        assert_eq!(
            wasp_interface,
            AnalysisInterface {
                if_then_else_trap: Some(AnalysisInterface::interface_if_then_else()),
                ..Default::default()
            }
        );
    }
}
