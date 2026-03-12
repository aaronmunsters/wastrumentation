use wastrumentation_rs_stdlib::*;
use shadow_execution_analysis::*;

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

fn fmt_val(v: &ShadowValue<Meta>) -> String {
    match v.value {
        WasmValue::I32(i) => format!("i32({})", i),
        WasmValue::I64(i) => format!("i64({})", i),
        WasmValue::F32(f) => format!("f32({})", f),
        WasmValue::F64(f) => format!("f64({})", f),
    }
}

fn fmt_loc(loc: &Location) -> String {
    format!("fn{}+{}", loc.function_index(), loc.instruction_index())
}

#[no_mangle]
pub unsafe fn apply_before(function: &WasmFunction, args: &MutDynArgs, _ress: &MutDynResults) {
    let args: Vec<String> = args
        .args_iter()
        .map(|v| fmt_val(&ShadowValue::from(v)))
        .collect();
    println!("[apply_before] fn={} args=[{}]", function.instr_f_idx, args.join(", "));
}

#[no_mangle]
pub unsafe fn apply_after(function: &WasmFunction, _args: &MutDynArgs, ress: &MutDynResults) {
    let ress: Vec<String> = ress
        .ress_iter()
        .map(|v| fmt_val(&ShadowValue::from(v)))
        .collect();
    println!("[apply_after] fn={} results=[{}]", function.instr_f_idx, ress.join(", "));
}

#[no_mangle]
pub unsafe fn call_to_imported_before(argument_count: usize) {
    println!("[call_to_imported_before] args={}", argument_count);
}

#[no_mangle]
pub unsafe fn call_to_imported_after(result_count: usize) {
    println!("[call_to_imported_after] results={}", result_count);
}

#[no_mangle]
pub unsafe fn call_pre(target_func: &FunctionIndex, location: &Location) {
    println!("[call_pre] target={} @ {}", target_func.value(), fmt_loc(location));
}

#[no_mangle]
pub unsafe fn call_post(target_func: &FunctionIndex, location: &Location) {
    println!("[call_post] target={} @ {}", target_func.value(), fmt_loc(location));
}

#[no_mangle]
pub unsafe fn call_indirect_pre(
    target_func: &FunctionTableIndex,
    _func_table_ident: &FunctionTable,
    location: &Location
) {
    println!("[call_indirect_pre] table_index={} @ {}", target_func.value(), fmt_loc(location));
}

#[no_mangle]
pub unsafe fn call_indirect_post(_target_func: &FunctionTable, location: &Location) {
    println!("[call_indirect_post] @ {}", fmt_loc(location));
}

#[no_mangle]
pub unsafe fn if_then_else_before(
    path: &PathContinuation,
    input_c: &IfThenElseInputCount,
    arity: &IfThenElseArity,
    location: &Location
) {
    let branch = if path.is_then() { "then" } else { "else" };
    println!(
        "[if_then_else_before] {} inputs={} arity={} @ {}",
        branch,
        input_c.value(),
        arity.value(),
        fmt_loc(location)
    );
}

#[no_mangle]
pub unsafe fn if_then_else_after(
    path: &PathContinuation,
    input_c: &IfThenElseInputCount,
    arity: &IfThenElseArity,
    location: &Location
) {
    let branch = if path.is_then() { "then" } else { "else" };
    println!(
        "[if_then_else_after] {} inputs={} arity={} @ {}",
        branch,
        input_c.value(),
        arity.value(),
        fmt_loc(location)
    );
}

#[no_mangle]
pub unsafe fn if_then_before(
    path: &PathContinuation,
    input_c: &IfThenInputCount,
    arity: &IfThenArity,
    location: &Location
) {
    let branch = if path.is_then() { "then" } else { "skip" };
    println!(
        "[if_then_before] {} inputs={} arity={} @ {}",
        branch,
        input_c.value(),
        arity.value(),
        fmt_loc(location)
    );
}

#[no_mangle]
pub unsafe fn if_then_after(
    path: &PathContinuation,
    input_c: &IfThenInputCount,
    arity: &IfThenArity,
    location: &Location
) {
    let branch = if path.is_then() { "then" } else { "skip" };
    println!(
        "[if_then_after] {} inputs={} arity={} @ {}",
        branch,
        input_c.value(),
        arity.value(),
        fmt_loc(location)
    );
}

#[no_mangle]
pub unsafe fn if_then_else_post_before(location: &Location) {
    println!("[if_then_else_post_before] @ {}", fmt_loc(location));
}

