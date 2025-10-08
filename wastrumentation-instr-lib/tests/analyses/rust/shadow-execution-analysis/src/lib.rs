#![allow(clippy::no_effect, clippy::wildcard_imports)]

// Author: Aäron Munsters
mod global_store;
mod shadow_memory;
mod shadow_stack;

use global_store::*;
use shadow_memory::*;
use shadow_stack::*;

use wastrumentation_rs_stdlib::*;

/// The first call on the module must come from the host.
/// As such this function prepares the shadow stack with
/// the arguments.
fn handle_host_is_caller_setup(args: &MutDynArgs, shadow_stack: &mut Stack) {
    let host_arity = usize::MAX;
    let host_function_index = usize::MAX;
    let host_arguments = vec![];
    let host_frame = Frame::new(host_arity, host_function_index, host_arguments);
    shadow_stack.push_activation_on_stack(host_frame);
    args.args_iter()
        .for_each(|arg| shadow_stack.push_value_on_stack(arg));
}

// If the function is imported, we manually handle our shadow stack
// since the body of the imported function could not reflect stack
// changes to our shadow stack datastructure
fn handle_call_to_imported(function: &WasmFunction, args: &MutDynArgs, ress: &MutDynResults) {
    SHADOW_STACK.with_borrow_mut(|shadow_stack: &mut Stack| {
        args.args_iter()
            .collect::<Vec<WasmValue>>()
            .into_iter()
            .for_each(|_| {
                let _ = shadow_stack.pop_value_from_stack();
            });
    });

    // Release runtime borrow during the function call,
    // as other `apply` hooks might be called by this call.
    function.apply();

    SHADOW_STACK.with_borrow_mut(|shadow_stack: &mut Stack| {
        ress.ress_iter()
            .collect::<Vec<WasmValue>>()
            .into_iter()
            .rev()
            .for_each(|res| shadow_stack.push_value_on_stack(res));
    });
}

#[allow(non_snake_case)]
fn enter_block_with_label(L: Label, shadow_stack: &mut Stack) {
    // https://webassembly.github.io/spec/core/exec/instructions.html#entering-xref-syntax-instructions-syntax-instr-mathit-instr-ast-with-label-l
    // 1. Push `L` to the stack.
    shadow_stack.push_label_on_stack(L);
    // 2. Jump to the start of the instruction sequence `instr^{*}`.
    "handled by VM";
}

#[allow(non_snake_case)]
fn exit_instr_with_label(shadow_stack: &mut Stack) {
    // https://webassembly.github.io/spec/core/exec/instructions.html#exiting-xref-syntax-instructions-syntax-instr-mathit-instr-ast-with-label-l
    // 1. Pop all values `val^{*}` from the top of the stack.
    let mut values: Vec<WasmValue> = vec![];
    while !matches!(shadow_stack.top_of_stack(), StackEntry::Label(_)) {
        values.push(shadow_stack.pop_value_from_stack());
    }
    // 2. Assert: due to validation, the label `L` is now on the top of the stack.
    debug_assert!(matches!(shadow_stack.top_of_stack(), StackEntry::Label(_)));
    // 3. Pop the label from the stack.
    let _ = shadow_stack.pop_label_from_stack();
    // 4. Push `val^{*}` back to the stack.
    while let Some(value) = values.pop() {
        shadow_stack.push_value_on_stack(value);
    }
    // 5. Jump to the position after the `end` of the structured control instruction associated with the label `L`.
    "handled by VM";
}

fn is_jump_flag_set() -> bool {
    JUMP_FLAG.with_borrow(|flag| *flag)
}

fn set_jump_flag_true() {
    JUMP_FLAG.with_borrow_mut(|flag| *flag = true);
}

fn set_jump_flag_false() {
    JUMP_FLAG.with_borrow_mut(|flag| *flag = false);
}

/////
// START ADVICE SPECIFICATION //
//                         /////

