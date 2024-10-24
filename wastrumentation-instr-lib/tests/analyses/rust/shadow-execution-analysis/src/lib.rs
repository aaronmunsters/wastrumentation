// Author: Aäron Munsters

mod global_store;
mod shadow_memory;
mod shadow_stack;

use global_store::*;
use shadow_memory::*;
use shadow_stack::*;
use wastrumentation_rs_stdlib::*;

static mut JUMP_FLAG: bool = false;
static mut CALL_STACK_DEPTH: i32 = 0;

fn host_is_caller() -> bool {
    let call_stack_depth = unsafe { CALL_STACK_DEPTH };
    call_stack_depth == 0
}

/// The first call on the module must come from the host.
/// As such this function prepares the shadow stack with
/// the arguments.
fn handle_host_is_caller_setup(args: &MutDynArgs) -> () {
    let host_arity = usize::MAX;
    let host_function_index = usize::MAX;
    let host_arguments = vec![];
    let host_frame = Frame::new(host_arity, host_function_index, host_arguments);
    push_activation_on_stack(host_frame);
    args.args_iter().for_each(push_value_on_stack);
}

fn handle_call_to_imported(function: WasmFunction, args: &MutDynArgs, ress: &MutDynResults) {
    println!("handle_call_to_imported>>pre-pop-args");
    args.args_iter()
        .collect::<Vec<WasmValue>>()
        .into_iter()
        .for_each(|_| {
            let _ = pop_value_from_stack();
        });
    println!("handle_call_to_imported>>post-pop-args");
    function.apply();
    // If the function is imported, we manually handle our shadow stack
    // since the body of the imported function could not reflect stack
    // changes to our shadow stack datastructure
    ress.ress_iter()
        .collect::<Vec<WasmValue>>()
        .into_iter()
        .rev()
        .for_each(push_value_on_stack)
}

#[allow(non_snake_case)]
fn enter_block_with_label(L: Label) {
    // https://webassembly.github.io/spec/core/exec/instructions.html#entering-xref-syntax-instructions-syntax-instr-mathit-instr-ast-with-label-l
    // 1. Push `L` to the stack.
    push_label_on_stack(L);
    // 2. Jump to the start of the instruction sequence `instr^{*}`.
    "handled by VM";
}

#[allow(non_snake_case)]
fn exit_instr_with_label() {
    // https://webassembly.github.io/spec/core/exec/instructions.html#exiting-xref-syntax-instructions-syntax-instr-mathit-instr-ast-with-label-l
    // 1. Pop all values `val^{*}` from the top of the stack.
    let mut values: Vec<WasmValue> = vec![];
    while !matches!(top_of_stack(), StackEntry::Label(_)) {
        println!("exit_instr_with_label>>pop_value_from_stack");
        values.push(pop_value_from_stack())
    }
    // 2. Assert: due to validation, the label `L` is now on the top of the stack.
    assert!(matches!(top_of_stack(), StackEntry::Label(_)));
    // 3. Pop the label from the stack.
    let _ = pop_label_from_stack();
    // 4. Push `val^{*}` back to the stack.
    while let Some(value) = values.pop() {
        push_value_on_stack(value);
    }
    // 5. Jump to the position after the `end` of the structured control instruction associated with the label `L`.
    "handled by VM";
}

fn is_jump_flag_set() -> bool {
    unsafe { JUMP_FLAG }
}

fn set_jump_flag_true() {
    unsafe { JUMP_FLAG = true }
}

fn set_jump_flag_false() {
    unsafe { JUMP_FLAG = false }
}

/////
// START ADVICE SPECIFICATION //
//                         /////

