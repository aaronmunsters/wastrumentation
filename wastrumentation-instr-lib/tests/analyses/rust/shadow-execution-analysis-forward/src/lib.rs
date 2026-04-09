use shadow_execution_analysis::*;
use wastrumentation_rs_stdlib::*;

#[derive(Default, Clone, Debug, PartialEq)]
pub struct Meta;

impl ShadowMeta for Meta {
    type ShadowMetaByte = u8;
    fn decompose(&self, len: usize) -> Vec<u8> {
        vec![0; len]
    }
    fn recompose(_: Vec<u8>) -> Self {
        Self
    }
}

shadow_execution!(Meta);

#[no_mangle]
pub unsafe fn apply_before(_ctx: &mut ApplyBeforeTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn apply_after(_ctx: &mut ApplyAfterTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn call_to_imported_before(_ctx: &mut CallToImportedBeforeTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn call_to_imported_after(_ctx: &mut CallToImportedAfterTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn call_pre(_ctx: &mut CallPreTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn call_post(_ctx: &mut CallPostTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn call_indirect_pre(_ctx: &mut CallIndirectPreTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn call_indirect_post(_ctx: &mut CallIndirectPostTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn if_then_else_before(_ctx: &mut IfThenElseTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn if_then_else_after(_ctx: &mut IfThenElseTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn if_then_before(_ctx: &mut IfThenTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn if_then_after(_ctx: &mut IfThenTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn if_then_else_post_before(_ctx: &mut IfThenElsePostTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn if_then_else_post_after(_ctx: &mut IfThenElsePostTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn if_then_post_before(_ctx: &mut IfThenPostTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn if_then_post_after(_ctx: &mut IfThenPostTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn br_before(_ctx: &mut BrTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn br_after(_ctx: &mut BrTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn br_if_before(_ctx: &mut BrIfTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn br_if_after(_ctx: &mut BrIfTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn br_table_before(_ctx: &mut BrTableTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn br_table_after(_ctx: &mut BrTableTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn select_before(_ctx: &mut SelectTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn select_after(_ctx: &mut SelectTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn unary(_ctx: &mut UnaryTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn binary(_ctx: &mut BinaryTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn drop(_ctx: &mut DropTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn return_before(_ctx: &mut ReturnTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn return_after(_ctx: &mut ReturnTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn const_(_ctx: &mut ConstTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn local(_ctx: &mut LocalTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn global(_ctx: &mut GlobalTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn load(_ctx: &mut LoadTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn store(_ctx: &mut StoreTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn memory_size(_ctx: &mut MemorySizeTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn memory_grow(_ctx: &mut MemoryGrowTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn memory_init(_ctx: &mut MemoryInitTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn memory_copy(_ctx: &mut MemoryCopyTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn memory_fill(_ctx: &mut MemoryFillTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn block_pre_before(_ctx: &mut BlockPreTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn block_pre_after(_ctx: &mut BlockPreTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn block_post_before(_ctx: &mut BlockPostTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn block_post_after(_ctx: &mut BlockPostTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn loop_pre_before(_ctx: &mut LoopPreTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn loop_pre_after(_ctx: &mut LoopPreTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn loop_post_before(_ctx: &mut LoopPostTrapContext<Meta>) {
    /* <TEMPLATE> */
}

#[no_mangle]
pub unsafe fn loop_post_after(_ctx: &mut LoopPostTrapContext<Meta>) {
    /* <TEMPLATE> */
}
