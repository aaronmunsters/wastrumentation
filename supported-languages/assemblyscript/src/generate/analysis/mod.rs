use std::fmt::{Debug, Display};

use indoc::writedoc;

mod util;
pub mod wasp;

use util::Alphabetical;

use crate::compile::AssemblyScript;
use crate::generate::analysis::wasp::WaspRoot;
use wasp_compiler::CompilationResult as WaspCompilerResult;
use wastrumentation::analysis::{AnalysisInterface, ProcessedAnalysis};

use wasp_compiler::ast::wasp::{
    AdviceDefinition, ApplyGen, ApplyHookSignature, ApplySpe, BranchFormalCondition,
    BranchFormalDefault, BranchFormalLabel, BranchFormalTarget, FormalIndex, FormalTable,
    FormalTarget, Root, SelectFormalCondition, TrapApply, TrapBlockPost, TrapBlockPre, TrapBrIf,
    TrapBrTable, TrapCall, TrapCallIndirectPost, TrapCallIndirectPre, TrapIfThen, TrapIfThenElse,
    TrapLoopPost, TrapLoopPre, TrapSelect, TrapSignature, WasmParameter, WasmType,
};
use wasp_compiler::wasp_interface::{WasmExport, WasmImport};
use wastrumentation::analysis::{
    FUNCTION_NAME_GENERIC_APPLY, FUNCTION_NAME_SELECT, FUNCTION_NAME_SPECIALIZED_BR_IF,
    FUNCTION_NAME_SPECIALIZED_BR_TABLE, FUNCTION_NAME_SPECIALIZED_CALL_INDIRECT_POST,
    FUNCTION_NAME_SPECIALIZED_CALL_INDIRECT_PRE, FUNCTION_NAME_SPECIALIZED_CALL_POST,
    FUNCTION_NAME_SPECIALIZED_CALL_PRE, FUNCTION_NAME_SPECIALIZED_IF_THEN,
    FUNCTION_NAME_SPECIALIZED_IF_THEN_ELSE, NAMESPACE_TRANSFORMED_INPUT, TRAP_NAME_POST_BLOCK,
    TRAP_NAME_POST_LOOP, TRAP_NAME_PRE_BLOCK, TRAP_NAME_PRE_LOOP,
};

const STD_ANALYSIS_LIB_GENRIC_APPLY: &str = include_str!("std_analysis_lib_gen_apply.ts");
const STD_ANALYSIS_LIB_IF: &str = include_str!("std_analysis_lib_if.ts");
const STD_ANALYSIS_LIB_CALL: &str = include_str!("std_analysis_lib_call.ts");

#[derive(Clone)]
pub struct WaspAnalysisSpec {
    pub wasp_source: String,
}

impl TryInto<ProcessedAnalysis<AssemblyScript>> for &WaspAnalysisSpec {
    type Error = wasp_compiler::Error;