advice! { apply (function: WasmFunction, args: MutDynArgs, ress: MutDynResults) {
        let call_to_imported = function.is_imported();
        println!("apply ({function_index}: WasmFunction, call-to-imported: {call_to_imported})", function_index = function.instr_f_idx);

        if host_is_caller() {
            handle_host_is_caller_setup(&args);
        }

        if function.is_imported() {
            handle_call_to_imported(function, &args, &ress);
            return;
        }

        ////////////////////////////////////
        // Invocation of function address //
        ////////////////////////////////////
        // https://webassembly.github.io/spec/core/exec/instructions.html#invocation-of-function-address-a
        //  1. Assert: due to validation, `S.funcs[a]` exists.
        "skipped assertion";
        //  2. Let `f` be the function instance, `S.funcs[a]`.
        let f = function;
        //  3. Let `[t^{n}_{1}] -> [t^{m}_{1}]` be the function type `f.type`.
        let n = args.argc.try_into().unwrap();
        let m = args.resc.try_into().unwrap();
        //  4. Let `[t^{*}]` be the list of value types `f.code.locals`.
        "handled by VM";
        //  5. Let `instr^{*}` be the expression `f.code.body`.
        "handled by VM";
        //  6. Assert: due to validation, `n` values are on the top of the stack.
        println!("assert_at_least_n_values_on_stack>>apply[1]");
        assert_at_least_n_values_on_stack(n);
        //  7. Pop the values `val^{n}` from the stack.
        let actual_values: Vec<WasmValue> = args.args_iter().collect();
        println!("apply>>pre>>pop_value_from_stack");
        let shadow_values = (0..n).map(|_| pop_value_from_stack()).collect::<Vec<WasmValue>>().into_iter().rev().collect();
        assert_eq!(&shadow_values, &actual_values);
        //  8. Let `F` be the frame `{module f.module, locals val^{n} (default_{t})^{*}}`.
        #[allow(non_snake_case)]
        let F = Frame::new(m, f.instr_f_idx.try_into().unwrap(), shadow_values);
        //  9. Push the activation of `F` with arity `m` to the stack.
        push_activation_on_stack(F);
        // 10. Let `L` be the label whose arity is `m` and whose continuation is the end of the function.
        #[allow(non_snake_case)]
        let L = Label::new(m, LabelOrigin::Function(f.instr_f_idx.try_into().unwrap()));
        // 11. Enter the instruction sequence `instr^{*}` with label `L`.
        enter_block_with_label(L);

        unsafe { CALL_STACK_DEPTH += 1 };
        println!("pre>>f.apply(); FIDX:{}", f.instr_f_idx);
        f.apply();
        println!("post>>f.apply(); FIDX:{}", f.instr_f_idx);
        unsafe { CALL_STACK_DEPTH -= 1 };

        println!("post>>f.apply(); FIDX:{} is_jump_flag_set? {}", f.instr_f_idx, is_jump_flag_set());
        if is_jump_flag_set() {
            set_jump_flag_false();
            return;
        } else {
            // Implicitly the `end` of a function is reached
            println!("apply>>exit_instr_with_label");
            exit_instr_with_label();
        }

        ///////////////////////////////
        // Returning from a function //
        ///////////////////////////////
        // https://webassembly.github.io/spec/core/exec/instructions.html#returning-from-a-function
        println!("apply>>returning from function");
        // 1. Let `F` be the current frame.
        #[allow(non_snake_case)]
        let F = current_frame();
        let function_index = F.function_index();
        // 2. Let `n` be the arity of the activation of `F`.
        let n = ress.resc.try_into().unwrap();
        // 3. Assert: due to validation, there are `n` values on the top of the stack.
        println!("assert_at_least_n_values_on_stack>>[2] (n == {n:?})");
        assert_at_least_n_values_on_stack(n);
        // 4. Pop the results `val^{n}` from the stack.
        println!("apply>>post>>pop_value_from_stack");
        let mut values: Vec<WasmValue> = (0..n).map(|_| pop_value_from_stack()).collect();
        // 5. Assert: due to validation, the frame `F` is now on the top of the stack.
        let StackEntry::Frame(top_of_stack_frame) = top_of_stack() else {
            panic!();
        };
        assert!(top_of_stack_frame.function_index() == function_index);
        // 6. Pop the frame from the stack.
        #[allow(non_snake_case)]
        let _ = pop_frame_from_stack();
        // 7. Push `val^{n}` back to the stack.
        while let Some(value) = values.pop() {
            push_value_on_stack(value);
        }
        // 8. Jump to the instruction after the original call.
        "handled by VM";
    }
}

// https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-control-mathsf-br-l
fn br_with(l: usize) {
    // 1. Assert: due to validation, the stack contains at least `l + 1` labels.
    assert!(stack_label_count() >= l + 1);
    // 2. Let `L` be the `l`-th label appearing on the stack, starting from the top and counting from zero.
    #[allow(non_snake_case)]
    let L = lth_label_on_stack_starting_from_top_counting_from_zero(l);
    println!("br_with --> target label: {L:?}");
    // 3. Let `n` be the arity of `L`.
    let n = L.arity();
    // 4. Assert: due to validation, there are at least `n` values on the top of the stack.
    println!("assert_at_least_n_values_on_stack>>br_with");
    assert_at_least_n_values_on_stack(n);
    // 5. Pop the values `val^{n}` from the stack.
    println!("br_with>>first_case>>pop_value_from_stack");
    let mut values: Vec<WasmValue> = (0..n).map(|_| pop_value_from_stack()).collect();
    println!("values popped for the br_with: {values:?}");
    // 6. Repeat `l + 1` times:
    for _ in 0..(l + 1) {
        // a. While the top of the stack is a value, do:
        while matches!(top_of_stack(), StackEntry::Value(_)) {
            // i. Pop the value from the stack.
            println!("br_with>>second_case>>pop_value_from_stack");
            let _popped: WasmValue = pop_value_from_stack();
        }
        // b. Assert: due to validation, the top of the stack now is a label.
        assert!(matches!(top_of_stack(), StackEntry::Label(_)));
        // c. Pop the label from the stack.
        let _popped = pop_label_from_stack();
    }
    // 7. Push the values `val^{n}` to the stack.
    while let Some(value) = values.pop() {
        push_value_on_stack(value);
    }
    // 8. Jump to the continuation of `L`.
    "taken care of by hook termination of caller";
    if matches!(top_of_stack(), StackEntry::Label(label) if matches!(label.origin(), LabelOrigin::Function(_)))
    {
        set_jump_flag_true();
    }
}

