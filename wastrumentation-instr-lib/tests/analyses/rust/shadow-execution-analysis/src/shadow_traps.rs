#[macro_export]
macro_rules! declare_shadow_traps {
    ($M:ty) => {
        pub mod shadow_traps {
            use super::*;

            extern "Rust" {
                pub fn apply_before(ctx: &mut ApplyBeforeTrapContext<$M>);
                pub fn apply_after(ctx: &mut ApplyAfterTrapContext<$M>);
                pub fn if_then_else_before(ctx: &mut IfThenElseTrapContext<$M>);
                pub fn if_then_else_after(ctx: &mut IfThenElseTrapContext<$M>);
                pub fn if_then_before(ctx: &mut IfThenTrapContext<$M>);
                pub fn if_then_after(ctx: &mut IfThenTrapContext<$M>);
                pub fn if_then_else_post_before(ctx: &mut IfThenElsePostTrapContext<$M>);
                pub fn if_then_else_post_after(ctx: &mut IfThenElsePostTrapContext<$M>);
                pub fn if_then_post_before(ctx: &mut IfThenPostTrapContext<$M>);
                pub fn if_then_post_after(ctx: &mut IfThenPostTrapContext<$M>);
                pub fn br_before(ctx: &mut BrTrapContext<$M>);
                pub fn br_after(ctx: &mut BrTrapContext<$M>);
                pub fn br_if_before(ctx: &mut BrIfTrapContext<$M>);
                pub fn br_if_after(ctx: &mut BrIfTrapContext<$M>);
                pub fn br_table_before(ctx: &mut BrTableTrapContext<$M>);
                pub fn br_table_after(ctx: &mut BrTableTrapContext<$M>);
                pub fn select_before(ctx: &mut SelectTrapContext<$M>);
                pub fn select_after(ctx: &mut SelectTrapContext<$M>);
                pub fn call_indirect_pre(ctx: &mut CallIndirectPreTrapContext<$M>);
                pub fn call_indirect_post(ctx: &mut CallIndirectPostTrapContext<$M>);
                pub fn call_pre(ctx: &mut CallPreTrapContext<$M>);
                pub fn call_post(ctx: &mut CallPostTrapContext<$M>);
                pub fn unary(ctx: &mut UnaryTrapContext<$M>);
                pub fn binary(ctx: &mut BinaryTrapContext<$M>);
                pub fn drop(ctx: &mut DropTrapContext<$M>);
                pub fn return_before(ctx: &mut ReturnTrapContext<$M>);
                pub fn return_after(ctx: &mut ReturnTrapContext<$M>);
                pub fn const_(ctx: &mut ConstTrapContext<$M>);
                pub fn local(ctx: &mut LocalTrapContext<$M>);
                pub fn global(ctx: &mut GlobalTrapContext<$M>);
                pub fn load(ctx: &mut LoadTrapContext<$M>);
                pub fn store(ctx: &mut StoreTrapContext<$M>);
                pub fn memory_size(ctx: &mut MemorySizeTrapContext<$M>);
                pub fn memory_grow(ctx: &mut MemoryGrowTrapContext<$M>);
                pub fn memory_init(ctx: &mut MemoryInitTrapContext<$M>);
                pub fn memory_copy(ctx: &mut MemoryCopyTrapContext<$M>);
                pub fn memory_fill(ctx: &mut MemoryFillTrapContext<$M>);
                pub fn block_pre_before(ctx: &mut BlockPreTrapContext<$M>);
                pub fn block_pre_after(ctx: &mut BlockPreTrapContext<$M>);
                pub fn block_post_before(ctx: &mut BlockPostTrapContext<$M>);
                pub fn block_post_after(ctx: &mut BlockPostTrapContext<$M>);
                pub fn loop_pre_before(ctx: &mut LoopPreTrapContext<$M>);
                pub fn loop_pre_after(ctx: &mut LoopPreTrapContext<$M>);
                pub fn loop_post_before(ctx: &mut LoopPostTrapContext<$M>);
                pub fn loop_post_after(ctx: &mut LoopPostTrapContext<$M>);
                pub fn call_to_imported_before(ctx: &mut CallToImportedBeforeTrapContext<$M>);
                pub fn call_to_imported_after(ctx: &mut CallToImportedAfterTrapContext<$M>);
            }
        }
    };
}