advice! { apply (function: WasmFunction, args: MutDynArgs, ress: MutDynResults) {
        if ShadowCallStackDepth::host_is_caller() {
            SHADOW_STACK.with_borrow_mut(|shadow_stack| {
                handle_host_is_caller_setup(&args, shadow_stack);
            });
        }

        if function.is_imported() {
            handle_call_to_imported(&function, &args, &ress);
            return;
        }

        SHADOW_STACK.with_borrow_mut(|shadow_stack| {
            let call_to_imported = function.is_imported();
            let _ = call_to_imported; // Optionally, usable information.

            ////////////////////////////////////
            // Invocation of function address //
            ////////////////////////////////////
            // https://webassembly.github.io/spec/core/exec/instructions.html#invocation-of-function-address-a
            //  1. Assert: due to validation, `S.funcs[a]` exists.
            "skipped assertion";
            //  2. Let `f` be the function instance, `S.funcs[a]`.
            let f = &function;
            //  3. Let `[t^{n}_{1}] -> [t^{m}_{1}]` be the function type `f.type`.
            let n = args.argc.try_into().unwrap();
            let m = args.resc.try_into().unwrap();
            //  4. Let `[t^{*}]` be the list of value types `f.code.locals`.
            "handled by VM";
            //  5. Let `instr^{*}` be the expression `f.code.body`.
            "handled by VM";
            //  6. Assert: due to validation, `n` values are on the top of the stack.
            shadow_stack.assert_at_least_n_values_on_stack(n);
            //  7. Pop the values `val^{n}` from the stack.
            let actual_values: Vec<WasmValue> = args.args_iter().collect();
            let shadow_values = (0..n).map(|_| shadow_stack.pop_value_from_stack()).collect::<Vec<WasmValue>>().into_iter().rev().collect();
            debug_assert_eq!(&shadow_values, &actual_values);
            //  8. Let `F` be the frame `{module f.module, locals val^{n} (default_{t})^{*}}`.
            #[allow(non_snake_case)]
            let F = Frame::new(m, f.instr_f_idx.try_into().unwrap(), shadow_values);
            //  9. Push the activation of `F` with arity `m` to the stack.
            shadow_stack.push_activation_on_stack(F);
            // 10. Let `L` be the label whose arity is `m` and whose continuation is the end of the function.
            #[allow(non_snake_case)]
            let L = Label::new(m, LabelOrigin::Function(f.instr_f_idx.try_into().unwrap()));
            // 11. Enter the instruction sequence `instr^{*}` with label `L`.
            enter_block_with_label(L, shadow_stack);
        });

        ShadowCallStackDepth::increment_call_stack_depth();
        function.apply();
        ShadowCallStackDepth::decrement_call_stack_depth();

        if is_jump_flag_set() {
            set_jump_flag_false();
            return;
        }
        // else:
        SHADOW_STACK.with_borrow_mut(|shadow_stack| {
            // Implicitly the `end` of a function is reached
            exit_instr_with_label(shadow_stack);

            ///////////////////////////////
            // Returning from a function //
            ///////////////////////////////
            // https://webassembly.github.io/spec/core/exec/instructions.html#returning-from-a-function
            // 1. Let `F` be the current frame.
            #[allow(non_snake_case)]
            let F = shadow_stack.current_frame_mut();
            let function_index = F.function_index();
            // 2. Let `n` be the arity of the activation of `F`.
            let n = ress.resc.try_into().unwrap();
            // 3. Assert: due to validation, there are `n` values on the top of the stack.
            shadow_stack.assert_at_least_n_values_on_stack(n);
            // 4. Pop the results `val^{n}` from the stack.
            let mut values: Vec<WasmValue> = (0..n).map(|_| shadow_stack.pop_value_from_stack()).collect();
            // 5. Assert: due to validation, the frame `F` is now on the top of the stack.
            let StackEntry::Frame(top_of_stack_frame) = shadow_stack.top_of_stack() else {
                panic!();
            };
            debug_assert!(top_of_stack_frame.function_index() == function_index);
            // 6. Pop the frame from the stack.
            #[allow(non_snake_case)]
            let _ = shadow_stack.pop_frame_from_stack();
            // 7. Push `val^{n}` back to the stack.
            while let Some(value) = values.pop() {
                shadow_stack.push_value_on_stack(value);
            }
            // 8. Jump to the instruction after the original call.
            "handled by VM";
        });
    }
}