advice! { if_ (
        path_continuation: PathContinuation,
        if_then_else_input_c: IfThenElseInputCount,
        if_then_else_arity: IfThenElseArity,
        _location: Location,
    ) {
        println!("if_ ({path_continuation:?}: PathContinuation)");
        let arguments = if_then_else_input_c.value().try_into().unwrap();
        let results = if_then_else_arity.value().try_into().unwrap();
        let if_then_else_block_type = BlockType { origin: LabelOrigin::If, arguments, results };
        // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-control-mathsf-if-xref-syntax-instructions-syntax-blocktype-mathit-blocktype-xref-syntax-instructions-syntax-instr-mathit-instr-1-ast-xref-syntax-instructions-syntax-instr-control-mathsf-else-xref-syntax-instructions-syntax-instr-mathit-instr-2-ast-xref-syntax-instructions-syntax-instr-control-mathsf-end
        // 1. Assert: due to validation, a value of value type `i32` is on the top of the stack.
        assert!(matches!(top_of_stack(), &StackEntry::Value(WasmValue::I32(_))));
        // 2. Pop the value `i32.const c` from the stack.
        let c = path_continuation;
        println!("if_then_else>>pop_value_from_stack");
        let _shadow_c = pop_value_from_stack();
        // 3. If `c` is non-zero, then:
        if c.is_then() {
            // a. Execute the block instruction `block blocktype instr^{*}_{1} end`.
            block_blocktype_instr_end(if_then_else_block_type);
            c
        // 4. Else:
        } else {
            // a. Execute the block instruction `block blocktype instr^{*}_{2} end`.
            block_blocktype_instr_end(if_then_else_block_type);
            c
        }
    }
}

advice! { if_then (
        path_continuation: PathContinuation,
        if_then_input_c: IfThenInputCount,
        if_then_arity: IfThenArity,
        _location: Location,
    ) {
        println!("if_then ({path_continuation:?}: PathContinuation)");
        let arguments = if_then_input_c.value().try_into().unwrap();
        let results = if_then_arity.value().try_into().unwrap();
        let if_then_block_type = BlockType { origin: LabelOrigin::If, arguments, results };
        // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-control-mathsf-if-xref-syntax-instructions-syntax-blocktype-mathit-blocktype-xref-syntax-instructions-syntax-instr-mathit-instr-1-ast-xref-syntax-instructions-syntax-instr-control-mathsf-else-xref-syntax-instructions-syntax-instr-mathit-instr-2-ast-xref-syntax-instructions-syntax-instr-control-mathsf-end
        // 1. Assert: due to validation, a value of value type `i32` is on the top of the stack.
        assert!(matches!(top_of_stack(), &StackEntry::Value(WasmValue::I32(_))));
        // 2. Pop the value `i32.const c` from the stack.
        let c = path_continuation;
        let shadow_c = pop_value_from_stack();
        assert_eq!(shadow_c, WasmValue::from(c.value()));
        println!("if_then>>post>>pop_value_from_stack");
        // If `c` is non-zero, then:
        if c.is_then() {
            println!("taking the 'then' branch in if_then");
            // a. Execute the block instruction `block blocktype instr^{*}_{1} end`.
            block_blocktype_instr_end(if_then_block_type);
            c
        // 4. Else:
        } else {
            // a. Execute the block instruction `block blocktype instr^{*}_{2} end`.
            block_blocktype_instr_end(if_then_block_type);
            c
        }
    }
}

advice! { if_post (_location: Location) {
        println!("if_post ()");
        println!("if_post>>exit_instr_with_label");
        exit_instr_with_label();
    }
}

advice! { if_then_post (_location: Location) {
        println!("if_then_post ()");
        println!("if_then_post>>exit_instr_with_label");
        exit_instr_with_label();
    }
}

advice! { br (branch_target_label: BranchTargetLabel, _location: Location) {
        println!("br ({branch_target_label:?}: BranchTargetLabel)");
        // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-control-mathsf-br-l
        br_with(branch_target_label.label().try_into().unwrap())
    }
}

