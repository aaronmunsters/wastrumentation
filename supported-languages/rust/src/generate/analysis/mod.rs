use std::collections::HashSet;

use crate::compile::{Rust, options::RustSource};
use serde::Deserialize;
use wastrumentation::analysis::{AnalysisInterface, ProcessedAnalysis};

#[derive(Clone)]
pub struct RustAnalysisSpec {
    pub source: RustSource,
    pub hooks: HashSet<Hook>,
}

impl From<RustAnalysisSpec> for ProcessedAnalysis<Rust> {
    fn from(value: RustAnalysisSpec) -> Self {
        let RustAnalysisSpec { ref hooks, source } = value;
        let analysis_interface: AnalysisInterface = interface_from(hooks);

        ProcessedAnalysis {
            analysis_interface,
            analysis_library: source,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash, Deserialize)]
pub enum Hook {
    GenericApply,
    CallPre,
    CallPost,
    CallIndirectPre,
    CallIndirectPost,
    IfThen,
    IfThenPost,
    IfThenElse,
    IfThenElsePost,
    Branch,
    BranchIf,
    BranchTable,
    Select,
    Unary,
    Binary,
    Drop,
    Return,
    Const,
    Local,
    Global,
    Store,
    Load,
    MemorySize,
    MemoryGrow,
    BlockPre,
    BlockPost,
    LoopPre,
    LoopPost,
    RefFunc,
    RefNull,
    RefIsNull,
    TableGet,
    TableSet,
    TableSize,
    TableGrow,
    TableFill,
    TableCopy,
    TableInit,
    ElemDrop,
}

impl Hook {
    pub fn all_hooks() -> HashSet<Self> {
        use Hook::*;
        HashSet::from([
            GenericApply,
            CallPre,
            CallPost,
            CallIndirectPre,
            CallIndirectPost,
            IfThen,
            IfThenPost,
            IfThenElse,
            IfThenElsePost,
            Branch,
            BranchIf,
            BranchTable,
            Select,
            Unary,
            Binary,
            Drop,
            Return,
            Const,
            Local,
            Global,
            Store,
            Load,
            MemorySize,
            MemoryGrow,
            BlockPre,
            BlockPost,
            LoopPre,
            LoopPost,
            RefFunc,
            RefNull,
            RefIsNull,
            TableGet,
            TableSet,
            TableSize,
            TableGrow,
            TableFill,
            TableCopy,
            TableInit,
            ElemDrop,
        ])
    }
}

pub fn interface_from(hooks: &HashSet<Hook>) -> AnalysisInterface {
    let mut interface = AnalysisInterface::default();
    for hook in hooks {
        match hook {
            Hook::GenericApply => {
                interface.generic_interface = Some(AnalysisInterface::interface_generic_apply())
            }
            Hook::CallPre => {
                interface.pre_trap_call = Some(AnalysisInterface::interface_call_pre())
            }
            Hook::CallPost => {
                interface.post_trap_call = Some(AnalysisInterface::interface_call_post())
            }
            Hook::CallIndirectPre => {
                interface.pre_trap_call_indirect =
                    Some(AnalysisInterface::interface_call_indirect_pre())
            }
            Hook::CallIndirectPost => {
                interface.post_trap_call_indirect =
                    Some(AnalysisInterface::interface_call_indirect_post())
            }
            Hook::IfThen => interface.if_then_trap = Some(AnalysisInterface::interface_if_then()),
            Hook::IfThenPost => {
                interface.if_then_post_trap = Some(AnalysisInterface::interface_if_then_post())
            }
            Hook::IfThenElse => {
                interface.if_then_else_trap = Some(AnalysisInterface::interface_if_then_else())
            }
            Hook::IfThenElsePost => {
                interface.if_then_else_post_trap =
                    Some(AnalysisInterface::interface_if_then_else_post())
            }
            Hook::Branch => interface.br_trap = Some(AnalysisInterface::interface_br()),
            Hook::BranchIf => interface.br_if_trap = Some(AnalysisInterface::interface_br_if()),
            Hook::BranchTable => {
                interface.br_table_trap = Some(AnalysisInterface::interface_br_table())
            }
            Hook::Select => interface.select = Some(AnalysisInterface::interface_select()),
            Hook::Unary => {
                interface.unary_i32_to_i32 = Some(AnalysisInterface::interface_unary_i32_to_i32());
                interface.unary_i64_to_i32 = Some(AnalysisInterface::interface_unary_i64_to_i32());
                interface.unary_i64_to_i64 = Some(AnalysisInterface::interface_unary_i64_to_i64());
                interface.unary_f32_to_f32 = Some(AnalysisInterface::interface_unary_f32_to_f32());
                interface.unary_f64_to_f64 = Some(AnalysisInterface::interface_unary_f64_to_f64());
                interface.unary_f32_to_i32 = Some(AnalysisInterface::interface_unary_f32_to_i32());
                interface.unary_f64_to_i32 = Some(AnalysisInterface::interface_unary_f64_to_i32());
                interface.unary_i32_to_i64 = Some(AnalysisInterface::interface_unary_i32_to_i64());
                interface.unary_f32_to_i64 = Some(AnalysisInterface::interface_unary_f32_to_i64());
                interface.unary_f64_to_i64 = Some(AnalysisInterface::interface_unary_f64_to_i64());
                interface.unary_i32_to_f32 = Some(AnalysisInterface::interface_unary_i32_to_f32());
                interface.unary_i64_to_f32 = Some(AnalysisInterface::interface_unary_i64_to_f32());
                interface.unary_f64_to_f32 = Some(AnalysisInterface::interface_unary_f64_to_f32());
                interface.unary_i32_to_f64 = Some(AnalysisInterface::interface_unary_i32_to_f64());
                interface.unary_i64_to_f64 = Some(AnalysisInterface::interface_unary_i64_to_f64());
                interface.unary_f32_to_f64 = Some(AnalysisInterface::interface_unary_f32_to_f64());
            }
            Hook::Binary => {
                interface.binary_i32_i32_to_i32 =
                    Some(AnalysisInterface::interface_binary_i32_i32_to_i32());
                interface.binary_i64_i64_to_i32 =
                    Some(AnalysisInterface::interface_binary_i64_i64_to_i32());
                interface.binary_f32_f32_to_i32 =
                    Some(AnalysisInterface::interface_binary_f32_f32_to_i32());
                interface.binary_f64_f64_to_i32 =
                    Some(AnalysisInterface::interface_binary_f64_f64_to_i32());
                interface.binary_i64_i64_to_i64 =
                    Some(AnalysisInterface::interface_binary_i64_i64_to_i64());
                interface.binary_f32_f32_to_f32 =
                    Some(AnalysisInterface::interface_binary_f32_f32_to_f32());
                interface.binary_f64_f64_to_f64 =
                    Some(AnalysisInterface::interface_binary_f64_f64_to_f64());
            }
            Hook::Drop => interface.drop_trap = Some(AnalysisInterface::interface_drop()),
            Hook::Return => interface.return_trap = Some(AnalysisInterface::interface_return()),
            Hook::Const => {
                interface.const_i32_trap = Some(AnalysisInterface::interface_const_i32());
                interface.const_f32_trap = Some(AnalysisInterface::interface_const_f32());
                interface.const_i64_trap = Some(AnalysisInterface::interface_const_i64());
                interface.const_f64_trap = Some(AnalysisInterface::interface_const_f64());
            }
            Hook::Local => {
                interface.local_get_i32 = Some(AnalysisInterface::interface_local_get_i32());
                interface.local_set_i32 = Some(AnalysisInterface::interface_local_set_i32());
                interface.local_tee_i32 = Some(AnalysisInterface::interface_local_tee_i32());
                interface.local_get_f32 = Some(AnalysisInterface::interface_local_get_f32());
                interface.local_set_f32 = Some(AnalysisInterface::interface_local_set_f32());
                interface.local_tee_f32 = Some(AnalysisInterface::interface_local_tee_f32());
                interface.local_get_i64 = Some(AnalysisInterface::interface_local_get_i64());
                interface.local_set_i64 = Some(AnalysisInterface::interface_local_set_i64());
                interface.local_tee_i64 = Some(AnalysisInterface::interface_local_tee_i64());
                interface.local_get_f64 = Some(AnalysisInterface::interface_local_get_f64());
                interface.local_set_f64 = Some(AnalysisInterface::interface_local_set_f64());
                interface.local_tee_f64 = Some(AnalysisInterface::interface_local_tee_f64());
            }
            Hook::Global => {
                interface.global_get_i32 = Some(AnalysisInterface::interface_global_get_i32());
                interface.global_set_i32 = Some(AnalysisInterface::interface_global_set_i32());
                interface.global_get_f32 = Some(AnalysisInterface::interface_global_get_f32());
                interface.global_set_f32 = Some(AnalysisInterface::interface_global_set_f32());
                interface.global_get_i64 = Some(AnalysisInterface::interface_global_get_i64());
                interface.global_set_i64 = Some(AnalysisInterface::interface_global_set_i64());
                interface.global_get_f64 = Some(AnalysisInterface::interface_global_get_f64());
                interface.global_set_f64 = Some(AnalysisInterface::interface_global_set_f64());
            }
            Hook::Store => {
                interface.f32_store = Some(AnalysisInterface::interface_f32_store());
                interface.f64_store = Some(AnalysisInterface::interface_f64_store());
                interface.i32_store = Some(AnalysisInterface::interface_i32_store());
                interface.i64_store = Some(AnalysisInterface::interface_i64_store());
            }
            Hook::Load => {
                interface.f32_load = Some(AnalysisInterface::interface_f32_load());
                interface.f64_load = Some(AnalysisInterface::interface_f64_load());
                interface.i32_load = Some(AnalysisInterface::interface_i32_load());
                interface.i64_load = Some(AnalysisInterface::interface_i64_load());
            }
            Hook::MemorySize => {
                interface.memory_size = Some(AnalysisInterface::interface_memory_size())
            }
            Hook::MemoryGrow => {
                interface.memory_grow = Some(AnalysisInterface::interface_memory_grow())
            }
            Hook::BlockPre => {
                interface.pre_block = Some(AnalysisInterface::interface_pre_block());
            }
            Hook::BlockPost => {
                interface.post_block = Some(AnalysisInterface::interface_post_block());
            }
            Hook::LoopPre => {
                interface.pre_loop = Some(AnalysisInterface::interface_pre_loop());
            }
            Hook::LoopPost => {
                interface.post_loop = Some(AnalysisInterface::interface_post_loop());
            }
            Hook::RefFunc => {
                interface.ref_func = Some(AnalysisInterface::interface_ref_func());
            }
            Hook::RefNull => {
                interface.ref_null = Some(AnalysisInterface::interface_ref_null());
            }
            Hook::RefIsNull => {
                interface.ref_is_null = Some(AnalysisInterface::interface_ref_is_null());
            }
            Hook::TableGet => {
                interface.table_get = Some(AnalysisInterface::interface_table_get());
            }
            Hook::TableSet => {
                interface.table_set = Some(AnalysisInterface::interface_table_set());
            }
            Hook::TableSize => {
                interface.table_size = Some(AnalysisInterface::interface_table_size());
            }
            Hook::TableGrow => {
                interface.table_grow = Some(AnalysisInterface::interface_table_grow());
            }
            Hook::TableFill => {
                interface.table_fill = Some(AnalysisInterface::interface_table_fill());
            }
            Hook::TableCopy => {
                interface.table_copy = Some(AnalysisInterface::interface_table_copy());
                interface.table_copy_get_source =
                    Some(AnalysisInterface::interface_table_copy_get_source());
                interface.table_copy_get_destination =
                    Some(AnalysisInterface::interface_table_copy_get_destination());
                interface.table_copy_get_size =
                    Some(AnalysisInterface::interface_table_copy_get_size());
            }
            Hook::TableInit => {
                interface.table_init = Some(AnalysisInterface::interface_table_init());
                interface.table_init_get_element_source =
                    Some(AnalysisInterface::interface_table_init_get_element_source());
                interface.table_init_get_table_destination =
                    Some(AnalysisInterface::interface_table_init_get_table_destination());
                interface.table_init_get_size =
                    Some(AnalysisInterface::interface_table_init_get_size());
            }
            Hook::ElemDrop => {
                interface.elem_drop = Some(AnalysisInterface::interface_elem_drop());
            }
        }
    }
    interface
}
