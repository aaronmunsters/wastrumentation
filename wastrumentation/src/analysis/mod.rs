pub const FUNCTION_NAME_BLOCK_PRE: &str = "block_pre";
pub const FUNCTION_NAME_BLOCK_POST: &str = "block_post";
pub const FUNCTION_NAME_CALL_BASE: &str = "call_base";
pub const FUNCTION_NAME_GENERIC_APPLY: &str = "generic_apply";
pub const FUNCTION_NAME_LOOP_PRE: &str = "loop_pre";
pub const FUNCTION_NAME_LOOP_POST: &str = "loop_post";
pub const FUNCTION_NAME_SELECT: &str = "specialized_select";
pub const FUNCTION_NAME_SPECIALIZED_BR: &str = "specialized_br";
pub const FUNCTION_NAME_SPECIALIZED_BR_IF: &str = "specialized_br_if";
pub const FUNCTION_NAME_SPECIALIZED_BR_TABLE: &str = "specialized_br_table";
pub const FUNCTION_NAME_SPECIALIZED_CALL_POST: &str = "specialized_call_post";
pub const FUNCTION_NAME_SPECIALIZED_CALL_PRE: &str = "specialized_call_pre";
pub const FUNCTION_NAME_SPECIALIZED_CALL_INDIRECT_POST: &str = "specialized_call_indirect_post";
pub const FUNCTION_NAME_SPECIALIZED_CALL_INDIRECT_PRE: &str = "specialized_call_indirect_pre";
pub const FUNCTION_NAME_SPECIALIZED_IF_THEN: &str = "specialized_if_then_k";
pub const FUNCTION_NAME_SPECIALIZED_IF_THEN_ELSE: &str = "specialized_if_then_else_k";
pub const NAMESPACE_TRANSFORMED_INPUT: &str = "transformed_input";

pub const TRAP_NAME_UNARY_I32_TO_I32: &str = "unary_i32_to_i32";
pub const TRAP_NAME_UNARY_I64_TO_I32: &str = "unary_i64_to_i32";
pub const TRAP_NAME_UNARY_I64_TO_I64: &str = "unary_i64_to_i64";
pub const TRAP_NAME_UNARY_F32_TO_F32: &str = "unary_f32_to_f32";
pub const TRAP_NAME_UNARY_F64_TO_F64: &str = "unary_f64_to_f64";
pub const TRAP_NAME_UNARY_F32_TO_I32: &str = "unary_f32_to_i32";
pub const TRAP_NAME_UNARY_F64_TO_I32: &str = "unary_f64_to_i32";
pub const TRAP_NAME_UNARY_I32_TO_I64: &str = "unary_i32_to_i64";
pub const TRAP_NAME_UNARY_F32_TO_I64: &str = "unary_f32_to_i64";
pub const TRAP_NAME_UNARY_F64_TO_I64: &str = "unary_f64_to_i64";
pub const TRAP_NAME_UNARY_I32_TO_F32: &str = "unary_i32_to_f32";
pub const TRAP_NAME_UNARY_I64_TO_F32: &str = "unary_i64_to_f32";
pub const TRAP_NAME_UNARY_F64_TO_F32: &str = "unary_f64_to_f32";
pub const TRAP_NAME_UNARY_I32_TO_F64: &str = "unary_i32_to_f64";
pub const TRAP_NAME_UNARY_I64_TO_F64: &str = "unary_i64_to_f64";
pub const TRAP_NAME_UNARY_F32_TO_F64: &str = "unary_f32_to_f64";

pub const TRAP_NAME_BINARY_I32_I32_TO_I32: &str = "binary_i32_i32_to_i32";
pub const TRAP_NAME_BINARY_I64_I64_TO_I32: &str = "binary_i64_i64_to_i32";
pub const TRAP_NAME_BINARY_F32_F32_TO_I32: &str = "binary_f32_f32_to_i32";
pub const TRAP_NAME_BINARY_F64_F64_TO_I32: &str = "binary_f64_f64_to_i32";
pub const TRAP_NAME_BINARY_I64_I64_TO_I64: &str = "binary_i64_i64_to_i64";
pub const TRAP_NAME_BINARY_F32_F32_TO_F32: &str = "binary_f32_f32_to_f32";
pub const TRAP_NAME_BINARY_F64_F64_TO_F64: &str = "binary_f64_f64_to_f64";

pub const TRAP_NAME_DROP: &str = "drop_trap";
pub const TRAP_NAME_RETURN: &str = "return_trap";