advice! { br_if (
        path_continuation: ParameterBrIfCondition,
        target_label: ParameterBrIfLabel,
        _location: Location,
    ) {
        println!("br_if ({path_continuation:?}: ParameterBrIfCondition, {target_label:?}: ParameterBrIfLabel) [taking ~> {taking}]", taking = if path_continuation.is_then() { "then" } else { "else" });
        // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-control-mathsf-br-if-l
        // 1. Assert: due to validation, a value of value type `i32` is on the top of the stack.
        assert!(matches!(top_of_stack(), &StackEntry::Value(WasmValue::I32(_))));
        // 2. Pop the value `i32.const c` from the stack.
        let c = path_continuation;
        println!("br_if>>pop_value_from_stack");
        let shadow_c = pop_value_from_stack();
        // 3. If `c` is non-zero, then:
        if c.is_then() {
            // a. Execute the instruction `br l`.
            assert!(shadow_c.as_wasm_bool());
            br_with(target_label.label().try_into().unwrap()); }
        // 4. Else:
        else {
            assert!(!shadow_c.as_wasm_bool());
            // a. Do nothing.
        }
        c
    }
}

advice! { br_table (
        branch_table_target: BranchTableTarget,
        branch_table_effective: BranchTableEffective,
        branch_table_default: BranchTableDefault,
        _location: Location,
    ) {
        println!("br_table ({branch_table_target:?}: BranchTableTarget, {branch_table_effective:?}: BranchTableEffective, {branch_table_default:?}: BranchTableDefault)");
        // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-control-mathsf-br-table-l-ast-l-n
        let _ = branch_table_default;
        // 1. Assert: due to validation, a value of value type `i32` is on the top of the stack.
        assert!(matches!(top_of_stack(), &StackEntry::Value(WasmValue::I32(_))));
        // 2. Pop the value `i32.const i` from the stack.
        let i: BranchTableTarget = branch_table_target;
        println!("br_table>>pop_value_from_stack");
        let _shadow_i = pop_value_from_stack();
        // 3. If `i` is smaller than the length of `l*`, then:
        //      a. Let  `l_{i}` be the label `l^{*}[i]`.
        //      b. Execute the instruction `br l_{i}`.
        // 4. Else:
        //      a. Execute the instruction  `br l_{N}`.
        br_with(branch_table_effective.label().try_into().unwrap());
        println!("Really br_table with index {i:?} targetting effective branch: {branch_table_effective:?}");
        i
    }
}

advice! { select (path_continuation: PathContinuation, _location: Location) {
        println!("select ({path_continuation:?}: PathContinuation)");
        // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-parametric-mathsf-select-t-ast
        // Assert: due to validation, a value of value type `i32` is on the top of the stack.
        assert!(matches!(top_of_stack(), &StackEntry::Value(WasmValue::I32(_))));
        // Pop the value `i32.const c` from the stack.
        println!("select>>c>>pop_value_from_stack");
        let _shadow_c = pop_value_from_stack();
        // Assert: due to validation, two more values (of the same value type) are on the top of the stack.
        let (v2, v1) = top_two_values_of_stack();
        assert_eq!(v2.type_(), v1.type_());
        // Pop the value `val_{2}` from the stack.
        println!("select>>v2>>pop_value_from_stack");
        let val_2 = pop_value_from_stack();
        // Pop the value `val_{1}` from the stack.
        println!("select>>v1>>pop_value_from_stack");
        let val_1 = pop_value_from_stack();
        // If `c` is not `0`, then:
        if path_continuation.is_then() {
            // Push the value `val_{1}` back to the stack.
            push_value_on_stack(val_1);
        // Else:
        } else {
            // Push the value `val_{2}` back to the stack.
            push_value_on_stack(val_2);
        }
        path_continuation
    }
}

