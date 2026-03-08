#[macro_export]
macro_rules! declare_shadow_traps {
    ($M:ty) => {
        pub mod shadow_traps {
            use super::*;

            extern "Rust" {

                pub fn apply(
                    function: &WasmFunction,
                    args: &MutDynArgs,
                    ress: &MutDynResults,
                ) -> ();

                pub fn if_then_else(
                    path_continuation: &PathContinuation,
                    if_then_else_input_c: &IfThenElseInputCount,
                    if_then_else_arity: &IfThenElseArity,
                    location: &Location,
                ) -> ();

                pub fn if_then(
                    path_continuation: &PathContinuation,
                    if_then_input_c: &IfThenInputCount,
                    if_then_arity: &IfThenArity,
                    location: &Location,
                ) -> ();

                pub fn if_then_else_post(location: &Location) -> ();

                pub fn if_then_post(location: &Location) -> ();

                pub fn br(branch_target_label: &BranchTargetLabel, location: &Location) -> ();

                pub fn br_if(
                    path_continuation: &ParameterBrIfCondition,
                    target_label: &ParameterBrIfLabel,
                    location: &Location,
                );

                pub fn br_table(
                    branch_table_target: &BranchTableTarget,
                    branch_table_effective: &BranchTableEffective,
                    branch_table_default: &BranchTableDefault,
                    location: &Location,
                ) -> ();

                pub fn select(path_continuation: &PathContinuation, location: &Location) -> ();

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

                pub fn return_(location: &Location) -> ();

                pub fn const_(value: &ShadowValue<$M>, location: &Location) -> ();

                pub fn local(
                    value: &ShadowValue<$M>,
                    index: &LocalIndex,
                    local_op: &LocalOp,
                    location: &Location,
                ) -> ();

                pub fn global(
                    value: &ShadowValue<$M>,
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
                    value: &ShadowValue<$M>,
                    offset: &StoreOffset,
                    operation: &StoreOperation,
                    location: &Location,
                ) -> ();

                pub fn memory_size(
                    size: &ShadowValue<$M>,
                    index: &MemoryIndex,
                    location: &Location,
                ) -> ();

                pub fn memory_grow(
                    amount: &ShadowValue<$M>,
                    index: &MemoryIndex,
                    location: &Location,
                ) -> ();

                pub fn memory_init(location: &Location) -> ();

                pub fn memory_copy(location: &Location) -> ();

                pub fn memory_fill(location: &Location) -> ();

                pub fn block_pre(
                    block_input_count: &BlockInputCount,
                    block_arity: &BlockArity,
                    location: &Location,
                ) -> ();

                pub fn block_post(location: &Location) -> ();

                pub fn loop_pre(
                    loop_input_count: &LoopInputCount,
                    loop_arity: &LoopArity,
                    location: &Location,
                ) -> ();

                pub fn loop_post(location: &Location) -> ();
            }
        }
    };
}
