// Include the shadow_execution_analysis, this will add
// exports & imports to the module that the instrumentation
// platform will link to the target program.
extern crate shadow_execution_analysis;

// Include the wastrumentation_rs_stdlib, this will
// include types to declare our shadow traps
use wastrumentation_rs_stdlib::*;

#[no_mangle]
pub fn apply(function: &WasmFunction, args: &MutDynArgs, ress: &MutDynResults) -> () {
    let _ = function;
    let _ = args;
    let _ = ress;
    /* <TEMPLATE> */
}

#[no_mangle]
pub fn if_then_else(
    path_continuation: &PathContinuation,
    if_then_else_input_c: &IfThenElseInputCount,
    if_then_else_arity: &IfThenElseArity,
    location: &Location,
) -> () {
    let _ = path_continuation;
    let _ = if_then_else_input_c;
    let _ = if_then_else_arity;
    let _ = location;
    /* <TEMPLATE> */
}

#[no_mangle]
pub fn if_then(
    path_continuation: &PathContinuation,
    if_then_input_c: &IfThenInputCount,
    if_then_arity: &IfThenArity,
    location: &Location,
) -> () {
    let _ = path_continuation;
    let _ = if_then_input_c;
    let _ = if_then_arity;
    let _ = location;
    /* <TEMPLATE> */
}

#[no_mangle]
pub fn if_then_else_post(location: &Location) -> () {
    let _ = location;
    /* <TEMPLATE> */
}

#[no_mangle]
pub fn if_then_post(location: &Location) -> () {
    let _ = location;
    /* <TEMPLATE> */
}

#[no_mangle]
pub fn br(branch_target_label: &BranchTargetLabel, location: &Location) -> () {
    let _ = branch_target_label;
    let _ = location;
    /* <TEMPLATE> */
}

#[no_mangle]
pub fn br_if(
    path_continuation: &ParameterBrIfCondition,
    target_label: &ParameterBrIfLabel,
    location: &Location,
) {
    let _ = path_continuation;
    let _ = target_label;
    let _ = location;
    /* <TEMPLATE> */
}

#[no_mangle]
pub fn br_table(
    branch_table_target: &BranchTableTarget,
    branch_table_effective: &BranchTableEffective,
    branch_table_default: &BranchTableDefault,
    location: &Location,
) -> () {
    let _ = branch_table_target;
    let _ = branch_table_effective;
    let _ = branch_table_default;
    let _ = location;
    /* <TEMPLATE> */
}

#[no_mangle]
pub fn select(path_continuation: &PathContinuation, location: &Location) -> () {
    let _ = path_continuation;
    let _ = location;
    /* <TEMPLATE> */
}

#[no_mangle]
pub fn call_indirect_pre(
    target_func: &FunctionTableIndex,
    _func_table_ident: &FunctionTable,
    location: &Location,
) -> () {
    let _ = target_func;
    let _ = _func_table_ident;
    let _ = location;
    /* <TEMPLATE> */
}

#[no_mangle]
pub fn call_indirect_post(_target_func: &FunctionTable, location: &Location) -> () {
    let _ = _target_func;
    let _ = location;
    /* <TEMPLATE> */
}

#[no_mangle]
pub fn call_pre(_target_func: &FunctionIndex, location: &Location) -> () {
    let _ = _target_func;
    let _ = location;
    /* <TEMPLATE> */
}

#[no_mangle]
pub fn call_post(_target_func: &FunctionIndex, location: &Location) -> () {
    let _ = _target_func;
    let _ = location;
    /* <TEMPLATE> */
}

#[no_mangle]
pub fn unary(unop: &UnaryOperator, c_1: &WasmValue, location: &Location) -> () {
    let _ = unop;
    let _ = c_1;
    let _ = location;
    /* <TEMPLATE> */
}

#[no_mangle]
pub fn binary(binop: &BinaryOperator, c_1: &WasmValue, c_2: &WasmValue, location: &Location) -> () {
    let _ = binop;
    let _ = c_1;
    let _ = c_2;
    let _ = location;
    /* <TEMPLATE> */
}

#[no_mangle]
pub fn drop(location: &Location) -> () {
    let _ = location;
    /* <TEMPLATE> */
}

#[no_mangle]
pub fn return_(location: &Location) -> () {
    let _ = location;
    /* <TEMPLATE> */
}

#[no_mangle]
pub fn const_(value: &WasmValue, location: &Location) -> () {
    let _ = value;
    let _ = location;
    /* <TEMPLATE> */
}

#[no_mangle]
pub fn local(value: &WasmValue, index: &LocalIndex, local_op: &LocalOp, location: &Location) -> () {
    let _ = value;
    let _ = index;
    let _ = local_op;
    let _ = location;
    /* <TEMPLATE> */
}

#[no_mangle]
pub fn global(
    value: &WasmValue,
    index: &GlobalIndex,
    global_op: &GlobalOp,
    location: &Location,
) -> () {
    let _ = value;
    let _ = index;
    let _ = global_op;
    let _ = location;
    /* <TEMPLATE> */
}

#[no_mangle]
pub fn load(
    store_index: &LoadIndex,
    offset: &LoadOffset,
    operation: &LoadOperation,
    location: &Location,
) -> () {
    let _ = store_index;
    let _ = offset;
    let _ = operation;
    let _ = location;
    /* <TEMPLATE> */
}

#[no_mangle]
pub fn store(
    store_index: &StoreIndex,
    value: &WasmValue,
    offset: &StoreOffset,
    operation: &StoreOperation,
    location: &Location,
) -> () {
    let _ = store_index;
    let _ = value;
    let _ = offset;
    let _ = operation;
    let _ = location;
    /* <TEMPLATE> */
}

#[no_mangle]
pub fn memory_size(size: &WasmValue, index: &MemoryIndex, location: &Location) -> () {
    let _ = size;
    let _ = index;
    let _ = location;
    /* <TEMPLATE> */
}

#[no_mangle]
pub fn memory_grow(amount: &WasmValue, index: &MemoryIndex, location: &Location) -> () {
    let _ = amount;
    let _ = index;
    let _ = location;
    /* <TEMPLATE> */
}

#[no_mangle]
pub fn memory_init(location: &Location) -> () {
    let _ = location;
    /* <TEMPLATE> */
}

#[no_mangle]
pub fn memory_copy(location: &Location) -> () {
    let _ = location;
    /* <TEMPLATE> */
}

#[no_mangle]
pub fn memory_fill(location: &Location) -> () {
    let _ = location;
    /* <TEMPLATE> */
}

#[no_mangle]
pub fn block_pre(
    block_input_count: &BlockInputCount,
    block_arity: &BlockArity,
    location: &Location,
) -> () {
    let _ = block_input_count;
    let _ = block_arity;
    let _ = location;
    /* <TEMPLATE> */
}

#[no_mangle]
pub fn block_post(location: &Location) -> () {
    let _ = location;
    /* <TEMPLATE> */
}

#[no_mangle]
pub fn loop_pre(
    loop_input_count: &LoopInputCount,
    loop_arity: &LoopArity,
    location: &Location,
) -> () {
    let _ = loop_input_count;
    let _ = loop_arity;
    let _ = location;
    /* <TEMPLATE> */
}

#[no_mangle]
pub fn loop_post(location: &Location) -> () {
    let _ = location;
    /* <TEMPLATE> */
}
