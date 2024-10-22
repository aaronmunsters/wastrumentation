extern crate wastrumentation_rs_stdlib;

use std::ptr::addr_of_mut;

use wastrumentation_rs_stdlib::*;

// Author: Michael Pradel

/*
 * Simple taint analysis that considers explicit flows only
 * (i.e., no flows caused by control flow dependencies, but only data flow).
 */

// Mirror program state; track taints instead of actual value
static mut STACK: Vec<StackFrame> = vec![];
static mut MEMORY: Vec<Taint> = vec![];
static mut GLOBALS: Vec<Taint> = vec![];
// to propagate return value's taint from end() to call_post()
static mut RETURN_VALUE: Option<Taint> = None;

struct StackFrame {
    blocks: Vec<Block>,
    locals: Vec<Option<Taint>>,
}

struct Block {
    values: Vec<Taint>,
}

impl Block {
    fn new() -> Block {
        Self { values: vec![] }
    }
}

// can hold any kind of more complex label; for now, just 0 (not tainted) and 1 (tainted)
#[derive(Copy, Clone)]
struct Taint {
    label: bool,
}

impl Taint {
    fn new() -> Self {
        Self { label: false }
    }

    fn with(label: bool) -> Self {
        Self { label }
    }
}

fn peek() -> Option<&'static mut StackFrame> {
    stack_mut().last_mut()
}

fn stack_mut() -> &'static mut Vec<StackFrame> {
    unsafe { addr_of_mut!(STACK).as_mut().unwrap() }
}

fn values() -> Option<&'static mut Vec<Taint>> {
    Some(peek()?.blocks.last_mut()?.values.as_mut())
}

fn join(taint1: Taint, taint2: Taint) -> Taint {
    Taint::with(taint1.label || taint2.label)
}

fn locals_set(index: LocalIndex, taint: Option<Taint>) {
    let index: usize = index.value().try_into().unwrap();
    peek().map(|vs| {
        if vs.locals.len() <= index {
            vs.locals.resize(index + 1, None);
        }
        vs.locals[index] = taint;
    });
}

fn locals_get(index: LocalIndex) -> Taint {
    let index: usize = index.value().try_into().unwrap();
    peek()
        .and_then(|vs| vs.locals.get(index))
        .and_then(|local| local.clone())
        .unwrap_or(Taint::new())
}

fn globals_set(index: GlobalIndex, taint: Option<Taint>) {
    let index = index.value().try_into().unwrap();
    let globals = unsafe { addr_of_mut!(GLOBALS).as_mut().unwrap() };
    if globals.len() <= index {
        globals.resize(index + 1, Taint::new());
    }
    if let Some(taint) = taint {
        globals[index] = taint;
    }
}

fn globals_get(index: GlobalIndex) -> Taint {
    let index: usize = index.value().try_into().unwrap();
    let globals = unsafe { addr_of_mut!(GLOBALS).as_mut().unwrap() };
    if globals.len() <= index {
        globals.resize(index + 1, Taint::new());
    }
    globals[index]
}

fn memory_get(store_index: &LoadIndex, offset: &LoadOffset) -> Taint {
    let store_index: usize = store_index.value().try_into().unwrap();
    let offset: usize = offset.value().try_into().unwrap();
    let effective = store_index + offset;

    let memory = unsafe { addr_of_mut!(MEMORY).as_mut().unwrap() };
    if memory.len() <= effective {
        memory.resize(effective + 1, Taint::new());
    }
    memory[effective]
}

fn memory_set(store_index: &StoreIndex, taint: Taint, offset: &StoreOffset) {
    let store_index: usize = store_index.value().try_into().unwrap();
    let offset: usize = offset.value().try_into().unwrap();
    let effective = store_index + offset;

    let memory = unsafe { addr_of_mut!(MEMORY).as_mut().unwrap() };
    if memory.len() <= effective {
        memory.resize(effective + 1, Taint::new());
    }
    memory[effective] = taint;
}

// Not directly an advice, but called from begin advices
fn begin() {
    peek().map(|s| s.blocks.push(Block::new()));
}

enum BlockType {
    Function,
    BlockType,
}

