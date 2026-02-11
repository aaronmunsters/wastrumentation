use super::*;

// Declaration of the ShadowAdvice, client must provide this
extern "Rust" {

    pub(crate) fn apply(function: &WasmFunction, args: &MutDynArgs, ress: &MutDynResults) -> ();

    pub(crate) fn if_then_else(
        path_continuation: &PathContinuation,
        if_then_else_input_c: &IfThenElseInputCount,
        if_then_else_arity: &IfThenElseArity,
        location: &Location,
    ) -> ();

    pub(crate) fn if_then(
        path_continuation: &PathContinuation,
        if_then_input_c: &IfThenInputCount,
        if_then_arity: &IfThenArity,
        location: &Location,
    ) -> ();

    pub(crate) fn if_then_else_post(location: &Location) -> ();

    pub(crate) fn if_then_post(location: &Location) -> ();

    pub(crate) fn br(branch_target_label: &BranchTargetLabel, location: &Location) -> ();

    pub(crate) fn br_if(
        path_continuation: &ParameterBrIfCondition,
        target_label: &ParameterBrIfLabel,
        location: &Location,
    );

    pub(crate) fn br_table(
        branch_table_target: &BranchTableTarget,
        branch_table_effective: &BranchTableEffective,
        branch_table_default: &BranchTableDefault,
        location: &Location,
    ) -> ();

    pub(crate) fn select(path_continuation: &PathContinuation, location: &Location) -> ();

    pub(crate) fn call_indirect_pre(
        target_func: &FunctionTableIndex,
        _func_table_ident: &FunctionTable,
        location: &Location,
    ) -> ();

    pub(crate) fn call_indirect_post(_target_func: &FunctionTable, location: &Location) -> ();

    pub(crate) fn call_pre(_target_func: &FunctionIndex, location: &Location) -> ();

    pub(crate) fn call_post(_target_func: &FunctionIndex, location: &Location) -> ();

    pub(crate) fn unary(unop: &UnaryOperator, c_1: &WasmValue, location: &Location) -> ();

    pub(crate) fn binary(
        binop: &BinaryOperator,
        c_1: &WasmValue,
        c_2: &WasmValue,
        location: &Location,
    ) -> ();

    pub(crate) fn drop(location: &Location) -> ();

    pub(crate) fn return_(location: &Location) -> ();

    pub(crate) fn const_(value: &WasmValue, location: &Location) -> ();

    pub(crate) fn local(
        value: &WasmValue,
        index: &LocalIndex,
        local_op: &LocalOp,
        location: &Location,
    ) -> ();

    pub(crate) fn global(
        value: &WasmValue,
        index: &GlobalIndex,
        global_op: &GlobalOp,
        location: &Location,
    ) -> ();

    pub(crate) fn load(
        store_index: &LoadIndex,
        offset: &LoadOffset,
        operation: &LoadOperation,
        location: &Location,
    ) -> ();

    pub(crate) fn store(
        store_index: &StoreIndex,
        value: &WasmValue,
        offset: &StoreOffset,
        operation: &StoreOperation,
        location: &Location,
    ) -> ();

    pub(crate) fn memory_size(size: &WasmValue, index: &MemoryIndex, location: &Location) -> ();

    pub(crate) fn memory_grow(amount: &WasmValue, index: &MemoryIndex, location: &Location) -> ();

    pub(crate) fn memory_init(location: &Location) -> ();

    pub(crate) fn memory_copy(location: &Location) -> ();

    pub(crate) fn memory_fill(location: &Location) -> ();

    pub(crate) fn block_pre(
        block_input_count: &BlockInputCount,
        block_arity: &BlockArity,
        location: &Location,
    ) -> ();

    pub(crate) fn block_post(location: &Location) -> ();

    pub(crate) fn loop_pre(
        loop_input_count: &LoopInputCount,
        loop_arity: &LoopArity,
        location: &Location,
    ) -> ();

    pub(crate) fn loop_post(location: &Location) -> ();
}