    fn try_into(self) -> std::result::Result<ProcessedAnalysis<AssemblyScript>, Self::Error> {
        let WaspCompilerResult {
            wasp_root,
            join_points: _,
        } = wasp_compiler::compile(&self.wasp_source)?;

        let wasp_root = WaspRoot(wasp_root);
        let analysis_interface = AnalysisInterface::from(&wasp_root);

        let WaspRoot(wasp_root) = wasp_root; // FIXME: ugly pattern of taking it out again
        let as_root = ASRoot(wasp_root);
        let AssemblyScriptProgram { content } = as_root.into();

        Ok(ProcessedAnalysis {
            analysis_interface,
            analysis_library: content,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct AssemblyScriptProgram {
    pub content: String,
}

pub struct ASRoot(pub Root);
impl From<ASRoot> for AssemblyScriptProgram {
    fn from(root: ASRoot) -> Self {
        let ASRoot(wasp_root) = root;
        let mut program_analysis_content = String::new();

        if wasp_root.instruments_generic_apply() {
            program_analysis_content.push_str(STD_ANALYSIS_LIB_GENRIC_APPLY);
        };

        if wasp_root.instruments_if() {
            program_analysis_content.push_str(STD_ANALYSIS_LIB_IF);
        };

        if wasp_root.instruments_call() {
            program_analysis_content.push_str(STD_ANALYSIS_LIB_CALL);
        }

        let Root(advice_definitions) = wasp_root;
        for advice_definition in advice_definitions {
            let as_advice_definition = ASAdviceDefinition(advice_definition);
            program_analysis_content.push_str(&as_advice_definition.to_string());
        }

        AssemblyScriptProgram {
            content: program_analysis_content,
        }
    }
}

struct ASAdviceDefinition(AdviceDefinition);
impl Display for ASAdviceDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(advice_definition) = self;
        match advice_definition {
            AdviceDefinition::AdviceGlobal(program) => write!(f, "{program}"),
            AdviceDefinition::AdviceTrap(trap) => ASTrapSignature(trap).fmt(f),
        }
    }
}

struct ASTrapSignature<'a>(&'a TrapSignature);
impl Display for ASTrapSignature<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(trap_signature) = self;
        match trap_signature {
            TrapSignature::TrapApply(trap_apply) => ASTrapApply(trap_apply).fmt(f),
            TrapSignature::TrapIfThen(trap_if_then) => ASTrapIfThen(trap_if_then).fmt(f),
            TrapSignature::TrapIfThenElse(trap_if_then_else) => {
                ASTrapIfThenElse(trap_if_then_else).fmt(f)
            }
            TrapSignature::TrapBrIf(trap_br_if) => ASTrapBrIf(trap_br_if).fmt(f),
            TrapSignature::TrapBrTable(trap_br_table) => ASTrapBrTable(trap_br_table).fmt(f),
            TrapSignature::TrapCall(trap_call) => ASTrapCall(trap_call).fmt(f),
            TrapSignature::TrapCallIndirectPre(trap_call_indirect_pre) => {
                ASTrapCallIndirectPre(trap_call_indirect_pre).fmt(f)
            }
            TrapSignature::TrapCallIndirectPost(trap_call_indirect_post) => {
                ASTrapCallIndirectPost(trap_call_indirect_post).fmt(f)
            }
            TrapSignature::TrapBlockPre(trap_block_pre) => ASTrapBlockPre(trap_block_pre).fmt(f),
            TrapSignature::TrapBlockPost(trap_block_post) => {
                ASTrapBlockPost(trap_block_post).fmt(f)
            }
            TrapSignature::TrapLoopPre(trap_loop_pre) => ASTrapLoopPre(trap_loop_pre).fmt(f),
            TrapSignature::TrapLoopPost(trap_loop_post) => ASTrapLoopPost(trap_loop_post).fmt(f),
            TrapSignature::TrapSelect(trap_select) => ASTrapSelect(trap_select).fmt(f),
        }
    }
}

struct ASTrapApply<'a>(&'a TrapApply);
impl Display for ASTrapApply<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(TrapApply {
            apply_hook_signature,
            body,
        }) = self;

        match apply_hook_signature {
            ApplyHookSignature::Gen(apply_gen) => ASApplyGen { apply_gen, body }.fmt(f),
            ApplyHookSignature::Spe(apply_spe) => ASApplySpe { apply_spe, body }.fmt(f),
        }
    }
}

struct ASApplyGen<'a> {
    apply_gen: &'a ApplyGen,
    body: &'a str,
}
impl Display for ASApplyGen<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            apply_gen:
                ApplyGen {
                    generic_means: _, // TODO: put to use?
                    parameter_function,
                    parameter_arguments,
                    parameter_results,
                },
            body,
        } = self;
        indoc::writedoc!(
            f,
            r"
            export function {GENERIC_APPLY_FUNCTION_NAME}(
                f_apply: i32,
                instr_f_idx: i32,
                argc: i32,
                resc: i32,
                sigv: i32,
                sigtypv: i32,
                code_present_serialized: i32,
            ): void {{
                let {parameter_function} = new WasmFunction(f_apply, instr_f_idx, sigv);
                let argsResults = new MutDynArgsResults(
                    argc,
                    resc,
                    sigv,
                    sigtypv,
                );
                let {parameter_arguments} = new MutDynArgs(argsResults);
                let {parameter_results} = new MutDynRess(argsResults);
                {body}
            }}
            ",
            GENERIC_APPLY_FUNCTION_NAME = FUNCTION_NAME_GENERIC_APPLY,
        )
    }
}