fn end(type_: BlockType) {
    let result_taint_arr = peek().and_then(|s| s.blocks.pop());
    // FIXME sometimes pop() returns undefined, not just an empty []. Why?
    // hacky workaround: just return early if [result_taint] pattern match would fail
    let Some(result_taint_arr) = result_taint_arr else {
        return;
    };

    let result_taint = result_taint_arr.values.last();
    match (type_, result_taint) {
        (BlockType::Function, Some(value)) => unsafe { RETURN_VALUE = Some(*value) },
        _ => (),
    }
}

advice! { if_ (
        path_continuation: PathContinuation,
        _if_then_else_input_c: IfThenElseInputCount,
        _if_then_else_arity: IfThenElseArity,
        _location: Location,
    ) {
        values().and_then(|vs| vs.pop());
        path_continuation
    }
}

advice! { if_post (
        _location: Location,
    ) {
        end(BlockType::BlockType);
    }
}

advice! { if_then (
    path_continuation: PathContinuation,
    _if_then_input_c: IfThenInputCount,
    _if_then_arity: IfThenArity,
    _location: Location,
    ) {
        values().and_then(|vs| vs.pop());
        path_continuation
    }
}

advice! { if_then_post (
        _location: Location,
    ) {
        end(BlockType::BlockType);
    }
}

advice! { br (
        _branch_target_label: BranchTargetLabel,
        _location: Location,
    ) {
        peek().and_then(|sf| sf.blocks.pop());
    }
}

advice! { br_if (
        path_continuation : ParameterBrIfCondition,
        _target_label : ParameterBrIfLabel,
        _location: Location,
    ) {
        values().and_then(|vs| vs.pop());
        if path_continuation.is_then() {
            peek().and_then(|sf| sf.blocks.pop());
        }

        path_continuation
    }
}

advice! { br_table (
        branch_table_target: BranchTableTarget,
        _branch_table_effective: BranchTableEffective,
        _branch_table_default: BranchTableDefault,
        _location: Location,
    ) {
        values().and_then(|vs| vs.pop());
        peek().and_then(|sf| sf.blocks.pop());

        branch_table_target
    }
}

advice! { select (
        path_continuation: PathContinuation,
        _location: Location,
    ) {
        values().and_then(|vs| vs.pop());
        let taint2 = values().and_then(|vs| vs.pop());
        let taint1 = values().and_then(|vs| vs.pop());
        match (taint2, taint1) {
            (Some(taint2), Some(taint1)) => {
                if path_continuation.is_then() {
                    values().map(|vs| vs.push(taint1));
                } else {
                    values().map(|vs| vs.push(taint2));
                }
            }
            ,
            _ => {
                values().map(|vs| vs.push(Taint::with(false)));
            }
        }

        path_continuation
    }
}

advice! { apply (function : WasmFunction, args : MutDynArgs, ress : MutDynResults) {
        let arg_taints: Vec<Option<Taint>> = args.args_iter().map(|_| values().and_then(|vs| vs.pop().map(|_| Taint::new()))).collect();
        stack_mut().push(StackFrame {
            blocks: vec![ Block { values : vec![] } ],
            locals: arg_taints,
        });
        function.apply();
        let _ = ress;
    }
}

advice! { call pre (
        _target_func : FunctionIndex,
        _location: Location,
    ) {
        /* cfr. apply */
    }
}

advice! { call post (
        _target_func : FunctionIndex,
        _location: Location,
    ) {
        stack_mut().pop();
        let return_value = unsafe { RETURN_VALUE };
        if let Some(return_value) = return_value {
            values().map(|vs|vs.push(return_value));
            unsafe { RETURN_VALUE = None };
        }
        end(BlockType::Function);
    }
}

advice! { call_indirect pre (
        target_func: FunctionTableIndex,
        _func_table_ident: FunctionTable,
        _location: Location,
    ) {
        /* cfr. apply */
        target_func
    }
}

advice! { call_indirect post (
        _target_func: FunctionTable,
        _location: Location,
    ) {
        /* cfr. apply */
        end(BlockType::Function);
    }
}

advice! { unary generic (
        operator: UnaryOperator,
        operand: WasmValue,
        _location: Location,
    ) {
        let taint = values().and_then(|vs| vs.pop());
        let taint_result = Taint::with(taint.map(|t| t.label).unwrap_or_default());
        values().map(|vs| vs.push(taint_result));

        operator.apply(operand)
    }
}

