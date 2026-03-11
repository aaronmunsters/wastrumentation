#[macro_export]
macro_rules! declare_shadow_traps {
    ($M:ty) => {
        pub mod shadow_traps {
            use super::*;

            extern "Rust" {

                pub fn apply_before(
                    function: &WasmFunction,
                    args: &MutDynArgs,
                    ress: &MutDynResults,
                ) -> ();

                pub fn apply_after(
                    function: &WasmFunction,
                    args: &MutDynArgs,
                    ress: &MutDynResults,
                ) -> ();

                pub fn if_then_else_before(
                    path_continuation: &PathContinuation,
                    if_then_else_input_c: &IfThenElseInputCount,
                    if_then_else_arity: &IfThenElseArity,
                    location: &Location,
                ) -> ();

                pub fn if_then_else_after(
                    path_continuation: &PathContinuation,
                    if_then_else_input_c: &IfThenElseInputCount,
                    if_then_else_arity: &IfThenElseArity,
                    location: &Location,
                ) -> ();

                pub fn if_then_before(
                    path_continuation: &PathContinuation,
                    if_then_input_c: &IfThenInputCount,
                    if_then_arity: &IfThenArity,
                    location: &Location,
                ) -> ();

                pub fn if_then_after(
                    path_continuation: &PathContinuation,
                    if_then_input_c: &IfThenInputCount,
                    if_then_arity: &IfThenArity,
                    location: &Location,
                ) -> ();

                pub fn if_then_else_post_before(location: &Location) -> ();

                pub fn if_then_else_post_after(location: &Location) -> ();

                pub fn if_then_post_before(location: &Location) -> ();

                pub fn if_then_post_after(location: &Location) -> ();

                pub fn br_before(
                    branch_target_label: &BranchTargetLabel,
                    location: &Location,
                ) -> ();

                pub fn br_after(branch_target_label: &BranchTargetLabel, location: &Location)
                    -> ();

                pub fn br_if_before(
                    path_continuation: &ParameterBrIfCondition,
                    target_label: &ParameterBrIfLabel,
                    location: &Location,
                );

                pub fn br_if_after(
                    path_continuation: &ParameterBrIfCondition,
                    target_label: &ParameterBrIfLabel,
                    location: &Location,
                );

                pub fn br_table_before(
                    branch_table_target: &BranchTableTarget,
                    branch_table_effective: &BranchTableEffective,
                    branch_table_default: &BranchTableDefault,
                    location: &Location,
                ) -> ();

                pub fn br_table_after(
                    branch_table_target: &BranchTableTarget,
                    branch_table_effective: &BranchTableEffective,
                    branch_table_default: &BranchTableDefault,
                    location: &Location,
                ) -> ();

                pub fn select_before(
                    path_continuation: &PathContinuation,
                    location: &Location,
                ) -> ();

                pub fn select_after(
                    path_continuation: &PathContinuation,
                    location: &Location,
                ) -> ();

                pub fn call_indirect_pre(
                    target_func: &FunctionTableIndex,
                    func_table_ident: &FunctionTable,
                    location: &Location,
                ) -> ();

                pub fn call_indirect_post(target_func: &FunctionTable, location: &Location) -> ();

                pub fn call_pre(target_func: &FunctionIndex, location: &Location) -> ();

                pub fn call_post(target_func: &FunctionIndex, location: &Location) -> ();

                pub fn unary(
                    unop: &UnaryOperator,
                    c_1: &ShadowValue<$M>,
                    c: &mut ShadowValue<$M>,
                    location: &Location,
                ) -> ();

                pub fn binary(
                    binop: &BinaryOperator,
                    c_1: &ShadowValue<$M>,
                    c_2: &ShadowValue<$M>,
                    c: &mut ShadowValue<$M>,
                    location: &Location,
                ) -> ();

                pub fn drop(location: &Location) -> ();

                pub fn return_before(location: &Location) -> ();

                pub fn return_after(location: &Location) -> ();

                pub fn const_(value: &ShadowValue<$M>, location: &Location) -> ();

                pub fn local(
                    value: &mut ShadowValue<$M>,
                    index: &LocalIndex,
                    local_op: &LocalOp,
                    location: &Location,
                ) -> ();

                pub fn global(
                    value: &mut ShadowValue<$M>,
                    index: &GlobalIndex,
                    global_op: &GlobalOp,
                    location: &Location,
                ) -> ();

                pub fn load(
                    store_index: &LoadIndex,
                    loaded_value: &mut ShadowValue<$M>,
                    offset: &LoadOffset,
                    operation: &LoadOperation,
                    location: &Location,
                ) -> ();

                pub fn store(
                    store_index: &StoreIndex,
                    value: &mut ShadowValue<$M>,
                    offset: &StoreOffset,
                    operation: &StoreOperation,
                    location: &Location,
                ) -> ();

                pub fn memory_size(
                    size: &mut ShadowValue<$M>,
                    index: &MemoryIndex,
                    location: &Location,
                ) -> ();

                pub fn memory_grow(
                    amount: &mut ShadowValue<$M>,
                    index: &MemoryIndex,
                    location: &Location,
                ) -> ();

                pub fn memory_init(location: &Location) -> ();

                pub fn memory_copy(location: &Location) -> ();

                pub fn memory_fill(location: &Location) -> ();

                pub fn block_pre_before(
                    block_input_count: &BlockInputCount,
                    block_arity: &BlockArity,
                    location: &Location,
                ) -> ();

                pub fn block_pre_after(
                    block_input_count: &BlockInputCount,
                    block_arity: &BlockArity,
                    location: &Location,
                ) -> ();

                pub fn block_post_before(location: &Location) -> ();

                pub fn block_post_after(location: &Location) -> ();

                pub fn loop_pre_before(
                    loop_input_count: &LoopInputCount,
                    loop_arity: &LoopArity,
                    location: &Location,
                ) -> ();

                pub fn loop_pre_after(
                    loop_input_count: &LoopInputCount,
                    loop_arity: &LoopArity,
                    location: &Location,
                ) -> ();

                pub fn loop_post_before(location: &Location) -> ();

                pub fn loop_post_after(location: &Location) -> ();

                pub fn call_to_imported_before(argument_count: usize) -> ();

                pub fn call_to_imported_after(result_count: usize) -> ();
            }
        }
    };
}