fn signature_to_string(
    mutable_signature: bool,
    parameters_arguments: &Vec<WasmParameter>,
    parameters_results: &Vec<WasmParameter>,
) -> String {
    let mutable_prefix = if mutable_signature { "mut_" } else { "" };
    let wasm_pars_to_types_string = |arguments: &Vec<WasmParameter>| {
        let arguments_types: Vec<String> = arguments
            .iter()
            .map(|argument| argument.identifier_type.to_string())
            .collect();
        arguments_types.join("_")
    };

    let argument_types_string = wasm_pars_to_types_string(parameters_arguments);
    let arguments_types: &str = argument_types_string.as_str();

    let result_types_string = wasm_pars_to_types_string(parameters_results);
    let results_types: &str = result_types_string.as_str();

    format!("{mutable_prefix}args_{arguments_types}_ress_{results_types}")
}

struct ASWasmImport(WasmImport);

impl ASWasmImport {
    #[must_use]
    pub fn for_extern_call_base(
        mutable_signature: bool,
        parameters_arguments: &Vec<WasmParameter>,
        parameters_results: &Vec<WasmParameter>,
    ) -> Self {
        let full_signature_name =
            signature_to_string(mutable_signature, parameters_arguments, parameters_results);
        let name = format!("call_base_{full_signature_name}");

        Self(WasmImport {
            namespace: NAMESPACE_TRANSFORMED_INPUT.into(),
            name,
            args: WasmParameterVec(parameters_arguments).types(),
            results: WasmParameterVec(parameters_results).types(),
        })
    }
}

impl Display for ASWasmImport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(WasmImport {
            namespace,
            name,
            args,
            results,
        }) = self;
        let args_signature = args
            .iter()
            .enumerate()
            .map(|(index, wasm_type)| format!("{}: {wasm_type}", index.to_alphabetic()))
            .collect::<Vec<String>>()
            .join(", ");

        let ress_signature = results
            .iter()
            .map(WasmType::to_string)
            .collect::<Vec<String>>()
            .join(", ");
        indoc::writedoc! { f, r#"
            @external("{namespace}", "{name}")
            declare function {name}({args_signature}): {ress_signature};
            "#
        }
    }
}

struct ASWasmExport(WasmExport);
impl ASWasmExport {
    #[must_use]
    pub fn for_exported_apply_trap(
        mutable_signature: bool,
        parameters_arguments: &Vec<WasmParameter>,
        parameters_results: &Vec<WasmParameter>,
    ) -> Self {
        let full_signature_name =
            signature_to_string(mutable_signature, parameters_arguments, parameters_results);
        let name = format!("apply_func_{full_signature_name}");
        Self(WasmExport {
            name,
            args: WasmParameterVec(parameters_arguments).types(),
            results: WasmParameterVec(parameters_results).types(),
        })
    }
}

// TODO: move this to elsewhere? is this tied to AssemblyScript?
struct WasmParameterVec<'a>(&'a [WasmParameter]);
impl WasmParameterVec<'_> {
    fn types(&self) -> Vec<WasmType> {
        let Self(vec) = self;
        vec.iter()
            .map(|wasm_parameter| wasm_parameter.identifier_type)
            .collect()
    }
}

struct ASApplySpe<'a> {
    apply_spe: &'a ApplySpe,
    body: &'a str,
}

impl ASApplySpe<'_> {
    fn to_assemblyscript_args_signature(&self) -> String {
        let parameters: Vec<String> = self
            .apply_spe
            .parameters_arguments
            .iter()
            .map(|a| format!("{}: {}", a.identifier, a.identifier_type))
            .collect();

        parameters.join(", ")
    }

    fn to_assemblyscript_ress_signature(&self) -> String {
        let results: Vec<String> = self
            .apply_spe
            .parameters_results
            .iter()
            .map(|a| a.identifier_type.to_string())
            .collect();

        results.join(", ")
    }
}

impl Display for ASApplySpe<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            apply_spe:
                ApplySpe {
                    mutable_signature,
                    parameters_arguments,
                    parameters_results,
                    ..
                },
            body,
        } = self;

        let ref as_wasm_import @ ASWasmImport(ref import) = ASWasmImport::for_extern_call_base(
            *mutable_signature,
            parameters_arguments,
            parameters_results,
        );
        let external_call_base_name = &import.name;

        let ASWasmExport(export) = ASWasmExport::for_exported_apply_trap(
            *mutable_signature,
            parameters_arguments,
            parameters_results,
        );

        let args_signature = self.to_assemblyscript_args_signature();
        let ress_signature = self.to_assemblyscript_ress_signature();

        let exported_func_inst_name = export.name;

        writedoc!(
            f,
            "
            {import_declaration}
            export function {exported_func_inst_name}({args_signature}): {ress_signature} {{
                let func = {external_call_base_name};
                {{
                    {body}
                }}
            }}
            ",
            import_declaration = as_wasm_import
        )
    }
}