#[no_mangle]
pub unsafe fn if_then_else_post_after(location: &Location) {
    println!("[if_then_else_post_after] @ {}", fmt_loc(location));
}

#[no_mangle]
pub unsafe fn if_then_post_before(location: &Location) {
    println!("[if_then_post_before] @ {}", fmt_loc(location));
}

#[no_mangle]
pub unsafe fn if_then_post_after(location: &Location) {
    println!("[if_then_post_after] @ {}", fmt_loc(location));
}

#[no_mangle]
pub unsafe fn br_before(target: &BranchTargetLabel, location: &Location) {
    println!("[br_before] label={} @ {}", target.label(), fmt_loc(location));
}

#[no_mangle]
pub unsafe fn br_after(target: &BranchTargetLabel, location: &Location) {
    println!("[br_after] label={} @ {}", target.label(), fmt_loc(location));
}

#[no_mangle]
pub unsafe fn br_if_before(
    path: &ParameterBrIfCondition,
    target: &ParameterBrIfLabel,
    location: &Location
) {
    let taken = if path.is_then() { "taken" } else { "not_taken" };
    println!("[br_if_before] {} label={} @ {}", taken, target.label(), fmt_loc(location));
}

#[no_mangle]
pub unsafe fn br_if_after(
    path: &ParameterBrIfCondition,
    target: &ParameterBrIfLabel,
    location: &Location
) {
    let taken = if path.is_then() { "taken" } else { "not_taken" };
    println!("[br_if_after] {} label={} @ {}", taken, target.label(), fmt_loc(location));
}

#[no_mangle]
pub unsafe fn br_table_before(
    target: &BranchTableTarget,
    effective: &BranchTableEffective,
    default: &BranchTableDefault,
    location: &Location
) {
    println!(
        "[br_table_before] target={} effective={} default={} @ {}",
        target.target(),
        effective.label(),
        default.value(),
        fmt_loc(location)
    );
}

#[no_mangle]
pub unsafe fn br_table_after(
    target: &BranchTableTarget,
    effective: &BranchTableEffective,
    default: &BranchTableDefault,
    location: &Location
) {
    println!(
        "[br_table_after] target={} effective={} default={} @ {}",
        target.target(),
        effective.label(),
        default.value(),
        fmt_loc(location)
    );
}

#[no_mangle]
pub unsafe fn select_before(path: &PathContinuation, location: &Location) {
    let chosen = if path.is_then() { "val1" } else { "val2" };
    println!("[select_before] {} @ {}", chosen, fmt_loc(location));
}

#[no_mangle]
pub unsafe fn select_after(path: &PathContinuation, location: &Location) {
    let chosen = if path.is_then() { "val1" } else { "val2" };
    println!("[select_after] {} @ {}", chosen, fmt_loc(location));
}

#[no_mangle]
pub unsafe fn unary(
    unop: &UnaryOperator,
    c_1: &ShadowValue<Meta>,
    c: &mut ShadowValue<Meta>,
    location: &Location
) {
    println!("[unary] {:?} {} -> {} @ {}", unop, fmt_val(c_1), fmt_val(c), fmt_loc(location));
}

#[no_mangle]
pub unsafe fn binary(
    binop: &BinaryOperator,
    c_1: &ShadowValue<Meta>,
    c_2: &ShadowValue<Meta>,
    c: &mut ShadowValue<Meta>,
    location: &Location
) {
    println!(
        "[binary] {:?} {} {} -> {} @ {}",
        binop,
        fmt_val(c_1),
        fmt_val(c_2),
        fmt_val(c),
        fmt_loc(location)
    );
}

#[no_mangle]
pub unsafe fn drop(location: &Location) {
    let top = SHADOW_STACK.with_borrow(|stack| {
        match stack.top_of_stack() {
            StackEntry::Value(v) => fmt_val(v),
            _ => "<non-value>".to_string(),
        }
    });
    println!("[drop] {} @ {}", top, fmt_loc(location));
}

#[no_mangle]
pub unsafe fn return_before(location: &Location) {
    let depth = SHADOW_STACK.with_borrow(|stack| stack.stack_label_count());
    println!("[return_before] label_depth={} @ {}", depth, fmt_loc(location));
}

#[no_mangle]
pub unsafe fn return_after(location: &Location) {
    println!("[return_after] @ {}", fmt_loc(location));
}

#[no_mangle]
pub unsafe fn const_(value: &mut ShadowValue<Meta>, location: &Location) {
    println!("[const] {} @ {}", fmt_val(value), fmt_loc(location));
}