advice! { call_indirect pre (
        target_func: FunctionTableIndex,
        _func_table_ident: FunctionTable,
        _location: Location,
    ) {
        let FunctionTableIndex(i) = target_func;

        // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-control-mathsf-call-indirect-x-y
        // 01. Let `F` be the current frame.
        #[allow(non_snake_case)]
        let F = current_frame();
        let _ = F;
        // 02. Assert: due to validation, `F.module.tableaddrs[x]` exists.
        "skipped assertion";
        // 03. Let `ta` be the table address `F.module.tableaddrs[x]`.
        "skipped operation";
        // 04. Assert: due to validation, `S.tables[ta]` exists.
        "skipped assertion";
        // 05. Let `tab` be the table instance `S.tables[ta]`.
        "skipped operation";
        // 06. Assert: due to validation, `F.module.types[y]` exists.
        "skipped assertion";
        // 07. Let `ft_expect` be the function type `F.module.types[y]`.
        "skipped operation";
        // 08. Assert: due to validation, a value with value type `i32` is on the top of the stack.
        assert!(matches!(top_of_stack(), StackEntry::Value(WasmValue::I32(_))));
        // 09. Pop the value `i32.const i` from the stack.
        println!("call_indirect_pre>>pop_value_from_stack");
        let shadow_i = pop_value_from_stack();
        assert_eq!(shadow_i, i.into());
        // 10. If `i` is not smaller than the length of `tab.elem`, then:
        "skipped assertion";
        //         a. Trap.
        // 11. Let `r` be the reference `tab.elem[i]`.
        "skipped assertion";
        // 12. If `r` is `ref.null t`, then:
        "skipped assertion";
        //         a. Trap.
        // 13. Assert: due to validation of table mutation, `r` is a function reference.
        "skipped operation";
        // 14. Let `ref a` be the function reference `r`.
        "skipped operation";
        // 15. Assert: due to validation of table mutation, `S.funcs[a]` exists.
        "skipped operation";
        // 16. Let `f` be the function instance `S.funcs[a]`
        "skipped operation";
        // 17. Let `ft_{actual}` be the function type `f.type`.
        "skipped operation";
        // 18. If `ft_{actual}` and `ft_expect` differ, then:
        "skipped operation";
        //         a. Trap.
        // 19. Invoke the function instance at address `a`.
        "handled by VM";
        target_func
    }
}

advice! { call_indirect post (_target_func: FunctionTable, _location: Location) {
        "No particular semantics";
    }
}

advice! { call pre (_target_func: FunctionIndex, _location: Location) {
        "No particular semantics";
    }
}

advice! { call post (_target_func: FunctionIndex, _location: Location) {
        "No particular semantics";
    }
}

advice! { unary generic (unop: UnaryOperator, c_1: WasmValue, _location: Location) {
        println!("unary generic ({unop:?}: UnaryOperator, {c_1:?}: WasmValue)");
        // https://webassembly.github.io/spec/core/exec/instructions.html#t-mathsf-xref-syntax-instructions-syntax-unop-mathit-unop
        // 1. Assert: due to validation, a value of value type `t` is on the top of the stack.
        assert!(matches!(top_of_stack(), StackEntry::Value(_)));
        // 2. Pop the value `t.const c_{1}` from the stack.
        println!("unary>>pop_value_from_stack");
        let shadow_c_1 = pop_value_from_stack();
        assert_eq!(shadow_c_1, c_1);
        // 3. If `unop_{t}(c_{1})` is defined, then:
            // a. Let `c` be a possible result of computing `unop_{t}(c_{1})`.
            let c = unop.apply(c_1);
            // b. Push the value `t.const c` to the stack.
            push_value_on_stack(c.clone());
        // 4. Else:
            "skipped operation";
            // a. Trap.
        c
    }
}

advice! { binary generic (
        binop: BinaryOperator,
        c_1: WasmValue,
        c_2: WasmValue,
        _location: Location,
    ) {
        println!("binary generic ({binop:?}: BinaryOperator, {c_1:?}: WasmValue, {c_2:?}: WasmValue)");
        // 1. Assert: due to validation, two values of value type `t` are on the top of the stack.
        "handled by validation";
        // 2. Pop the value `t.const c_{2}` from the stack.
        println!("binary>>c2>>pop_value_from_stack");
        let shadow_c_2 = pop_value_from_stack();
        assert_eq!(shadow_c_2, c_2);
        // 3. Pop the value `t.const c_{1}` from the stack.
        println!("binary>>c1>>pop_value_from_stack");
        let shadow_c_1 = pop_value_from_stack();
        assert_eq!(shadow_c_1, c_1);
        // 4. If `binop_{t}(c_{1},c_{2})` is defined, then:
            // a. Let `c` be a possible result of computing `binop_{t}(c_{1},c_{2})`.
            let c = binop.apply(c_1, c_2);
            // b. Push the value `t.const c` to the stack.
            push_value_on_stack(c.clone());
        // 5. Else:
            "handled by VM";
            // a. Trap.
        c
    }
}

advice! { drop (_location: Location) {
        println!("drop ()");
        // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-parametric-mathsf-drop
        // Assert: due to validation, a value is on the top of the stack.
        assert!(matches!(top_of_stack(), StackEntry::Value(_)));
        // Pop the value `val` from the stack.
        println!("drop>>pop_value_from_stack");
        let _ = pop_value_from_stack();
    }
}