// // TODO: mangle the variables that are hardcoded now.
// // eg. currently, there's `path_kontinuation` as a variable, however
// //     this may clash with analysis-provided variables.
// //     The same goes for `low_level_label`.

struct ASTrapIfThen<'a>(&'a TrapIfThen);
impl Display for ASTrapIfThen<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(TrapIfThen {
            branch_formal_condition: BranchFormalCondition(parameter_condition),
            body,
        }) = &self;

        writedoc!(
            f,
            "
            export function {FUNCTION_NAME_SPECIALIZED_IF_THEN}(
                path_kontinuation: i32,
                if_then_input_c: i32,
                if_then_arity: i32,
                func_index: i64,
                istr_index: i64,
            ): i32 {{
                let {parameter_condition} = new ParameterIfThenCondition(path_kontinuation);
                {body}
                // Fallback, if no return value
                return path_kontinuation;
            }}
            "
        )
    }
}

struct ASTrapIfThenElse<'a>(&'a TrapIfThenElse);
impl Display for ASTrapIfThenElse<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(TrapIfThenElse {
            branch_formal_condition: BranchFormalCondition(parameter_condition),
            body,
        }) = &self;

        writedoc!(
            f,
            "
            export function {FUNCTION_NAME_SPECIALIZED_IF_THEN_ELSE}(
                path_kontinuation: i32,
                if_then_else_input_c: i32,
                if_then_else_arity: i32,
                func_index: i64,
                istr_index: i64,
            ): i32 {{
                let {parameter_condition} = new ParameterIfThenElseCondition(path_kontinuation);
                {body}
                // Fallback, if no return value
                return path_kontinuation;
            }}
            "
        )
    }
}

struct ASTrapBrIf<'a>(&'a TrapBrIf);
impl Display for ASTrapBrIf<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(TrapBrIf {
            branch_formal_condition: BranchFormalCondition(parameter_condition),
            branch_formal_label: BranchFormalLabel(parameter_label),
            body,
        }) = &self;

        writedoc!(
            f,
            "
            export function {FUNCTION_NAME_SPECIALIZED_BR_IF}(
                path_kontinuation: i32,
                low_level_label: i32,
                func_index: i64,
                istr_index: i64,
            ): i32 {{
                let {parameter_condition} = new ParameterBrIfCondition(path_kontinuation);
                let {parameter_label} = new ParameterBrIfLabel(low_level_label);
                {body}
                // Fallback, if no return value
                return path_kontinuation;
            }}
            "
        )
    }
}
struct ASTrapBrTable<'a>(&'a TrapBrTable);
impl Display for ASTrapBrTable<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(TrapBrTable {
            branch_formal_target: BranchFormalTarget(parameter_target),
            branch_formal_default: BranchFormalDefault(parameter_default),
            body,
        }) = &self;

        writedoc!(
            f,
            "
            export function {FUNCTION_NAME_SPECIALIZED_BR_TABLE}(
                br_table_target: i32,
                effective_label: i32,
                br_table_default: i32,
                func_index: i64,
                istr_index: i64,
            ): i32 {{
                let {parameter_target} = new ParameterBrTableTarget(br_table_target);
                let {parameter_default} = new ParameterBrTableDefault(br_table_default);
                {body}
                // Fallback, if no return value
                return br_table_target;
            }}
            "
        )
    }
}

struct ASTrapCall<'a>(&'a TrapCall);
impl Display for ASTrapCall<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(TrapCall {
            call_qualifier,
            formal_target: FormalTarget(parameter_target),
            body,
        }) = &self;

        let specialized_name = match call_qualifier {
            wasp_compiler::ast::pest::CallQualifier::Pre => FUNCTION_NAME_SPECIALIZED_CALL_PRE,
            wasp_compiler::ast::pest::CallQualifier::Post => FUNCTION_NAME_SPECIALIZED_CALL_POST,
        };

        writedoc!(
            f,
            "
            export function {specialized_name}(
                function_target: i32,
                func_index: i64,
                istr_index: i64,
            ): void {{
                let {parameter_target} = new FunctionIndex(function_target);
                {body}
            }}
            "
        )
    }
}

