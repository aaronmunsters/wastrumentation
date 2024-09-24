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

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum WasmType {
    I32,
    F32,
    I64,
    F64,
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

#[derive(Debug, PartialEq, Eq, Default)]
pub struct AnalysisInterface {
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
    pub drop_trap: Option<WasmExport>,
    pub return_trap: Option<WasmExport>,
    pub const_i32_trap: Option<WasmExport>,
    pub const_f32_trap: Option<WasmExport>,
    pub const_i64_trap: Option<WasmExport>,
    pub const_f64_trap: Option<WasmExport>,
    pub unary_i32_to_i32: Option<WasmExport>,
    pub unary_i64_to_i32: Option<WasmExport>,
    pub unary_i64_to_i64: Option<WasmExport>,
    pub unary_f32_to_f32: Option<WasmExport>,
    pub unary_f64_to_f64: Option<WasmExport>,
    pub unary_f32_to_i32: Option<WasmExport>,
    pub unary_f64_to_i32: Option<WasmExport>,
    pub unary_i32_to_i64: Option<WasmExport>,
    pub unary_f32_to_i64: Option<WasmExport>,
    pub unary_f64_to_i64: Option<WasmExport>,
    pub unary_i32_to_f32: Option<WasmExport>,
    pub unary_i64_to_f32: Option<WasmExport>,
    pub unary_f64_to_f32: Option<WasmExport>,
    pub unary_i32_to_f64: Option<WasmExport>,
    pub unary_i64_to_f64: Option<WasmExport>,
    pub unary_f32_to_f64: Option<WasmExport>,
    pub binary_i32_i32_to_i32: Option<WasmExport>,
    pub binary_i64_i64_to_i32: Option<WasmExport>,
    pub binary_f32_f32_to_i32: Option<WasmExport>,
    pub binary_f64_f64_to_i32: Option<WasmExport>,
    pub binary_i64_i64_to_i64: Option<WasmExport>,
    pub binary_f32_f32_to_f32: Option<WasmExport>,
    pub binary_f64_f64_to_f64: Option<WasmExport>,
    pub memory_size: Option<WasmExport>,
    pub memory_grow: Option<WasmExport>,
    pub local_get_i32: Option<WasmExport>,
    pub local_set_i32: Option<WasmExport>,
    pub local_tee_i32: Option<WasmExport>,
    pub global_get_i32: Option<WasmExport>,
    pub global_set_i32: Option<WasmExport>,
    pub local_get_f32: Option<WasmExport>,
    pub local_set_f32: Option<WasmExport>,
    pub local_tee_f32: Option<WasmExport>,
    pub global_get_f32: Option<WasmExport>,
    pub global_set_f32: Option<WasmExport>,
    pub local_get_i64: Option<WasmExport>,
    pub local_set_i64: Option<WasmExport>,
    pub local_tee_i64: Option<WasmExport>,
    pub global_get_i64: Option<WasmExport>,
    pub global_set_i64: Option<WasmExport>,
    pub local_get_f64: Option<WasmExport>,
    pub local_set_f64: Option<WasmExport>,
    pub local_tee_f64: Option<WasmExport>,
    pub global_get_f64: Option<WasmExport>,
    pub global_set_f64: Option<WasmExport>,
    pub f32_store: Option<WasmExport>,
    pub f64_store: Option<WasmExport>,
    pub i32_store: Option<WasmExport>,
    pub i64_store: Option<WasmExport>,
    pub f32_load: Option<WasmExport>,
    pub f64_load: Option<WasmExport>,
    pub i32_load: Option<WasmExport>,
    pub i64_load: Option<WasmExport>,
}

pub struct ProcessedAnalysis<Language: SourceCodeBound> {
    pub analysis_library: Language::SourceCode,
    pub analysis_interface: AnalysisInterface,
}

type ApplyInterface = (WasmExport, WasmImport);

use crate::{analysis::WasmType::I32, compiler::SourceCodeBound};

impl AnalysisInterface {
    pub fn interface_generic_apply() -> ApplyInterface {
        // The analysis its interface
        // -> EXPORTS a `generic apply`, implemented by the analysis developer
        // -> IMPORTS a `call base`, which the analysis may call into to 'resume' case computation
        (
            WasmExport {
                name: FUNCTION_NAME_GENERIC_APPLY.into(),
                // f_apply, instr_f_idx, argc, resc, sigv, sigtypv
                args: vec![I32, I32, I32, I32, I32, I32],
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

    pub fn interface_if_then() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_SPECIALIZED_IF_THEN.into(),
            // path_kontinuation
            args: vec![I32],
            // path_kontinuation
            results: vec![I32],
        }
    }

    pub fn interface_if_then_else() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_SPECIALIZED_IF_THEN_ELSE.into(),
            // path_kontinuation
            args: vec![I32],
            // path_kontinuation
            results: vec![I32],
        }
    }

    pub fn interface_br_if() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_SPECIALIZED_BR_IF.into(),
            // path_kontinuation, label
            // TODO: is `label` interesting? This value does not change at runtime
            args: vec![I32, I32],
            // path_kontinuation
            results: vec![I32],
        }
    }

    pub fn interface_br_table() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_SPECIALIZED_BR_TABLE.into(),
            // table_target_index, default
            args: vec![I32, I32],
            // table_target_index
            results: vec![I32],
        }
    }

    pub fn interface_call_pre() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_SPECIALIZED_CALL_PRE.into(),
            // function_target
            args: vec![I32],
            // void
            results: vec![],
        }
    }

    pub fn interface_call_post() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_SPECIALIZED_CALL_POST.into(),
            // function_target
            args: vec![I32],
            // void
            results: vec![],
        }
    }

    pub fn interface_call_indirect_pre() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_SPECIALIZED_CALL_INDIRECT_PRE.into(),
            // function_table_index, function_table
            args: vec![I32, I32],
            // function_table_index
            results: vec![I32],
        }
    }

    pub fn interface_call_indirect_post() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_SPECIALIZED_CALL_INDIRECT_POST.into(),
            // function_table
            args: vec![I32],
            // void
            results: vec![],
        }
    }

    pub fn interface_block_pre() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_BLOCK_PRE.into(),
            args: vec![],
            results: vec![],
        }
    }
    pub fn interface_block_post() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_BLOCK_POST.into(),
            args: vec![],
            results: vec![],
        }
    }
    pub fn interface_loop_pre() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_LOOP_PRE.into(),
            args: vec![],
            results: vec![],
        }
    }
    pub fn interface_loop_post() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_LOOP_POST.into(),
            args: vec![],
            results: vec![],
        }
    }
    pub fn interface_select() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_SELECT.into(),
            // condition
            args: vec![I32],
            // kontinuation
            results: vec![I32],
        }
    }
}