// https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-control-mathsf-return
advice! { return_ (_location: Location) {
        println!("return_ ()");
        set_jump_flag_true();
        //  1. Let {F} be the current frame.
        #[allow(non_snake_case)]
        let F = current_frame();
        let f_identifier = F.function_index();
        //  2. Let {n} be the arity of {F}.
        let n = F.arity();
        //  3. Assert: due to validation, there are at least {n} values on the top of the stack.
        println!("assert_at_least_n_values_on_stack>>return_");
        assert_at_least_n_values_on_stack(n);
        //  4. Pop the results {val^n} from the stack.
        println!("return_>>results>>pop_value_from_stack");
        let mut results: Vec<WasmValue> = (0..n).map(|_| pop_value_from_stack()).collect();
        //  5. Assert: due to validation, the stack contains at least one frame.
        "skipped assertion";
        //  6. While the top of the stack is not a frame, do:
        while !matches!(top_of_stack(), StackEntry::Frame(_)) {
            // a. Pop the top element from the stack.
            let _popped = pop_stack();
        }
        //  7. Assert: the top of the stack is the frame {F}.
        let StackEntry::Frame(top_of_stack_frame) = top_of_stack() else {
            panic!();
        };
        assert!(top_of_stack_frame.function_index() == f_identifier);
        //  8. Pop the frame from the stack.
        #[allow(non_snake_case)]
        let F = pop_frame_from_stack();
        let _ = F;
        //  9. Push {val^n} to the stack.
        while let Some(result) = results.pop() {
            push_value_on_stack(result);
        }
        // for result in results { push_value_on_stack(result); }
        // 10. Jump to the instruction after the original call that pushed the frame.
        "handled by VM";
    }
}

advice! { const_ generic (value: WasmValue, _location: Location) {
        println!("const_ generic (value: WasmValue) => {}", value.bytes_string());
        // https://webassembly.github.io/spec/core/exec/instructions.html#t-mathsf-xref-syntax-instructions-syntax-instr-numeric-mathsf-const-c
        // 1. Push the value `t.const c` to the stack.
        push_value_on_stack(value.clone());
        println!("const_ >> returning with value");
        value
    }
}

fn local_set(x: usize, actual_value: &WasmValue) {
    //  https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-variable-mathsf-local-set-x
    // 1. Let `F` be the current frame.
    #[allow(non_snake_case)]
    let F = current_frame();
    // 2. Assert: due to validation, `F.locals[x]` exists.
    F.assert_local_exists(x);
    // 3. Assert: due to validation, a value is on the top of the stack.
    assert!(matches!(top_of_stack(), StackEntry::Value(_)));
    // 4. Pop the value `val` from the stack.
    println!("local_set>>pop_value_from_stack");
    let shadow_value = pop_value_from_stack();
    assert_eq!(&shadow_value, actual_value);
    // 5. Replace `F.locals[x]` with the value `val`.
    F.replace_local_with(x, shadow_value);
}

advice! { local generic (
        value: WasmValue,
        index: LocalIndex,
        local_op: LocalOp,
        _location: Location,
    ) {
        let x: usize = index.value().try_into().unwrap();
        match local_op {
            LocalOp::Get => {
                println!("(local.{local_op:?} {x:?}) == {value:?}");
                // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-variable-mathsf-local-get-x
                // 1. Let `F` be the current frame.
                #[allow(non_snake_case)]
                let F = current_frame();
                // 2. Assert: due to validation, `F.locals[x]` exists.
                F.assert_local_exists(x);
                // 3. Let `val` be the value `F.locals[x]`.
                let shadow_val = F.get_locals(x, &value.type_());
                assert_eq!(shadow_val, value);
                // 4. Push the value `val` to the stack.
                push_value_on_stack(shadow_val);
            },
            LocalOp::Set => {
                println!("(local.{local_op:?} {x:?}) := {value:?}");
                // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-variable-mathsf-local-set-x
                local_set(x, &value);
            },
            LocalOp::Tee => {
                println!("(local.{local_op:?} {x:?}) := {value:?} & keep on stack");
                // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-variable-mathsf-local-tee-x
                // 1. Assert: due to validation, a value is on the top of the stack.
                assert!(matches!(top_of_stack(), StackEntry::Value(_)));
                // 2. Pop the value `val` from the stack.
                println!("local_tee>>pop_value_from_stack");
                let shadow_val = pop_value_from_stack();
                // 3. Push the value `val` to the stack.
                push_value_on_stack(shadow_val.clone());
                // 4. Push the value `val` to the stack.
                push_value_on_stack(shadow_val);
                // 5. Execute the instruction `local.set x`.
                local_set(x, &value);
            },
        };
        value
    }
}