struct ASTrapCallIndirectPre<'a>(&'a TrapCallIndirectPre);
impl Display for ASTrapCallIndirectPre<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(TrapCallIndirectPre {
            formal_table: FormalTable(parameter_table),
            formal_index: FormalIndex(parameter_index),
            body,
        }) = &self;

        writedoc!(
            f,
            "
            export function {FUNCTION_NAME_SPECIALIZED_CALL_INDIRECT_PRE}(
                function_table_index: i32, // NOTE: index first, eases transformation!
                function_table: i32,
                func_index: i64,
                istr_index: i64,
            ): i32 {{
                let {parameter_table} = new FunctionTable(function_table);
                let {parameter_index} = new FunctionTableIndex(function_table_index);
                {body}
                // Fallback, if no return value
                return function_table_index;
            }}
            "
        )
    }
}

struct ASTrapCallIndirectPost<'a>(&'a TrapCallIndirectPost);
impl Display for ASTrapCallIndirectPost<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(TrapCallIndirectPost {
            formal_table: FormalTable(parameter_table),
            body,
        }) = &self;
        writedoc!(
            f,
            "
            export function {FUNCTION_NAME_SPECIALIZED_CALL_INDIRECT_POST}(
                function_table: i32,
                func_index: i64,
                istr_index: i64,
            ): void {{
                let {parameter_table} = new FunctionTable(function_table);
                {body}
            }}
            "
        )
    }
}

struct ASTrapBlockPre<'a>(&'a TrapBlockPre);
impl Display for ASTrapBlockPre<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(TrapBlockPre { body }) = &self;
        writedoc!(
            f,
            "
            export function {FUNCTION_NAME_BLOCK_PRE}(
                input_count: i32,
                arity: i32,
                func_index: i64,
                istr_index: i64,
            ): void {{
                {body}
            }}
            ",
            FUNCTION_NAME_BLOCK_PRE = TRAP_NAME_PRE_BLOCK,
        )
    }
}

struct ASTrapBlockPost<'a>(&'a TrapBlockPost);
impl Display for ASTrapBlockPost<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(TrapBlockPost { body }) = &self;
        writedoc!(
            f,
            "
            export function {FUNCTION_NAME_BLOCK_POST}(
                func_index: i64,
                istr_index: i64,
            ): void {{
                {body}
            }}
            ",
            FUNCTION_NAME_BLOCK_POST = TRAP_NAME_POST_BLOCK,
        )
    }
}

struct ASTrapLoopPre<'a>(&'a TrapLoopPre);
impl Display for ASTrapLoopPre<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(TrapLoopPre { body }) = &self;
        writedoc!(
            f,
            "
            export function {FUNCTION_NAME_LOOP_PRE}(
                input_count: i32,
                arity: i32,
                func_index: i64,
                istr_index: i64,
            ): void {{
                {body}
            }}
            ",
            FUNCTION_NAME_LOOP_PRE = TRAP_NAME_PRE_LOOP,
        )
    }
}

struct ASTrapLoopPost<'a>(&'a TrapLoopPost);
impl Display for ASTrapLoopPost<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(TrapLoopPost { body }) = &self;
        writedoc!(
            f,
            "
            export function {FUNCTION_NAME_LOOP_POST}(
                func_index: i64,
                istr_index: i64,
            ): void {{
                {body}
            }}
            ",
            FUNCTION_NAME_LOOP_POST = TRAP_NAME_POST_LOOP,
        )
    }
}