#[no_mangle]
pub unsafe fn local(
    value: &mut ShadowValue<Meta>,
    index: &LocalIndex,
    op: &LocalOp,
    location: &Location
) {
    let op = match op {
        LocalOp::Get => "get",
        LocalOp::Set => "set",
        LocalOp::Tee => "tee",
    };
    println!(
        "[local.{}] index={} value={} @ {}",
        op,
        index.value(),
        fmt_val(value),
        fmt_loc(location)
    );
}

#[no_mangle]
pub unsafe fn global(
    value: &mut ShadowValue<Meta>,
    index: &GlobalIndex,
    op: &GlobalOp,
    location: &Location
) {
    let op = match op {
        GlobalOp::Get => "get",
        GlobalOp::Set => "set",
    };
    println!(
        "[global.{}] index={} value={} @ {}",
        op,
        index.value(),
        fmt_val(value),
        fmt_loc(location)
    );
}

#[no_mangle]
pub unsafe fn load(
    store_index: &LoadIndex,
    loaded_value: &mut ShadowValue<Meta>,
    offset: &LoadOffset,
    operation: &LoadOperation,
    location: &Location
) {
    println!(
        "[load] {:?} ptr={} offset={} value={} @ {}",
        operation,
        store_index.value(),
        offset.value(),
        fmt_val(loaded_value),
        fmt_loc(location)
    );
}

#[no_mangle]
pub unsafe fn store(
    store_index: &StoreIndex,
    value: &mut ShadowValue<Meta>,
    offset: &StoreOffset,
    operation: &StoreOperation,
    location: &Location
) {
    println!(
        "[store] {:?} ptr={} offset={} value={} @ {}",
        operation,
        store_index.value(),
        offset.value(),
        fmt_val(value),
        fmt_loc(location)
    );
}

#[no_mangle]
pub unsafe fn memory_size(size: &mut ShadowValue<Meta>, index: &MemoryIndex, location: &Location) {
    println!("[memory.size] mem={} size={} @ {}", index.value(), fmt_val(size), fmt_loc(location));
}

#[no_mangle]
pub unsafe fn memory_grow(
    amount: &mut ShadowValue<Meta>,
    index: &MemoryIndex,
    location: &Location
) {
    println!(
        "[memory.grow] mem={} amount={} @ {}",
        index.value(),
        fmt_val(amount),
        fmt_loc(location)
    );
}

#[no_mangle]
pub unsafe fn memory_init(location: &Location) {
    println!("[memory.init] @ {}", fmt_loc(location));
}

#[no_mangle]
pub unsafe fn memory_copy(location: &Location) {
    println!("[memory.copy] @ {}", fmt_loc(location));
}

#[no_mangle]
pub unsafe fn memory_fill(location: &Location) {
    println!("[memory.fill] @ {}", fmt_loc(location));
}

#[no_mangle]
pub unsafe fn block_pre_before(
    input_count: &BlockInputCount,
    arity: &BlockArity,
    location: &Location
) {
    println!(
        "[block_pre_before] inputs={} arity={} @ {}",
        input_count.value(),
        arity.value(),
        fmt_loc(location)
    );
}

#[no_mangle]
pub unsafe fn block_pre_after(
    input_count: &BlockInputCount,
    arity: &BlockArity,
    location: &Location
) {
    println!(
        "[block_pre_after] inputs={} arity={} @ {}",
        input_count.value(),
        arity.value(),
        fmt_loc(location)
    );
}

#[no_mangle]
pub unsafe fn block_post_before(location: &Location) {
    println!("[block_post_before] @ {}", fmt_loc(location));
}

#[no_mangle]
pub unsafe fn block_post_after(location: &Location) {
    println!("[block_post_after] @ {}", fmt_loc(location));
}

#[no_mangle]
pub unsafe fn loop_pre_before(
    input_count: &LoopInputCount,
    arity: &LoopArity,
    location: &Location
) {
    println!(
        "[loop_pre_before] inputs={} arity={} @ {}",
        input_count.value(),
        arity.value(),
        fmt_loc(location)
    );
}

#[no_mangle]
pub unsafe fn loop_pre_after(input_count: &LoopInputCount, arity: &LoopArity, location: &Location) {
    println!(
        "[loop_pre_after] inputs={} arity={} @ {}",
        input_count.value(),
        arity.value(),
        fmt_loc(location)
    );
}

#[no_mangle]
pub unsafe fn loop_post_before(location: &Location) {
    println!("[loop_post_before] @ {}", fmt_loc(location));
}

#[no_mangle]
pub unsafe fn loop_post_after(location: &Location) {
    println!("[loop_post_after] @ {}", fmt_loc(location));
}