advice! { global generic (
        value: WasmValue,
        index: GlobalIndex,
        global_op: GlobalOp,
        _location: Location,
    ) {
        let x: usize = index.value().try_into().unwrap();
        match global_op {
            GlobalOp::Get => {
                println!("(global.{global_op:?} {x:?}) == {value:?}");
                // Let `F` be the current frame.
                #[allow(non_snake_case)]
                let F = current_frame();
                let _ = F;
                // Assert: due to validation, `F.module.globaladdrs[x]` exists.
                "skipped assertion";
                // Let `a` be the global address `F.module.globaladdrs[x]`.
                let a = global_address(x);
                // Assert: due to validation, `S.globals[a]` exists.
                assert_global_exists(&a);
                // Let `glob` be the global instance `S.globals[a]`.
                let glob = global(a);
                // Let `val` be the value `glob.value`.
                let shadow_val = glob.value(&value.type_());
                assert_global_value(&value, &shadow_val);
                // Push the value `val` to the stack.
                push_value_on_stack(shadow_val);
            },
            GlobalOp::Set => {
                println!("(global.{global_op:?} {x:?}) := {value:?}");
                // Let `F` be the current frame.
                #[allow(non_snake_case)]
                let F = current_frame();
                let _ = F;
                // Assert: due to validation, `F.module.globaladdrs[x]` exists.
                "skipped assertion";
                // Let `a` be the global address `F.module.globaladdrs[x]`.
                let a = global_address(x);
                // Assert: due to validation, `S.globals[a]` exists.
                assert_global_exists(&a);
                // Let `glob` be the global instance `S.globals[a]`.
                let glob = global(a);
                // Assert: due to validation, a value is on the top of the stack.
                assert!(matches!(top_of_stack(), StackEntry::Value(_)));
                // Pop the value `val` from the stack.
                println!("global_set>>pop_value_from_stack");
                let shadow_val = pop_value_from_stack();
                assert_eq!(value, shadow_val);
                // Replace `glob.value` with the value `val`.
                glob.replace_value_with(shadow_val);
            },
        };
        value
    }
}

advice! { load generic (
        store_index: LoadIndex,
        offset: LoadOffset,
        operation: LoadOperation,
        _location: Location,
    ) {
        println!("load generic ({store_index:?}: LoadIndex, {offset:?}: LoadOffset, {operation:?}: LoadOperation)");
        // https://webassembly.github.io/spec/core/exec/instructions.html#t-mathsf-xref-syntax-instructions-syntax-instr-memory-mathsf-load-xref-syntax-instructions-syntax-memarg-mathit-memarg-and-t-mathsf-xref-syntax-instructions-syntax-instr-memory-mathsf-load-n-mathsf-xref-syntax-instructions-syntax-sx-mathit-sx-xref-syntax-instructions-syntax-memarg-mathit-memarg
        // TODO: link to Wasm spec
        // offset is a constant
        // store_index = dynamic address
        println!("load>>pop_value_from_stack");
        let shadow_pointer = pop_value_from_stack();
        let pointer = WasmValue::from(store_index.value());
        assert_eq!(pointer, shadow_pointer);
        let loaded_value = operation.perform(&store_index, &offset);
        let shadow_value = shadow_memory_load(shadow_pointer, &offset, operation);
        assert_shadow_memory(&loaded_value, &shadow_value);
        push_value_on_stack(loaded_value.clone());
        loaded_value
    }
}

advice! { store generic (
        store_index: StoreIndex,
        value: WasmValue,
        offset: StoreOffset,
        operation: StoreOperation,
        _location: Location,
    ) {
        println!("store generic ({store_index:?}: StoreIndex, {value:?}: WasmValue, {offset:?}: StoreOffset, {operation:?}: StoreOperation)");
        // https://webassembly.github.io/spec/core/exec/instructions.html#t-mathsf-xref-syntax-instructions-syntax-instr-memory-mathsf-store-xref-syntax-instructions-syntax-memarg-mathit-memarg-and-t-mathsf-xref-syntax-instructions-syntax-instr-memory-mathsf-store-n-xref-syntax-instructions-syntax-memarg-mathit-memarg
        // TODO: link to Wasm spec
        // offset is a constant
        // store_index = dynamic address
        let pointer = WasmValue::from(store_index.value());
        // Value to write
        println!("store_generic>>value_to_write>>pop_value_from_stack");
        let shadow_value = pop_value_from_stack();
        // Pointer
        println!("store_generic>>pointer>>pop_value_from_stack");
        let shadow_pointer = pop_value_from_stack();
        assert_eq!(pointer, shadow_pointer);
        assert_eq!(value, shadow_value);
        // Perform write
        operation.perform(&store_index, &value, &offset);
        shadow_memory_store(shadow_pointer, shadow_value, &offset, operation);
    }
}

advice! { memory_size (
        size: WasmValue,
        index: MemoryIndex,
        _location: Location,
    ) {
        println!("memory_size (size: WasmValue, index: MemoryIndex)");
        let _ = index;
        push_value_on_stack(size.clone());
        size
    }
}

advice! { memory_grow (
        amount: WasmValue,
        index: MemoryIndex,
        _location: Location,
    ) {
        println!("memory_grow ({amount:?}: WasmValue, {index:?}: MemoryIndex)");
        println!("memory_grow>>value_to_write>>pop_value_from_stack");
        let shadow_amount = pop_value_from_stack();
        assert_eq!(shadow_amount, amount);
        let grow_result = index.grow(amount);
        println!("memory_grow>>grow_result == {grow_result:?}");
        push_value_on_stack(grow_result.clone());
        grow_result
    }
}