struct ASTrapSelect<'a>(&'a TrapSelect);
impl Display for ASTrapSelect<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(TrapSelect {
            body,
            select_formal_condition: SelectFormalCondition(select_formal_condition),
        }) = &self;
        write!(
            f,
            "
            export function {FUNCTION_NAME_SELECT}(
                path_kontinuation: i32,
                func_index: i64,
                istr_index: i64,
            ): i32 {{
                let {select_formal_condition} = new ParameterSelectCondition(path_kontinuation);
                {body}
                // Fallback, if no return value
                return path_kontinuation;
            }}
            "
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use wasp_compiler::ast::wasp::{GenericTarget, WasmParameter, WasmType};

    #[test]
    fn generate_apply_spe_mut() {
        let ast: TrapSignature = TrapSignature::TrapApply(TrapApply {
            apply_hook_signature: ApplyHookSignature::Spe(ApplySpe {
                mutable_signature: true,
                apply_parameter: "func".into(),
                parameters_arguments: vec![
                    WasmParameter {
                        identifier: "a".into(),
                        identifier_type: WasmType::I32,
                    },
                    WasmParameter {
                        identifier: "b".into(),
                        identifier_type: WasmType::F32,
                    },
                    WasmParameter {
                        identifier: "c".into(),
                        identifier_type: WasmType::I64,
                    },
                ],
                parameters_results: vec![WasmParameter {
                    identifier: "r".into(),
                    identifier_type: WasmType::F64,
                }],
            }),
            body: "console.log(a); return func(a);".into(),
        });

        assert_eq!(
            ASTrapSignature(&ast).to_string(),
            indoc! { r#"
                @external("transformed_input", "call_base_mut_args_i32_f32_i64_ress_f64")
                declare function call_base_mut_args_i32_f32_i64_ress_f64(a: i32, b: f32, c: i64): f64;

                export function apply_func_mut_args_i32_f32_i64_ress_f64(a: i32, b: f32, c: i64): f64 {
                    let func = call_base_mut_args_i32_f32_i64_ress_f64;
                    {
                        console.log(a); return func(a);
                    }
                }
                "# }
        )
    }

    #[test]
    fn generate_apply_spe_imut() {
        let ast: TrapSignature = TrapSignature::TrapApply(TrapApply {
            apply_hook_signature: ApplyHookSignature::Spe(ApplySpe {
                mutable_signature: false,
                apply_parameter: "func".into(),
                parameters_arguments: vec![
                    WasmParameter {
                        identifier: "a".into(),
                        identifier_type: WasmType::I32,
                    },
                    WasmParameter {
                        identifier: "b".into(),
                        identifier_type: WasmType::F32,
                    },
                    WasmParameter {
                        identifier: "c".into(),
                        identifier_type: WasmType::I64,
                    },
                ],
                parameters_results: vec![WasmParameter {
                    identifier: "r".into(),
                    identifier_type: WasmType::F64,
                }],
            }),
            body: "console.log(a); return func(a);".into(),
        });

        assert_eq!(
            ASTrapSignature(&ast).to_string(),
            indoc! { r#"
                @external("transformed_input", "call_base_args_i32_f32_i64_ress_f64")
                declare function call_base_args_i32_f32_i64_ress_f64(a: i32, b: f32, c: i64): f64;

                export function apply_func_args_i32_f32_i64_ress_f64(a: i32, b: f32, c: i64): f64 {
                    let func = call_base_args_i32_f32_i64_ress_f64;
                    {
                        console.log(a); return func(a);
                    }
                }
                "# }
        )
    }

    #[test]
    fn generate_apply_gen() {
        let ast: TrapSignature = TrapSignature::TrapApply(TrapApply {
            apply_hook_signature: ApplyHookSignature::Gen(ApplyGen {
                generic_means: GenericTarget::MutableDynamic,
                parameter_function: "func".to_string(),
                parameter_arguments: "args".to_string(),
                parameter_results: "results".to_string(),
            }),
            body: "console.log(args.get<i32>(0)); func.apply();".into(),
        });

        let expected = indoc! { r"
            export function generic_apply(
                f_apply: i32,
                instr_f_idx: i32,
                argc: i32,
                resc: i32,
                sigv: i32,
                sigtypv: i32,
                code_present_serialized: i32,
            ): void {
                let func = new WasmFunction(f_apply, instr_f_idx, sigv);
                let argsResults = new MutDynArgsResults(
                    argc,
                    resc,
                    sigv,
                    sigtypv,
                );
                let args = new MutDynArgs(argsResults);
                let results = new MutDynRess(argsResults);
                console.log(args.get<i32>(0)); func.apply();
            }
            " };

        assert_eq!(ASTrapSignature(&ast).to_string(), expected);
    }

    #[test]
    fn generate_if_then() {
        let ast: TrapSignature = TrapSignature::TrapIfThen(TrapIfThen {
            branch_formal_condition: BranchFormalCondition("cond".into()),
            body: "console.log('it');".into(),
        });

        let expected = indoc! { r"
        export function specialized_if_then_k(
            path_kontinuation: i32,
            if_then_input_c: i32,
            if_then_arity: i32,
            func_index: i64,
            istr_index: i64,
        ): i32 {
            let cond = new ParameterIfThenCondition(path_kontinuation);
            console.log('it');
            // Fallback, if no return value
            return path_kontinuation;
        }
        " };

        assert_eq!(ASTrapSignature(&ast).to_string(), expected);
    }

    #[test]
    fn generate_if_then_else() {
        let ast: TrapSignature = TrapSignature::TrapIfThenElse(TrapIfThenElse {
            branch_formal_condition: BranchFormalCondition("cond".into()),
            body: "console.log('ite');".into(),
        });

        let expected = indoc! { r"
        export function specialized_if_then_else_k(
            path_kontinuation: i32,
            if_then_else_input_c: i32,
            if_then_else_arity: i32,
            func_index: i64,
            istr_index: i64,
        ): i32 {
            let cond = new ParameterIfThenElseCondition(path_kontinuation);
            console.log('ite');
            // Fallback, if no return value
            return path_kontinuation;
        }
        " };

        assert_eq!(ASTrapSignature(&ast).to_string(), expected);
    }

    // #[test]
    // fn from_input_program() {
    //     let input_program = indoc! { r#"
    //         (aspect
    //             (global >>>GUEST>>>console.log("Hello world!");
    //             <<<GUEST<<<)
    //             (advice apply (func    WasmFunction)
    //                           (args    MutDynArgs)
    //                           (results MutDynResults) >>>GUEST>>>console.log(args.get<i32>(0)); func.apply();<<<GUEST<<<)
    //             (advice if_then_else (cond Condition) >>>GUEST>>>console.log('ite');<<<GUEST<<<))"# };
    //     let assemblyscript_program = AssemblyScriptProgram::try_from(input_program).unwrap();
    //     let expected_outcome = format!(
    //         "{}{}{}",
    //         STD_ANALYSIS_LIB_GENRIC_APPLY,
    //         STD_ANALYSIS_LIB_IF,
    //         indoc! { r#"
    //         console.log("Hello world!");
    //             export function generic_apply(
    //             f_apply: i32,
    //             instr_f_idx: i32,
    //             argc: i32,
    //             resc: i32,
    //             sigv: i32,
    //             sigtypv: i32,
    //         ): void {
    //             let func = new WasmFunction(f_apply, instr_f_idx, sigv);
    //             let argsResults = new MutDynArgsResults(
    //                 argc,
    //                 resc,
    //                 sigv,
    //                 sigtypv,
    //             );
    //             let args = new MutDynArgs(argsResults);
    //             let results = new MutDynRess(argsResults);
    //             console.log(args.get<i32>(0)); func.apply();
    //         }
    //         export function specialized_if_then_else_k(
    //             path_kontinuation: i32,
    //         ): i32 {
    //             let cond = new ParameterIfThenElseCondition(path_kontinuation);
    //             console.log('ite');
    //             // Fallback, if no return value
    //             return path_kontinuation;
    //         }
    //         "# }
    //     );

    //     assert_eq!(assemblyscript_program.content, expected_outcome);
    // }

    #[test]
    fn should_debug() {
        let assemblyscript_program = AssemblyScriptProgram {
            content: "console.log(43)".to_string(),
        };
        assert_eq!(
            format!("{assemblyscript_program:?}"),
            r#"AssemblyScriptProgram { content: "console.log(43)" }"#
        )
    }

    // #[test]
    // fn parse_fail_pest() {
    //     assert!(AssemblyScriptProgram::try_from("")
    //         .unwrap_err()
    //         .to_string()
    //         .as_str()
    //         .contains("expected wasp"))
    // }

    // #[test]
    // fn assemblyscript_conversion_fail() {
    //     assert_eq!(
    //         AssemblyScriptProgram::try_from(
    //             "
    //             (aspect
    //                 (advice apply (a WasmFunction)
    //                               (a Args)
    //                               (a Results) >>>GUEST>>>
    //                     1;
    //                 <<<GUEST<<<))
    //             "
    //         )
    //         .unwrap_err()
    //         .to_string()
    //         .as_str(),
    //         "Parameters must be unique, got: a, a, a."
    //     )
    // }
}