// https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-control-mathsf-br-l
fn br_with(l: usize, shadow_stack: &mut Stack) {
    // 1. Assert: due to validation, the stack contains at least `l + 1` labels.
    #[allow(clippy::int_plus_one)]
    {
        debug_assert!(shadow_stack.stack_label_count() >= l + 1);
    }
    // 2. Let `L` be the `l`-th label appearing on the stack, starting from the top and counting from zero.
    #[allow(non_snake_case)]
    let L = shadow_stack.lth_label_on_stack_starting_from_top_counting_from_zero(l);
    #[allow(non_snake_case)]
    let L_origin = *L.origin();
    // 3. Let `n` be the arity of `L`.
    let n = L.arity();
    // 4. Assert: due to validation, there are at least `n` values on the top of the stack.
    shadow_stack.assert_at_least_n_values_on_stack(n);
    // 5. Pop the values `val^{n}` from the stack.
    let mut values: Vec<WasmValue> = shadow_stack.pop_values_from_stack(n);
    // 6. Repeat `l + 1` times:
    #[allow(clippy::range_plus_one)]
    for _ in 0..(l + 1) {
        // a. While the top of the stack is a value, do:
        while matches!(shadow_stack.top_of_stack(), StackEntry::Value(_)) {
            // i. Pop the value from the stack.
            let _popped: WasmValue = shadow_stack.pop_value_from_stack();
        }
        // b. Assert: due to validation, the top of the stack now is a label.
        debug_assert!(matches!(shadow_stack.top_of_stack(), StackEntry::Label(_)));
        // c. Pop the label from the stack.
        let _popped = shadow_stack.pop_label_from_stack();
    }
    // 7. Push the values `val^{n}` to the stack.
    while let Some(value) = values.pop() {
        shadow_stack.push_value_on_stack(value);
    }
    // 8. Jump to the continuation of `L`.
    "taken care of by hook termination of caller";
    if matches!(L_origin, LabelOrigin::Function(_)) {
        set_jump_flag_true();
    }
}

advice! { if_then_else (
        path_continuation: PathContinuation,
        if_then_else_input_c: IfThenElseInputCount,
        if_then_else_arity: IfThenElseArity,
        _location: Location,
    ) {
        SHADOW_STACK.with_borrow_mut(|shadow_stack| {
            let arguments = if_then_else_input_c.value().try_into().unwrap();
            let results = if_then_else_arity.value().try_into().unwrap();
            let if_then_else_block_type = BlockType { origin: LabelOrigin::If, arguments, results };
            // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-control-mathsf-if-xref-syntax-instructions-syntax-blocktype-mathit-blocktype-xref-syntax-instructions-syntax-instr-mathit-instr-1-ast-xref-syntax-instructions-syntax-instr-control-mathsf-else-xref-syntax-instructions-syntax-instr-mathit-instr-2-ast-xref-syntax-instructions-syntax-instr-control-mathsf-end
            // 1. Assert: due to validation, a value of value type `i32` is on the top of the stack.
            debug_assert!(matches!(shadow_stack.top_of_stack(), &StackEntry::Value(WasmValue::I32(_))));
            // 2. Pop the value `i32.const c` from the stack.
            let c = path_continuation;
            let _shadow_c = shadow_stack.pop_value_from_stack();
            // 3. If `c` is non-zero, then:
            if c.is_then() {
                // a. Execute the block instruction `block blocktype instr^{*}_{1} end`.
                block_blocktype_instr_end(&if_then_else_block_type, shadow_stack);
                c
            // 4. Else:
            } else {
                // a. Execute the block instruction `block blocktype instr^{*}_{2} end`.
                block_blocktype_instr_end(&if_then_else_block_type, shadow_stack);
                c
            }
        })
    }
}

advice! { if_then (
        path_continuation: PathContinuation,
        if_then_input_c: IfThenInputCount,
        if_then_arity: IfThenArity,
        _location: Location,
    ) {
        SHADOW_STACK.with_borrow_mut(|shadow_stack| {
            let arguments = if_then_input_c.value().try_into().unwrap();
            let results = if_then_arity.value().try_into().unwrap();
            let if_then_block_type = BlockType { origin: LabelOrigin::If, arguments, results };
            // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-control-mathsf-if-xref-syntax-instructions-syntax-blocktype-mathit-blocktype-xref-syntax-instructions-syntax-instr-mathit-instr-1-ast-xref-syntax-instructions-syntax-instr-control-mathsf-else-xref-syntax-instructions-syntax-instr-mathit-instr-2-ast-xref-syntax-instructions-syntax-instr-control-mathsf-end
            // 1. Assert: due to validation, a value of value type `i32` is on the top of the stack.
            debug_assert!(matches!(shadow_stack.top_of_stack(), &StackEntry::Value(WasmValue::I32(_))));
            // 2. Pop the value `i32.const c` from the stack.
            let c = path_continuation;
            let shadow_c = shadow_stack.pop_value_from_stack();
            debug_assert_eq!(shadow_c, WasmValue::from(c.value()));
            // If `c` is non-zero, then:
            if c.is_then() {
                // a. Execute the block instruction `block blocktype instr^{*}_{1} end`.
                block_blocktype_instr_end(&if_then_block_type, shadow_stack);
                c
            // 4. Else:
            } else {
                // a. Execute the block instruction `block blocktype instr^{*}_{2} end`.
                block_blocktype_instr_end(&if_then_block_type, shadow_stack);
                c
            }
        })
    }
}