pub const TRAP_CONST_I32: &str = "trap_const_i32";
pub const TRAP_CONST_F32: &str = "trap_const_f32";
pub const TRAP_CONST_I64: &str = "trap_const_i64";
pub const TRAP_CONST_F64: &str = "trap_const_f64";

pub const TRAP_NAME_LOCAL_GET_I32: &str = "trap_local_get_i32";
pub const TRAP_NAME_LOCAL_SET_I32: &str = "trap_local_set_i32";
pub const TRAP_NAME_LOCAL_TEE_I32: &str = "trap_local_tee_i32";
pub const TRAP_NAME_LOCAL_GET_F32: &str = "trap_local_get_f32";
pub const TRAP_NAME_LOCAL_SET_F32: &str = "trap_local_set_f32";
pub const TRAP_NAME_LOCAL_TEE_F32: &str = "trap_local_tee_f32";
pub const TRAP_NAME_LOCAL_GET_I64: &str = "trap_local_get_i64";
pub const TRAP_NAME_LOCAL_SET_I64: &str = "trap_local_set_i64";
pub const TRAP_NAME_LOCAL_TEE_I64: &str = "trap_local_tee_i64";
pub const TRAP_NAME_LOCAL_GET_F64: &str = "trap_local_get_f64";
pub const TRAP_NAME_LOCAL_SET_F64: &str = "trap_local_set_f64";
pub const TRAP_NAME_LOCAL_TEE_F64: &str = "trap_local_tee_f64";
pub const TRAP_NAME_GLOBAL_GET_I32: &str = "trap_global_get_i32";
pub const TRAP_NAME_GLOBAL_SET_I32: &str = "trap_global_set_i32";
pub const TRAP_NAME_GLOBAL_GET_F32: &str = "trap_global_get_f32";
pub const TRAP_NAME_GLOBAL_SET_F32: &str = "trap_global_set_f32";
pub const TRAP_NAME_GLOBAL_GET_I64: &str = "trap_global_get_i64";
pub const TRAP_NAME_GLOBAL_SET_I64: &str = "trap_global_set_i64";
pub const TRAP_NAME_GLOBAL_GET_F64: &str = "trap_global_get_f64";
pub const TRAP_NAME_GLOBAL_SET_F64: &str = "trap_global_set_f64";

pub const TRAP_NAME_F32_STORE: &str = "trap_f32_store";
pub const TRAP_NAME_F64_STORE: &str = "trap_f64_store";
pub const TRAP_NAME_I32_STORE: &str = "trap_i32_store";
pub const TRAP_NAME_I64_STORE: &str = "trap_i64_store";
pub const TRAP_NAME_F32_LOAD: &str = "trap_f32_load";
pub const TRAP_NAME_F64_LOAD: &str = "trap_f64_load";
pub const TRAP_NAME_I32_LOAD: &str = "trap_i32_load";
pub const TRAP_NAME_I64_LOAD: &str = "trap_i64_load";

pub const TRAP_NAME_MEMORY_SIZE: &str = "trap_memory_size";
pub const TRAP_NAME_MEMORY_GROW: &str = "trap_memory_grow";

const SER_OPRTR_TYP: WasmType = I32;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum WasmType {
    I32,
    F32,
    I64,
    F64,
}

use crate::{
    analysis::WasmType::{F32, F64, I32, I64},
    compiler::SourceCodeBound,
};

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
    pub br_trap: Option<WasmExport>,
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
}

macro_rules! simple_interfaces {
    ($($interface_call_name:ident $trap_name:ident : $($args:expr)* => $($results:expr)*),* $(,)?) => {
        $(
            simple_interface!($interface_call_name $trap_name : $($args)* => $($results)*);
        )*
    };
}

macro_rules! simple_interface {
    ($interface_call_name:ident $trap_name:ident : $($args:expr)* => $($results:expr)*) => {
        impl AnalysisInterface {
            pub fn $interface_call_name() -> WasmExport {
                WasmExport {
                    name: $trap_name.into(),
                    args: vec![$($args),*],
                    results: vec![$($results),*],
                }
            }
        }
    };
}