advice! { binary generic (
        operator: BinaryOperator,
        l_operand: WasmValue,
        r_operand: WasmValue,
        _location: Location,
    ) {
        let taint1 = values().and_then(|vs| vs.pop());
        let taint2 = values().and_then(|vs| vs.pop());
        let taint_result = match (taint1, taint2) {
            (Some(taint1), Some(taint2)) => {
                join(taint1, taint2)
            },
            _ => Taint::new()
        };
        values().map(|vs| vs.push(taint_result));

        operator.apply(l_operand, r_operand)
    }
}

advice! { drop (
        _location: Location,
    ) {
        values().map(|vs| vs.pop());
    }
}

advice! { return_ (
        _location: Location,
    ) {
        // Note on interaction between end() and return_():
        //  * end() may or may not be called on function returns
        //  * return_() may or may not be called on function returns
        //  * end() always happens before return_()
        //  * We try to retrieve the return value taint in end(),
        //    and if none found, we try to retrieve it in return_()
        let mut return_value = unsafe { RETURN_VALUE };
        if return_value.is_some() && peek().is_some_and(|sf| !sf.blocks.is_empty()) {
            let result_taint = peek().and_then(|sf| sf.blocks.last_mut().and_then(|b| b.values.pop()));
            return_value = result_taint.map(|t| Taint::with(t.label));
        }
        unsafe { RETURN_VALUE = return_value};
    }
}

advice! { const_ generic (
        value: WasmValue,
        _location: Location,
    ) {
        values().map(|vs| vs.push(Taint::new()));

        value
    }
}

advice! { local generic (
        value: WasmValue,
        index: LocalIndex,
        local_op: LocalOp,
        _location: Location,
    ) {
        match local_op {
            LocalOp::Set => {
                let taint = values().and_then(|vs| vs.pop());
                locals_set(index, taint);
            }
            LocalOp::Tee => {
                let taint = values().and_then(|vs| vs.pop());
                locals_set(index, taint);
            }
            LocalOp::Get => {
                let taint = locals_get(index);
                values().map(|vs| vs.push(taint));
            }
        }

        value
    }
}

advice! { global generic (
        value: WasmValue,
        index: GlobalIndex,
        global_op: GlobalOp,
        _location: Location,
    ) {
        match global_op {
            GlobalOp::Set => {
                let taint = values().and_then(|vs| vs.pop());
                globals_set(index, taint);
            }
            GlobalOp::Get => {
                let taint = globals_get(index);
                values().map(|vs| vs.push(taint));
            }
        }

        value
    }
}

advice! { load generic (
        store_index: LoadIndex,
        offset: LoadOffset,
        operation: LoadOperation,
        _location: Location,
    ) {
        values().and_then(|vs| vs.pop());
        let taint = memory_get(&store_index, &offset);
        values().map(|vs| vs.push(taint));

        let value = operation.perform(&store_index, &offset);
        value
    }
}

advice! { store generic (
        store_index: StoreIndex,
        value: WasmValue,
        offset: StoreOffset,
        operation: StoreOperation,
        _location: Location,
    ) {
        let taint = values().and_then(|vs| vs.pop()).unwrap_or(Taint::new());
        values().and_then(|vs| vs.pop());
        memory_set(&store_index, taint, &offset);

        operation.perform(&store_index, &value, &offset);
    }
}

advice! { memory_size (
        size: WasmValue,
        _index: MemoryIndex,
        _location: Location,
    ) {
        values().map(|vs| vs.push(Taint::new()));

        size
    }
}

advice! { memory_grow (
        amount: WasmValue,
        index: MemoryIndex,
        _location: Location,
    ) {
        values().and_then(|vs| vs.pop());
        values().map(|vs| vs.push(Taint::new()));

        index.grow(amount)
    }
}

advice! { block pre (
        _block_input_count: BlockInputCount,
        _block_arity: BlockArity,
        _location: Location,
    ) {
        begin();
    }
}

advice! { block post (
        _location: Location,
    ) {
        end(BlockType::BlockType);
    }
}

advice! { loop_ pre (
        _loop_input_count: LoopInputCount,
        _loop_arity: LoopArity,
        _location: Location,
    ) {
        begin();
    }
}

advice! { loop_ post (
        _location: Location,
    ) {
        end(BlockType::BlockType);
    }
}