#[derive(Debug)]
struct BlockType {
    origin: LabelOrigin,
    arguments: usize,
    results: usize,
}

impl BlockType {
    fn expand(&self) -> (usize, usize) {
        (self.arguments, self.results)
    }
}

// TODO: rename below to `block_blocktype_instr` and implement `blocktype_instr_end` / `exit_instr_with_label`
//       where one asserts on the blocktype's origin if possible
fn block_blocktype_instr_end(blocktype: BlockType) {
    // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-control-mathsf-block-xref-syntax-instructions-syntax-blocktype-mathit-blocktype-xref-syntax-instructions-syntax-instr-mathit-instr-ast-xref-syntax-instructions-syntax-instr-control-mathsf-end
    println!("block_blocktype_instr_end {blocktype:?}");
    // 1. Let `F` be the current frame.
    #[allow(non_snake_case)]
    let F = current_frame();
    let _ = F;
    // 2. Assert: due to validation, `expand_{F}(blocktype)` is defined.
    "skipped assertion";
    // 3. Let `[t^{m}_{1}] -> [t^{n}_{2}]` be the function type `expand_{F}(blocktype)`.
    let (/* t */ m, /* t */ n) = blocktype.expand();
    // 4. Let `L` be the label whose arity is `n` and whose continuation is the end of the block.
    #[allow(non_snake_case)]
    let L = Label::new(n, blocktype.origin);
    // 5. Assert: due to validation, there are at least `m` values on the top of the stack.
    println!("assert_at_least_n_values_on_stack>>block pre");
    assert_at_least_n_values_on_stack(m);
    // 6. Pop the values `val^{m}` from the stack.
    println!("block_pre>>pop_value_from_stack");
    let mut val_m: Vec<WasmValue> = (0..m).map(|_| pop_value_from_stack()).collect();
    // 7. Enter the block `val^{m} instr^{*}` with label `L`.
    while let Some(val) = val_m.pop() {
        push_value_on_stack(val);
    }
    enter_block_with_label(L);
}

advice! { block pre (
        block_input_count: BlockInputCount,
        block_arity: BlockArity,
        _location: Location,
    ) {
        let origin = LabelOrigin::Block;
        let arguments = block_input_count.value().try_into().unwrap();
        let results = block_arity.value().try_into().unwrap();
        block_blocktype_instr_end(BlockType { origin, arguments, results});
    }
}

advice! { block post (_location: Location) {
        println!("block post ()");
        println!("block_post>>exit_instr_with_label");
        exit_instr_with_label();
    }
}

advice! { loop_ pre (
        loop_input_count: LoopInputCount,
        loop_arity: LoopArity,
        _location: Location,
    ) {
        // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-control-mathsf-loop-xref-syntax-instructions-syntax-blocktype-mathit-blocktype-xref-syntax-instructions-syntax-instr-mathit-instr-ast-xref-syntax-instructions-syntax-instr-control-mathsf-end
        println!("loop_ pre ({loop_input_count:?}: LoopInputCount, {loop_arity:?}: LoopArity)");
        // 1. Let `F` be the current frame.
        #[allow(non_snake_case)]
        let F = current_frame();
        let _ = F;
        // 2. Assert: due to validation, `expand_{F}(blocktype)` is defined.
        "skipped assertion";
        // 3. Let `[t^{m}_{1}] -> [t^{n}_{2}]` be the function type `expand_{F}(blocktype)`.
        let m = loop_input_count.value().try_into().unwrap();
        let n = loop_arity.value();
        let _ = n;
        // 4. Let `L` be the label whose arity is `m` and whose continuation is the start of the loop.
        #[allow(non_snake_case)]
        let L = Label::new(m, LabelOrigin::Loop);
        // 5. Assert: due to validation, there are at least `m` values on the top of the stack.
        println!("assert_at_least_n_values_on_stack>>loop_ pre");
        assert_at_least_n_values_on_stack(m);
        // 6. Pop the values `val^{m}` from the stack.
        println!("loop_pre>>pop_value_from_stack");
        let mut val_m: Vec<WasmValue> = (0..m).map(|_|  pop_value_from_stack() ).collect();
        while let Some(val) = val_m.pop() {
            push_value_on_stack(val);
        }
        // 7. Enter the block `val^{m} instr^{*}` with label `L`.
        enter_block_with_label(L);
    }
}

advice! { loop_ post (_location: Location) {
        println!("loop_ post ()");
        println!("loop_post>>exit_instr_with_label");
        exit_instr_with_label();
    }
}