advice! { if_then_else_post (_location: Location) {
        SHADOW_STACK.with_borrow_mut(|shadow_stack| {
            exit_instr_with_label(shadow_stack);
        });
    }
}

advice! { if_then_post (_location: Location) {
        SHADOW_STACK.with_borrow_mut(|shadow_stack| {
            exit_instr_with_label(shadow_stack);
        });
    }
}

advice! { br (branch_target_label: BranchTargetLabel, _location: Location) {
        SHADOW_STACK.with_borrow_mut(|shadow_stack| {
            // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-control-mathsf-br-l
            br_with(branch_target_label.label().try_into().unwrap(), shadow_stack);
        });
    }
}

advice! { br_if (
        path_continuation: ParameterBrIfCondition,
        target_label: ParameterBrIfLabel,
        _location: Location,
    ) {
        SHADOW_STACK.with_borrow_mut(|shadow_stack| {
            // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-control-mathsf-br-if-l
            // 1. Assert: due to validation, a value of value type `i32` is on the top of the stack.
            debug_assert!(matches!(shadow_stack.top_of_stack(), &StackEntry::Value(WasmValue::I32(_))));
            // 2. Pop the value `i32.const c` from the stack.
            let c = path_continuation;
            let shadow_c = shadow_stack.pop_value_from_stack();
            // 3. If `c` is non-zero, then:
            if c.is_then() {
                // a. Execute the instruction `br l`.
                debug_assert!(shadow_c.as_wasm_bool());
                br_with(target_label.label().try_into().unwrap(), shadow_stack); }
            // 4. Else:
            else {
                debug_assert!(!shadow_c.as_wasm_bool());
                // a. Do nothing.
            }
            c
        })
    }
}

advice! { br_table (
        branch_table_target: BranchTableTarget,
        branch_table_effective: BranchTableEffective,
        branch_table_default: BranchTableDefault,
        _location: Location,
    ) {
        SHADOW_STACK.with_borrow_mut(|shadow_stack| {
            // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-control-mathsf-br-table-l-ast-l-n
            let _ = branch_table_default;
            // 1. Assert: due to validation, a value of value type `i32` is on the top of the stack.
            debug_assert!(matches!(shadow_stack.top_of_stack(), &StackEntry::Value(WasmValue::I32(_))));
            // 2. Pop the value `i32.const i` from the stack.
            let i: BranchTableTarget = branch_table_target;
            let _shadow_i = shadow_stack.pop_value_from_stack();
            // 3. If `i` is smaller than the length of `l*`, then:
            //      a. Let  `l_{i}` be the label `l^{*}[i]`.
            //      b. Execute the instruction `br l_{i}`.
            // 4. Else:
            //      a. Execute the instruction  `br l_{N}`.
            br_with(branch_table_effective.label().try_into().unwrap(), shadow_stack);
            i
        })
    }
}

advice! { select (path_continuation: PathContinuation, _location: Location) {
        SHADOW_STACK.with_borrow_mut(|shadow_stack| {
            // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-parametric-mathsf-select-t-ast
            // Assert: due to validation, a value of value type `i32` is on the top of the stack.
            debug_assert!(matches!(shadow_stack.top_of_stack(), &StackEntry::Value(WasmValue::I32(_))));
            // Pop the value `i32.const c` from the stack.
            let _shadow_c = shadow_stack.pop_value_from_stack();
            // Assert: due to validation, two more values (of the same value type) are on the top of the stack.
            let (v2, v1) = shadow_stack.top_two_values_of_stack();
            debug_assert_eq!(v2.type_(), v1.type_());
            // Pop the value `val_{2}` from the stack.
            let val_2 = shadow_stack.pop_value_from_stack();
            // Pop the value `val_{1}` from the stack.
            let val_1 = shadow_stack.pop_value_from_stack();
            // If `c` is not `0`, then:
            if path_continuation.is_then() {
                // Push the value `val_{1}` back to the stack.
                shadow_stack.push_value_on_stack(val_1);
            // Else:
            } else {
                // Push the value `val_{2}` back to the stack.
                shadow_stack.push_value_on_stack(val_2);
            }
            path_continuation
        })
    }
}

