use wastrumentation_rs_stdlib::*;
use shadow_execution_analysis::*;

#[derive(Default, Clone, Debug)]
pub struct Taint(bool);
impl ShadowMeta for Taint {
    type ShadowMetaByte = bool;

    fn decompose(&self, len: usize) -> Vec<Self::ShadowMetaByte> {
        vec![self.0; len]
    }

    fn recompose(bytes: Vec<Self::ShadowMetaByte>) -> Self {
        Self(bytes.into_iter().any(|b| b))
    }
}
impl PartialEq for Taint {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

shadow_execution!(Taint);

fn fmt_val(v: &ShadowValue<Taint>) -> String {
    match v {
        ShadowValue { value, meta } => {
            match value {
                WasmValue::I32(i) => format!("i32({})[{}]", i, meta.0),
                WasmValue::I64(i) => format!("i64({})[{}]", i, meta.0),
                WasmValue::F32(f) => format!("f32({})[{}]", f, meta.0),
                WasmValue::F64(f) => format!("f64({})[{}]", f, meta.0),
            }
        }
    }
}

fn fmt_loc(loc: &Location) -> String {
    format!("fn{}+{}", loc.function_index(), loc.instruction_index())
}

#[no_mangle]
pub fn apply(function: &WasmFunction, args: &MutDynArgs, ress: &MutDynResults) {
    let arg_strs: Vec<String> = args
        .args_iter()
        .map(|v| fmt_val(&ShadowValue::from(v)))
        .collect();
    let res_strs: Vec<String> = ress
        .ress_iter()
        .map(|v| fmt_val(&ShadowValue::from(v)))
        .collect();
    println!(
        "[apply] fn_idx={} args=[{}] results=[{}]",
        function.instr_f_idx,
        arg_strs.join(", "),
        res_strs.join(", ")
    );
}

#[no_mangle]
pub fn if_then_else(
    path_continuation: &PathContinuation,
    if_then_else_input_c: &IfThenElseInputCount,
    if_then_else_arity: &IfThenElseArity,
    location: &Location
) {
    let branch = if path_continuation.is_then() { "then" } else { "else" };
    println!(
        "[if_then_else] {} @ {} (inputs={}, arity={})",
        branch,
        fmt_loc(location),
        if_then_else_input_c.value(),
        if_then_else_arity.value()
    );
}

#[no_mangle]
pub fn if_then(
    path_continuation: &PathContinuation,
    if_then_input_c: &IfThenInputCount,
    if_then_arity: &IfThenArity,
    location: &Location
) {
    let branch = if path_continuation.is_then() { "then" } else { "skip" };
    println!(
        "[if_then] {} @ {} (inputs={}, arity={})",
        branch,
        fmt_loc(location),
        if_then_input_c.value(),
        if_then_arity.value()
    );
}

#[no_mangle]
pub fn if_then_else_post(location: &Location) {
    println!("[if_then_else_post] @ {}", fmt_loc(location));
}

#[no_mangle]
pub fn if_then_post(location: &Location) {
    println!("[if_then_post] @ {}", fmt_loc(location));
}

#[no_mangle]
pub fn br(branch_target_label: &BranchTargetLabel, location: &Location) {
    println!("[br] label={} @ {}", branch_target_label.label(), fmt_loc(location));
}

#[no_mangle]
pub fn br_if(
    path_continuation: &ParameterBrIfCondition,
    target_label: &ParameterBrIfLabel,
    location: &Location
) {
    let taken = if path_continuation.is_then() { "taken" } else { "not taken" };
    println!("[br_if] {} label={} @ {}", taken, target_label.label(), fmt_loc(location));
}

#[no_mangle]
pub fn br_table(
    branch_table_target: &BranchTableTarget,
    branch_table_effective: &BranchTableEffective,
    branch_table_default: &BranchTableDefault,
    location: &Location
) {
    println!(
        "[br_table] target={} effective={} default={} @ {}",
        branch_table_target.target(),
        branch_table_effective.label(),
        branch_table_default.value(),
        fmt_loc(location)
    );
}

#[no_mangle]
pub fn select(path_continuation: &PathContinuation, location: &Location) {
    let chosen = if path_continuation.is_then() { "val1" } else { "val2" };
    println!("[select] chose {} @ {}", chosen, fmt_loc(location));
}

#[no_mangle]
pub fn call_indirect_pre(
    target_func: &FunctionTableIndex,
    _func_table_ident: &FunctionTable,
    location: &Location
) {
    println!("[call_indirect_pre] table_index={} @ {}", target_func.value(), fmt_loc(location));
}

#[no_mangle]
pub fn call_indirect_post(_target_func: &FunctionTable, location: &Location) {
    println!("[call_indirect_post] @ {}", fmt_loc(location));
}

#[no_mangle]
pub fn call_pre(_target_func: &FunctionIndex, location: &Location) {
    println!("[call_pre] target_fn={} @ {}", _target_func.value(), fmt_loc(location));
}

#[no_mangle]
pub fn call_post(_target_func: &FunctionIndex, location: &Location) {
    println!("[call_post] target_fn={} @ {}", _target_func.value(), fmt_loc(location));
}

#[no_mangle]
pub fn unary(unop: &UnaryOperator, c_1: &ShadowValue<Taint>, location: &Location) {
    println!("[unary] {:?} {} @ {}", unop, fmt_val(c_1), fmt_loc(location));
}

#[no_mangle]
pub fn binary(
    binop: &BinaryOperator,
    c_1: &ShadowValue<Taint>,
    c_2: &ShadowValue<Taint>,
    c: &mut ShadowValue<Taint>,
    location: &Location
) {
    // if matches!(*binop, BinaryOperator::I32Mul) {
    //     c.meta_mut().0 = true;
    // }
    // if c_1.meta().0 || c_2.meta().0 {
    //     println!("One of the operands is tainted, propagating taint to result");
    // }
    println!(
        "[binary] {:?} {} {} res: {} @ {}",
        binop,
        fmt_val(c_1),
        fmt_val(c_2),
        fmt_val(c),
        fmt_loc(location)
    );
}

#[no_mangle]
pub fn drop(location: &Location) {
    // Peek at what is about to be dropped without disturbing the stack.
    let top = SHADOW_STACK.with_borrow(|stack| {
        match stack.top_of_stack() {
            StackEntry::Value(v) => fmt_val(v),
            _ => "<non-value>".to_string(),
        }
    });
    println!("[drop] dropping {} @ {}", top, fmt_loc(location));
}

#[no_mangle]
pub fn return_(location: &Location) {
    let depth = SHADOW_STACK.with_borrow(|stack| stack.stack_label_count());
    println!("[return] label_depth={} @ {}", depth, fmt_loc(location));
}

#[no_mangle]
pub fn const_(value: &ShadowValue<Taint>, location: &Location) {
    println!("[const] {} @ {}", fmt_val(value), fmt_loc(location));
}

#[no_mangle]
pub fn local(
    value: &ShadowValue<Taint>,
    index: &LocalIndex,
    local_op: &LocalOp,
    location: &Location
) {
    let op = match local_op {
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
pub fn global(
    value: &ShadowValue<Taint>,
    index: &GlobalIndex,
    global_op: &GlobalOp,
    location: &Location
) {
    let op = match global_op {
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
pub fn load(
    store_index: &LoadIndex,
    offset: &LoadOffset,
    operation: &LoadOperation,
    location: &Location
) {
    println!(
        "[load] {:?} ptr={} offset={} @ {}",
        operation,
        store_index.value(),
        offset.value(),
        fmt_loc(location)
    );
}

#[no_mangle]
pub fn store(
    store_index: &StoreIndex,
    value: &ShadowValue<Taint>,
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
pub fn memory_size(size: &ShadowValue<Taint>, index: &MemoryIndex, location: &Location) {
    println!("[memory.size] mem={} size={} @ {}", index.value(), fmt_val(size), fmt_loc(location));
}

#[no_mangle]
pub fn memory_grow(amount: &ShadowValue<Taint>, index: &MemoryIndex, location: &Location) {
    println!(
        "[memory.grow] mem={} amount={} @ {}",
        index.value(),
        fmt_val(amount),
        fmt_loc(location)
    );
}

#[no_mangle]
pub fn memory_init(location: &Location) {
    println!("[memory.init] @ {}", fmt_loc(location));
}

#[no_mangle]
pub fn memory_copy(location: &Location) {
    println!("[memory.copy] @ {}", fmt_loc(location));
}

#[no_mangle]
pub fn memory_fill(location: &Location) {
    println!("[memory.fill] @ {}", fmt_loc(location));
}

#[no_mangle]
pub fn block_pre(
    block_input_count: &BlockInputCount,
    block_arity: &BlockArity,
    location: &Location
) {
    println!(
        "[block] enter inputs={} arity={} @ {}",
        block_input_count.value(),
        block_arity.value(),
        fmt_loc(location)
    );
}

#[no_mangle]
pub fn block_post(location: &Location) {
    println!("[block] exit @ {}", fmt_loc(location));
}

#[no_mangle]
pub fn loop_pre(loop_input_count: &LoopInputCount, loop_arity: &LoopArity, location: &Location) {
    println!(
        "[loop] enter inputs={} arity={} @ {}",
        loop_input_count.value(),
        loop_arity.value(),
        fmt_loc(location)
    );
}

#[no_mangle]
pub fn loop_post(location: &Location) {
    println!("[loop] exit @ {}", fmt_loc(location));
}