simple_interfaces! {
    interface_if_then               FUNCTION_NAME_SPECIALIZED_IF_THEN            :                                                       /*cndt:*/ I32 =>           /*cont:*/ I32,
    interface_if_then_else          FUNCTION_NAME_SPECIALIZED_IF_THEN_ELSE       :                                                       /*cndt:*/ I32 =>           /*cont:*/ I32,
    interface_br                    FUNCTION_NAME_SPECIALIZED_BR                 :                                                        /*lbl:*/ I64 =>                /*void*/,
    interface_br_if                 FUNCTION_NAME_SPECIALIZED_BR_IF              :                                          /*cndt:*/ I32 /*lbl:*/ I32 =>           /*cont:*/ I32,
    interface_br_table              FUNCTION_NAME_SPECIALIZED_BR_TABLE           :                           /*br_tbl_tgt_idx:*/ I32 /*dflt_idx:*/ I32 => /*br_tbl_tgt_idx:*/ I32,
    interface_call_pre              FUNCTION_NAME_SPECIALIZED_CALL_PRE           :                                                      /*f_tgt:*/ I32 =>                /*void*/,
    interface_call_post             FUNCTION_NAME_SPECIALIZED_CALL_POST          :                                                      /*f_tgt:*/ I32 =>                /*void*/,
    interface_call_indirect_pre     FUNCTION_NAME_SPECIALIZED_CALL_INDIRECT_PRE  :                                 /*fn_tbl_idx:*/ I32 /*fn_tbl:*/ I32 =>    /*fn_tbl_idx:*/ I32 ,
    interface_call_indirect_post    FUNCTION_NAME_SPECIALIZED_CALL_INDIRECT_POST :                                                     /*fn_tbl:*/ I32 =>                /*void*/,
    interface_block_pre             FUNCTION_NAME_BLOCK_PRE                      :                                                            /*void*/ =>                /*void*/,
    interface_block_post            FUNCTION_NAME_BLOCK_POST                     :                                                            /*void*/ =>                /*void*/,
    interface_loop_pre              FUNCTION_NAME_LOOP_PRE                       :                                                            /*void*/ =>                /*void*/,
    interface_loop_post             FUNCTION_NAME_LOOP_POST                      :                                                            /*void*/ =>                /*void*/,
    interface_select                FUNCTION_NAME_SELECT                         :                                                       /*cndt:*/ I32 =>           /*cont:*/ I32,
    interface_return                TRAP_NAME_RETURN                             :                                                            /*void*/ =>                /*void*/,
    interface_drop                  TRAP_NAME_DROP                               :                                                            /*void*/ =>                /*void*/,
    interface_const_i32             TRAP_CONST_I32                               :                                                      /*const:*/ I32 =>            /*res:*/ I32,
    interface_const_f32             TRAP_CONST_F32                               :                                                      /*const:*/ F32 =>            /*res:*/ F32,
    interface_const_i64             TRAP_CONST_I64                               :                                                      /*const:*/ I64 =>            /*res:*/ I64,
    interface_const_f64             TRAP_CONST_F64                               :                                                      /*const:*/ F64 =>            /*res:*/ F64,
    interface_binary_i32_i32_to_i32 TRAP_NAME_BINARY_I32_I32_TO_I32              :              /*lopnd:*/ I32 /*ropnd:*/ I32 /*oprtr:*/ SER_OPRTR_TYP =>            /*res:*/ I32,
    interface_binary_i64_i64_to_i32 TRAP_NAME_BINARY_I64_I64_TO_I32              :              /*lopnd:*/ I64 /*ropnd:*/ I64 /*oprtr:*/ SER_OPRTR_TYP =>            /*res:*/ I32,
    interface_binary_f32_f32_to_i32 TRAP_NAME_BINARY_F32_F32_TO_I32              :              /*lopnd:*/ F32 /*ropnd:*/ F32 /*oprtr:*/ SER_OPRTR_TYP =>            /*res:*/ I32,
    interface_binary_f64_f64_to_i32 TRAP_NAME_BINARY_F64_F64_TO_I32              :              /*lopnd:*/ F64 /*ropnd:*/ F64 /*oprtr:*/ SER_OPRTR_TYP =>            /*res:*/ I32,
    interface_binary_i64_i64_to_i64 TRAP_NAME_BINARY_I64_I64_TO_I64              :              /*lopnd:*/ I64 /*ropnd:*/ I64 /*oprtr:*/ SER_OPRTR_TYP =>            /*res:*/ I64,
    interface_binary_f32_f32_to_f32 TRAP_NAME_BINARY_F32_F32_TO_F32              :              /*lopnd:*/ F32 /*ropnd:*/ F32 /*oprtr:*/ SER_OPRTR_TYP =>            /*res:*/ F32,
    interface_binary_f64_f64_to_f64 TRAP_NAME_BINARY_F64_F64_TO_F64              :              /*lopnd:*/ F64 /*ropnd:*/ F64 /*oprtr:*/ SER_OPRTR_TYP =>            /*res:*/ F64,
    interface_unary_i32_to_i32      TRAP_NAME_UNARY_I32_TO_I32                   :                              /*opnd:*/ I32 /*oprtr:*/ SER_OPRTR_TYP =>            /*res:*/ I32,
    interface_unary_i64_to_i32      TRAP_NAME_UNARY_I64_TO_I32                   :                              /*opnd:*/ I64 /*oprtr:*/ SER_OPRTR_TYP =>            /*res:*/ I32,
    interface_unary_i64_to_i64      TRAP_NAME_UNARY_I64_TO_I64                   :                              /*opnd:*/ I64 /*oprtr:*/ SER_OPRTR_TYP =>            /*res:*/ I64,
    interface_unary_f32_to_f32      TRAP_NAME_UNARY_F32_TO_F32                   :                              /*opnd:*/ F32 /*oprtr:*/ SER_OPRTR_TYP =>            /*res:*/ F32,
    interface_unary_f64_to_f64      TRAP_NAME_UNARY_F64_TO_F64                   :                              /*opnd:*/ F64 /*oprtr:*/ SER_OPRTR_TYP =>            /*res:*/ F64,
    interface_unary_f32_to_i32      TRAP_NAME_UNARY_F32_TO_I32                   :                              /*opnd:*/ F32 /*oprtr:*/ SER_OPRTR_TYP =>            /*res:*/ I32,
    interface_unary_f64_to_i32      TRAP_NAME_UNARY_F64_TO_I32                   :                              /*opnd:*/ F64 /*oprtr:*/ SER_OPRTR_TYP =>            /*res:*/ I32,
    interface_unary_i32_to_i64      TRAP_NAME_UNARY_I32_TO_I64                   :                              /*opnd:*/ I32 /*oprtr:*/ SER_OPRTR_TYP =>            /*res:*/ I64,
    interface_unary_f32_to_i64      TRAP_NAME_UNARY_F32_TO_I64                   :                              /*opnd:*/ F32 /*oprtr:*/ SER_OPRTR_TYP =>            /*res:*/ I64,
    interface_unary_f64_to_i64      TRAP_NAME_UNARY_F64_TO_I64                   :                              /*opnd:*/ F64 /*oprtr:*/ SER_OPRTR_TYP =>            /*res:*/ I64,
    interface_unary_i32_to_f32      TRAP_NAME_UNARY_I32_TO_F32                   :                              /*opnd:*/ I32 /*oprtr:*/ SER_OPRTR_TYP =>            /*res:*/ F32,
    interface_unary_i64_to_f32      TRAP_NAME_UNARY_I64_TO_F32                   :                              /*opnd:*/ I64 /*oprtr:*/ SER_OPRTR_TYP =>            /*res:*/ F32,
    interface_unary_f64_to_f32      TRAP_NAME_UNARY_F64_TO_F32                   :                              /*opnd:*/ F64 /*oprtr:*/ SER_OPRTR_TYP =>            /*res:*/ F32,
    interface_unary_i32_to_f64      TRAP_NAME_UNARY_I32_TO_F64                   :                              /*opnd:*/ I32 /*oprtr:*/ SER_OPRTR_TYP =>            /*res:*/ F64,
    interface_unary_i64_to_f64      TRAP_NAME_UNARY_I64_TO_F64                   :                              /*opnd:*/ I64 /*oprtr:*/ SER_OPRTR_TYP =>            /*res:*/ F64,
    interface_unary_f32_to_f64      TRAP_NAME_UNARY_F32_TO_F64                   :                              /*opnd:*/ F32 /*oprtr:*/ SER_OPRTR_TYP =>            /*res:*/ F64,
    interface_local_get_i32         TRAP_NAME_LOCAL_GET_I32                      :                                         /*value:*/ I32 /*idx:*/ I64 =>          /*value:*/ I32,
    interface_local_set_i32         TRAP_NAME_LOCAL_SET_I32                      :                                         /*value:*/ I32 /*idx:*/ I64 =>          /*value:*/ I32,
    interface_local_tee_i32         TRAP_NAME_LOCAL_TEE_I32                      :                                         /*value:*/ I32 /*idx:*/ I64 =>          /*value:*/ I32,
    interface_local_get_f32         TRAP_NAME_LOCAL_GET_F32                      :                                         /*value:*/ F32 /*idx:*/ I64 =>          /*value:*/ F32,
    interface_local_set_f32         TRAP_NAME_LOCAL_SET_F32                      :                                         /*value:*/ F32 /*idx:*/ I64 =>          /*value:*/ F32,
    interface_local_tee_f32         TRAP_NAME_LOCAL_TEE_F32                      :                                         /*value:*/ F32 /*idx:*/ I64 =>          /*value:*/ F32,
    interface_local_get_i64         TRAP_NAME_LOCAL_GET_I64                      :                                         /*value:*/ I64 /*idx:*/ I64 =>          /*value:*/ I64,
    interface_local_set_i64         TRAP_NAME_LOCAL_SET_I64                      :                                         /*value:*/ I64 /*idx:*/ I64 =>          /*value:*/ I64,
    interface_local_tee_i64         TRAP_NAME_LOCAL_TEE_I64                      :                                         /*value:*/ I64 /*idx:*/ I64 =>          /*value:*/ I64,
    interface_local_get_f64         TRAP_NAME_LOCAL_GET_F64                      :                                         /*value:*/ F64 /*idx:*/ I64 =>          /*value:*/ F64,
    interface_local_set_f64         TRAP_NAME_LOCAL_SET_F64                      :                                         /*value:*/ F64 /*idx:*/ I64 =>          /*value:*/ F64,
    interface_local_tee_f64         TRAP_NAME_LOCAL_TEE_F64                      :                                         /*value:*/ F64 /*idx:*/ I64 =>          /*value:*/ F64,
    interface_global_get_i32        TRAP_NAME_GLOBAL_GET_I32                     :                                         /*value:*/ I32 /*idx:*/ I64 =>          /*value:*/ I32,
    interface_global_set_i32        TRAP_NAME_GLOBAL_SET_I32                     :                                         /*value:*/ I32 /*idx:*/ I64 =>          /*value:*/ I32,
    interface_global_get_f32        TRAP_NAME_GLOBAL_GET_F32                     :                                         /*value:*/ F32 /*idx:*/ I64 =>          /*value:*/ F32,
    interface_global_set_f32        TRAP_NAME_GLOBAL_SET_F32                     :                                         /*value:*/ F32 /*idx:*/ I64 =>          /*value:*/ F32,
    interface_global_get_i64        TRAP_NAME_GLOBAL_GET_I64                     :                                         /*value:*/ I64 /*idx:*/ I64 =>          /*value:*/ I64,
    interface_global_set_i64        TRAP_NAME_GLOBAL_SET_I64                     :                                         /*value:*/ I64 /*idx:*/ I64 =>          /*value:*/ I64,
    interface_global_get_f64        TRAP_NAME_GLOBAL_GET_F64                     :                                         /*value:*/ F64 /*idx:*/ I64 =>          /*value:*/ F64,
    interface_global_set_f64        TRAP_NAME_GLOBAL_SET_F64                     :                                         /*value:*/ F64 /*idx:*/ I64 =>          /*value:*/ F64,
    interface_f32_store             TRAP_NAME_F32_STORE                          : /*write_idx:*/ I32 /*val:*/ F32 /*offs:*/ I64 /*op:*/ SER_OPRTR_TYP =>                /*void*/,
    interface_f64_store             TRAP_NAME_F64_STORE                          : /*write_idx:*/ I32 /*val:*/ F64 /*offs:*/ I64 /*op:*/ SER_OPRTR_TYP =>                /*void*/,
    interface_i32_store             TRAP_NAME_I32_STORE                          : /*write_idx:*/ I32 /*val:*/ I32 /*offs:*/ I64 /*op:*/ SER_OPRTR_TYP =>                /*void*/,
    interface_i64_store             TRAP_NAME_I64_STORE                          : /*write_idx:*/ I32 /*val:*/ I64 /*offs:*/ I64 /*op:*/ SER_OPRTR_TYP =>                /*void*/,
    interface_f32_load              TRAP_NAME_F32_LOAD                           :               /*load_idx:*/ I32 /*offs:*/ I64 /*op:*/ SER_OPRTR_TYP =>            /*res:*/ F32,
    interface_f64_load              TRAP_NAME_F64_LOAD                           :               /*load_idx:*/ I32 /*offs:*/ I64 /*op:*/ SER_OPRTR_TYP =>            /*res:*/ F64,
    interface_i32_load              TRAP_NAME_I32_LOAD                           :               /*load_idx:*/ I32 /*offs:*/ I64 /*op:*/ SER_OPRTR_TYP =>            /*res:*/ I32,
    interface_i64_load              TRAP_NAME_I64_LOAD                           :               /*load_idx:*/ I32 /*offs:*/ I64 /*op:*/ SER_OPRTR_TYP =>            /*res:*/ I64,
    interface_memory_size           TRAP_NAME_MEMORY_SIZE                        :                                          /*size:*/ I32 /*idx:*/ I64 =>           /*size:*/ I32,
    interface_memory_grow           TRAP_NAME_MEMORY_GROW                        :                                        /*amount:*/ I32 /*idx:*/ I64 => /*delta-or-neg-1:*/ I32,
}
