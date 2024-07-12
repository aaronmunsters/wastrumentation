use wasp_compiler::{
    ast::{
        assemblyscript::AssemblyScriptProgram,
        pest::CallQualifier,
        wasp::{
            AdviceDefinition, ApplyHookSignature, ApplySpe, Root, TrapApply, TrapBlockAfter,
            TrapBlockBefore, TrapCall, TrapCallIndirectAfter, TrapCallIndirectBefore,
            TrapLoopAfter, TrapLoopBefore, TrapSelect, TrapSignature,
        },
    },
    compile as wasp_compile,
    wasp_interface::WaspInterface,
    CompilationResult as WaspCompilationResult,
};
use wastrumentation_instr_lib::std_lib_compile::assemblyscript::compiler_options::CompilerOptions as AssemblyscriptCompilerOptions;

use super::AnalysisInterface;

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