advice! { call_indirect pre (
        target_func: FunctionTableIndex,
        _func_table_ident: FunctionTable,
        _location: Location,
    ) {
        SHADOW_STACK.with_borrow_mut(|shadow_stack| {

            let FunctionTableIndex(i) = target_func;

            // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-control-mathsf-call-indirect-x-y
            // 01. Let `F` be the current frame.
            #[allow(non_snake_case)]
            let F = shadow_stack.current_frame_mut();
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
            debug_assert!(matches!(shadow_stack.top_of_stack(), StackEntry::Value(WasmValue::I32(_))));
            // 09. Pop the value `i32.const i` from the stack.
            let shadow_i = shadow_stack.pop_value_from_stack();
            debug_assert_eq!(shadow_i, i.into());
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
        })
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

advice! { unary (unop: UnaryOperator, c_1: WasmValue, _location: Location) {
        SHADOW_STACK.with_borrow_mut(|shadow_stack| {
            // https://webassembly.github.io/spec/core/exec/instructions.html#t-mathsf-xref-syntax-instructions-syntax-unop-mathit-unop
            // 1. Assert: due to validation, a value of value type `t` is on the top of the stack.
            debug_assert!(matches!(shadow_stack.top_of_stack(), StackEntry::Value(_)));
            // 2. Pop the value `t.const c_{1}` from the stack.
            let shadow_c_1 = shadow_stack.pop_value_from_stack();
            debug_assert_eq!(shadow_c_1, c_1);
            // 3. If `unop_{t}(c_{1})` is defined, then:
                // a. Let `c` be a possible result of computing `unop_{t}(c_{1})`.
                let c = unop.apply(c_1);
                // b. Push the value `t.const c` to the stack.
                shadow_stack.push_value_on_stack(c.clone());
            // 4. Else:
                "skipped operation";
                // a. Trap.
            c
        })
    }
}

advice! { binary (
        binop: BinaryOperator,
        c_1: WasmValue,
        c_2: WasmValue,
        _location: Location,
    ) {
        SHADOW_STACK.with_borrow_mut(|shadow_stack| {
            // 1. Assert: due to validation, two values of value type `t` are on the top of the stack.
            "handled by validation";
            // 2. Pop the value `t.const c_{2}` from the stack.
            let shadow_c_2 = shadow_stack.pop_value_from_stack();
            debug_assert_eq!(shadow_c_2, c_2);
            // 3. Pop the value `t.const c_{1}` from the stack.
            let shadow_c_1 = shadow_stack.pop_value_from_stack();
            debug_assert_eq!(shadow_c_1, c_1);
            // 4. If `binop_{t}(c_{1},c_{2})` is defined, then:
                // a. Let `c` be a possible result of computing `binop_{t}(c_{1},c_{2})`.
                let c = binop.apply(c_1, c_2);
                // b. Push the value `t.const c` to the stack.
                shadow_stack.push_value_on_stack(c.clone());
            // 5. Else:
                "handled by VM";
                // a. Trap.
            c
        })
    }
}

advice! { drop (_location: Location) {
        SHADOW_STACK.with_borrow_mut(|shadow_stack| {
            // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-parametric-mathsf-drop
            // Assert: due to validation, a value is on the top of the stack.
            debug_assert!(matches!(shadow_stack.top_of_stack(), StackEntry::Value(_)));
            // Pop the value `val` from the stack.
            let _ = shadow_stack.pop_value_from_stack();
        });
    }
}

// https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-control-mathsf-return
advice! { return_ (_location: Location) {
        SHADOW_STACK.with_borrow_mut(|shadow_stack| {
            set_jump_flag_true();
            //  1. Let {F} be the current frame.
            #[allow(non_snake_case)]
            let F = shadow_stack.current_frame_mut();
            let f_identifier = F.function_index();
            //  2. Let {n} be the arity of {F}.
            let n = F.arity();
            //  3. Assert: due to validation, there are at least {n} values on the top of the stack.
            shadow_stack.assert_at_least_n_values_on_stack(n);
            //  4. Pop the results {val^n} from the stack.
            let mut results: Vec<WasmValue> = (0..n).map(|_| shadow_stack.pop_value_from_stack()).collect();
            //  5. Assert: due to validation, the stack contains at least one frame.
            "skipped assertion";
            //  6. While the top of the stack is not a frame, do:
            while !matches!(shadow_stack.top_of_stack(), StackEntry::Frame(_)) {
                // a. Pop the top element from the stack.
                let _popped = shadow_stack.pop_stack();
            }
            //  7. Assert: the top of the stack is the frame {F}.
            let StackEntry::Frame(top_of_stack_frame) = shadow_stack.top_of_stack() else {
                panic!();
            };
            debug_assert!(top_of_stack_frame.function_index() == f_identifier);
            //  8. Pop the frame from the stack.
            #[allow(non_snake_case)]
            let F = shadow_stack.pop_frame_from_stack();
            let _ = F;
            //  9. Push {val^n} to the stack.
            while let Some(result) = results.pop() {
                shadow_stack.push_value_on_stack(result);
            }
            // for result in results { push_value_on_stack(result); }
            // 10. Jump to the instruction after the original call that pushed the frame.
            "handled by VM";
        });
    }
}

advice! { const_ (value: WasmValue, _location: Location) {
        SHADOW_STACK.with_borrow_mut(|shadow_stack| {
            // https://webassembly.github.io/spec/core/exec/instructions.html#t-mathsf-xref-syntax-instructions-syntax-instr-numeric-mathsf-const-c
            // 1. Push the value `t.const c` to the stack.
            shadow_stack.push_value_on_stack(value.clone());
            value
        })
    }
}

fn local_set(x: usize, actual_value: &WasmValue, shadow_stack: &mut Stack) {
    //  https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-variable-mathsf-local-set-x
    // 1. Let `F` be the current frame.
    #[allow(non_snake_case)]
    let F = shadow_stack.current_frame_mut();
    // 2. Assert: due to validation, `F.locals[x]` exists.
    F.assert_local_exists(x);
    // 3. Assert: due to validation, a value is on the top of the stack.
    debug_assert!(matches!(shadow_stack.top_of_stack(), StackEntry::Value(_)));
    // 4. Pop the value `val` from the stack.
    let shadow_value = shadow_stack.pop_value_from_stack();
    debug_assert_eq!(&shadow_value, actual_value);
    // 5. Replace `F.locals[x]` with the value `val`.
    #[allow(non_snake_case)]
    let F = shadow_stack.current_frame_mut();
    F.replace_local_with(x, shadow_value);
}

advice! { local (
        value: WasmValue,
        index: LocalIndex,
        local_op: LocalOp,
        _location: Location,
    ) {
        SHADOW_STACK.with_borrow_mut(|shadow_stack| {
            let x: usize = index.value().try_into().unwrap();
            match local_op {
                LocalOp::Get => {
                    // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-variable-mathsf-local-get-x
                    // 1. Let `F` be the current frame.
                    #[allow(non_snake_case)]
                    let F = shadow_stack.current_frame_mut();
                    // 2. Assert: due to validation, `F.locals[x]` exists.
                    F.assert_local_exists(x);
                    // 3. Let `val` be the value `F.locals[x]`.
                    let shadow_val = F.get_locals(x, value.type_());
                    debug_assert_eq!(shadow_val, value);
                    // 4. Push the value `val` to the stack.
                    shadow_stack.push_value_on_stack(shadow_val);
                },
                LocalOp::Set => {
                    // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-variable-mathsf-local-set-x
                    local_set(x, &value, shadow_stack);
                },
                LocalOp::Tee => {
                    // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-variable-mathsf-local-tee-x
                    // 1. Assert: due to validation, a value is on the top of the stack.
                    debug_assert!(matches!(shadow_stack.top_of_stack(), StackEntry::Value(_)));
                    // 2. Pop the value `val` from the stack.
                    let shadow_val = shadow_stack.pop_value_from_stack();
                    // 3. Push the value `val` to the stack.
                    shadow_stack.push_value_on_stack(shadow_val.clone());
                    // 4. Push the value `val` to the stack.
                    shadow_stack.push_value_on_stack(shadow_val);
                    // 5. Execute the instruction `local.set x`.
                    local_set(x, &value, shadow_stack);
                },
            }
            value
        })
    }
}

advice! { global (
        value: WasmValue,
        index: GlobalIndex,
        global_op: GlobalOp,
        _location: Location,
    ) {
        SHADOW_STACK.with_borrow_mut(|shadow_stack| {
            let x: usize = index.value().try_into().unwrap();
            match global_op {
                GlobalOp::Get => {
                    // Let `F` be the current frame.
                    #[allow(non_snake_case)]
                    let F = shadow_stack.current_frame_mut();
                    let _ = F;
                    // Assert: due to validation, `F.module.globaladdrs[x]` exists.
                    "skipped assertion";
                    // Let `a` be the global address `F.module.globaladdrs[x]`.
                    let a = GlobalAddress::new(x);
                    GLOBAL_STORE.with_borrow_mut(|shadow_store| {
                        // Assert: due to validation, `S.globals[a]` exists.
                        shadow_store.assert_global_exists(&a);
                        // Let `glob` be the global instance `S.globals[a]`.
                        let glob = shadow_store.global(&a);
                        // Let `val` be the value `glob.value`.
                        let shadow_val = glob.value(value.type_(), &value);
                        assert_global_value(&value, &shadow_val);
                        // Push the value `val` to the stack.
                        shadow_stack.push_value_on_stack(shadow_val);
                    });
                },
                GlobalOp::Set => {
                    // Let `F` be the current frame.
                    #[allow(non_snake_case)]
                    let F = shadow_stack.current_frame_mut();
                    let _ = F;
                    // Assert: due to validation, `F.module.globaladdrs[x]` exists.
                    "skipped assertion";
                    // Let `a` be the global address `F.module.globaladdrs[x]`.
                    let a = GlobalAddress::new(x);
                    GLOBAL_STORE.with_borrow_mut(|shadow_store| {
                        // Assert: due to validation, `S.globals[a]` exists.
                        shadow_store.assert_global_exists(&a);
                        // Let `glob` be the global instance `S.globals[a]`.
                        let glob = shadow_store.global(&a);
                        // Assert: due to validation, a value is on the top of the stack.
                        debug_assert!(matches!(shadow_stack.top_of_stack(), StackEntry::Value(_)));
                        // Pop the value `val` from the stack.
                        let shadow_val = shadow_stack.pop_value_from_stack();
                        debug_assert_eq!(value, shadow_val);
                        // Replace `glob.value` with the value `val`.
                        glob.replace_value_with(shadow_val);
                    });
                },
            }
            value
        })
    }
}

advice! { load (
        store_index: LoadIndex,
        offset: LoadOffset,
        operation: LoadOperation,
        _location: Location,
    ) {
        SHADOW_STACK.with_borrow_mut(|shadow_stack| {
            // https://webassembly.github.io/spec/core/exec/instructions.html#t-mathsf-xref-syntax-instructions-syntax-instr-memory-mathsf-load-xref-syntax-instructions-syntax-memarg-mathit-memarg-and-t-mathsf-xref-syntax-instructions-syntax-instr-memory-mathsf-load-n-mathsf-xref-syntax-instructions-syntax-sx-mathit-sx-xref-syntax-instructions-syntax-memarg-mathit-memarg
            // TODO: link to Wasm spec
            // offset is a constant
            // store_index = dynamic address
            let shadow_pointer = shadow_stack.pop_value_from_stack();
            let pointer = WasmValue::from(store_index.value());
            debug_assert_eq!(pointer, shadow_pointer);
            let loaded_value = operation.perform(&store_index, &offset);
            let shadow_value = SHADOW_MEMORY.with_borrow_mut(|memory| memory.load(&shadow_pointer, &offset, operation));
            assert_shadow_memory(&loaded_value, &shadow_value);
            shadow_stack.push_value_on_stack(loaded_value.clone());
            loaded_value
        })
    }
}

advice! { store (
        store_index: StoreIndex,
        value: WasmValue,
        offset: StoreOffset,
        operation: StoreOperation,
        _location: Location,
    ) {
        SHADOW_STACK.with_borrow_mut(|shadow_stack| {
            // https://webassembly.github.io/spec/core/exec/instructions.html#t-mathsf-xref-syntax-instructions-syntax-instr-memory-mathsf-store-xref-syntax-instructions-syntax-memarg-mathit-memarg-and-t-mathsf-xref-syntax-instructions-syntax-instr-memory-mathsf-store-n-xref-syntax-instructions-syntax-memarg-mathit-memarg
            // TODO: link to Wasm spec
            // offset is a constant
            // store_index = dynamic address
            let pointer = WasmValue::from(store_index.value());
            // Value to write
            let shadow_value = shadow_stack.pop_value_from_stack();
            // Pointer
            let shadow_pointer = shadow_stack.pop_value_from_stack();
            debug_assert_eq!(pointer, shadow_pointer);
            debug_assert_eq!(value, shadow_value);
            // Perform write
            operation.perform(&store_index, &value, &offset);
            SHADOW_MEMORY.with_borrow_mut(|memory| memory.store(&shadow_pointer, &shadow_value, &offset, operation));
        });
    }
}

advice! { memory_size (
        size: WasmValue,
        index: MemoryIndex,
        _location: Location,
    ) {
        SHADOW_STACK.with_borrow_mut(|shadow_stack| {
            let _ = index;
            shadow_stack.push_value_on_stack(size.clone());
            size
        })
    }
}

advice! { memory_grow (
        amount: WasmValue,
        index: MemoryIndex,
        _location: Location,
    ) {
        SHADOW_STACK.with_borrow_mut(|shadow_stack| {
            let shadow_amount = shadow_stack.pop_value_from_stack();
            debug_assert_eq!(shadow_amount, amount);
            let grow_result = index.grow(amount);
            shadow_stack.push_value_on_stack(grow_result.clone());
            grow_result
        })
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
fn block_blocktype_instr_end(blocktype: &BlockType, shadow_stack: &mut Stack) {
    // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-control-mathsf-block-xref-syntax-instructions-syntax-blocktype-mathit-blocktype-xref-syntax-instructions-syntax-instr-mathit-instr-ast-xref-syntax-instructions-syntax-instr-control-mathsf-end
    // 1. Let `F` be the current frame.
    #[allow(non_snake_case)]
    let F = shadow_stack.current_frame_mut();
    let _ = F;
    // 2. Assert: due to validation, `expand_{F}(blocktype)` is defined.
    "skipped assertion";
    // 3. Let `[t^{m}_{1}] -> [t^{n}_{2}]` be the function type `expand_{F}(blocktype)`.
    let (/* t */ m, /* t */ n) = blocktype.expand();
    // 4. Let `L` be the label whose arity is `n` and whose continuation is the end of the block.
    #[allow(non_snake_case)]
    let L = Label::new(n, blocktype.origin);
    // 5. Assert: due to validation, there are at least `m` values on the top of the stack.
    shadow_stack.assert_at_least_n_values_on_stack(m);
    // 6. Pop the values `val^{m}` from the stack.
    let mut val_m: Vec<WasmValue> = (0..m)
        .map(|_| shadow_stack.pop_value_from_stack())
        .collect();
    // 7. Enter the block `val^{m} instr^{*}` with label `L`.
    while let Some(val) = val_m.pop() {
        shadow_stack.push_value_on_stack(val);
    }
    enter_block_with_label(L, shadow_stack);
}

advice! { block pre (block_input_count: BlockInputCount, block_arity: BlockArity, _location: Location) {
        let origin = LabelOrigin::Block;
        let arguments = block_input_count.value().try_into().unwrap();
        let results = block_arity.value().try_into().unwrap();
        SHADOW_STACK.with_borrow_mut(|shadow_stack| {
            block_blocktype_instr_end(&BlockType { origin, arguments, results}, shadow_stack);
        })
    }
}

advice! { block post (_location: Location) {
        SHADOW_STACK.with_borrow_mut(|shadow_stack|{
            exit_instr_with_label(shadow_stack);
        });
    }
}

advice! { loop_ pre (
        loop_input_count: LoopInputCount,
        loop_arity: LoopArity,
        _location: Location,
    ) {
        SHADOW_STACK.with_borrow_mut(|shadow_stack|{
            // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-control-mathsf-loop-xref-syntax-instructions-syntax-blocktype-mathit-blocktype-xref-syntax-instructions-syntax-instr-mathit-instr-ast-xref-syntax-instructions-syntax-instr-control-mathsf-end
            // 1. Let `F` be the current frame.
            #[allow(non_snake_case)]
            let F = shadow_stack.current_frame_mut();
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
            shadow_stack.assert_at_least_n_values_on_stack(m);
            // 6. Pop the values `val^{m}` from the stack.
            let mut val_m: Vec<WasmValue> = (0..m).map(|_|  shadow_stack.pop_value_from_stack() ).collect();
            while let Some(val) = val_m.pop() {
                shadow_stack.push_value_on_stack(val);
            }
            // 7. Enter the block `val^{m} instr^{*}` with label `L`.
            enter_block_with_label(L, shadow_stack);
        });
    }
}

advice! { loop_ post (_location: Location) {
        SHADOW_STACK.with_borrow_mut(|shadow_stack|{
            exit_instr_with_label(shadow_stack);
        });
    }
}
