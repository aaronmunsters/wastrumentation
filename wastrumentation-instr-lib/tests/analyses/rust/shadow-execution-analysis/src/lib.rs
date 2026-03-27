#![allow(clippy::no_effect, clippy::wildcard_imports)]

// Author: Aäron Munsters
mod global_store;
mod shadow_memory;
mod shadow_stack;
mod shadow_trap_contexts;
mod shadow_traps;

pub use global_store::*;
pub use shadow_memory::*;
pub use shadow_stack::*;
pub use shadow_trap_contexts::*;

use wastrumentation_rs_stdlib::*;

pub trait ShadowMeta: Default + Clone + std::fmt::Debug + PartialEq + 'static {
    type ShadowMetaByte: Clone + Default;

    fn decompose(&self, len: usize) -> Vec<Self::ShadowMetaByte>;
    fn recompose(bytes: Vec<Self::ShadowMetaByte>) -> Self;
}

impl ShadowMeta for () {
    type ShadowMetaByte = ();

    fn decompose(&self, len: usize) -> Vec<Self::ShadowMetaByte> {
        vec![(); len]
    }

    fn recompose(bytes: Vec<Self::ShadowMetaByte>) -> Self {
        debug_assert!(bytes.iter().all(|b| *b == ()));
        ()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShadowValue<M: ShadowMeta> {
    pub value: WasmValue,
    pub meta: M,
}

impl<M: ShadowMeta> ShadowValue<M> {
    pub fn new(value: WasmValue, meta: Option<M>) -> Self {
        Self {
            value,
            meta: meta.unwrap_or_default(),
        }
    }

    pub fn with_meta(self, meta: M) -> Self {
        Self { meta, ..self }
    }
}

impl<M: ShadowMeta> From<WasmValue> for ShadowValue<M> {
    fn from(value: WasmValue) -> Self {
        Self::new(value, None)
    }
}

pub struct ShadowState<M: ShadowMeta> {
    pub stack: Stack<M>,
    pub memory: Memory<M>,
    pub globals: GlobalStore<M>,
}

impl<M: ShadowMeta> ShadowState<M> {
    pub const fn new() -> Self {
        Self {
            stack: Stack::new(),
            memory: Memory::new(),
            globals: GlobalStore::new(),
        }
    }
}

#[link(wasm_import_module = "instrumented_input")]
unsafe extern "C" {
    fn get_function_idx_by_name(hash: u32) -> i32;
    fn get_global_idx_by_name(hash: u32) -> i32;
    fn get_memory_idx_by_name(hash: u32) -> i32;
}

pub enum ExportKind { Function, Global, Memory }

pub fn fnv1a(s: &str) -> u32 {
    let mut hash: u32 = 2166136261;
    for &byte in s.as_bytes() {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(16777619);
    }
    hash
}

pub fn idx_by_name(kind: ExportKind, name: &str) -> i32 {
    let hash = fnv1a(name);
    unsafe {
        match kind {
            ExportKind::Function => get_function_idx_by_name(hash),
            ExportKind::Global   => get_global_idx_by_name(hash),
            ExportKind::Memory   => get_memory_idx_by_name(hash),
        }
    }
}


#[macro_export]
macro_rules! name_to_idx {

    // marco for individual static index variables:
    // "name_to_idx! { Function "<exported_name>" => VAR_NAME, Global "<exported_name>" => VAR_NAME, ... }"
    ($kind:ident $($name:expr => $var:ident),+) => {
        $(
            static $var: std::sync::LazyLock<i32> =
                std::sync::LazyLock::new(|| $crate::idx_by_name($crate::ExportKind::$kind, $name));
        )+
    };

    // macro for static vectors of indices:
    // "name_to_idx! { Function VAR_NAME, "<exported_name_1>", "<exported_name_2>", ... }"
    ($kind:ident $var:ident, $($name:expr),+) => {
        static $var: std::sync::LazyLock<Vec<i32>> =
            std::sync::LazyLock::new(|| vec![$( $crate::idx_by_name($crate::ExportKind::$kind, $name) ),+]);
    };
}

#[macro_export]
macro_rules! declare_shadow_storage {
    ($M:ty) => {
        thread_local! {
            pub static SHADOW_STATE: ::std::cell::RefCell<ShadowState<$M>> = const { ::std::cell::RefCell::new(ShadowState::new()) };
        }
    };
}

#[macro_export]
macro_rules! shadow_execution {
    ($M:ty) => {

    declare_shadow_storage!($M);
    declare_shadow_traps!($M);


    /// The first call on the module must come from the host.
    /// As such this function prepares the shadow stack with
    /// the arguments.
    fn handle_host_is_caller_setup(args: &MutDynArgs, shadow_stack: &mut Stack<$M>) {
        let host_arity = usize::MAX;
        let host_function_index = usize::MAX;
        let host_arguments = vec![];
        let host_frame = Frame::new(host_arity, host_function_index, host_arguments);
        shadow_stack.push_activation_on_stack(host_frame);
        args.args_iter()
            .for_each(|arg| shadow_stack.push_value_on_stack(ShadowValue::<$M>::from(arg)));
    }

    // If the function is imported, we manually handle our shadow stack
    // since the body of the imported function could not reflect stack
    // changes to our shadow stack datastructure
    fn handle_call_to_imported(function: &WasmFunction, args: &MutDynArgs, ress: &MutDynResults) {

        SHADOW_STATE.with_borrow_mut(|state| {
            let arg_count = args.args_iter().count();
            // [shadow trap call]: call_to_imported_before
            unsafe { shadow_traps::call_to_imported_before(&mut CallToImportedBeforeTrapContext { state, arg_count }) };
        });

        SHADOW_STATE.with_borrow_mut(|state| {
            args.args_iter()
                .for_each(|_| { let _ = state.stack.pop_value_from_stack(); });
        });

        // Release runtime borrow during the function call,
        // as other `apply` hooks might be called by this call.
        function.apply();

        SHADOW_STATE.with_borrow_mut(|state| {
            ress.ress_iter()
                .collect::<Vec<WasmValue>>()
                .into_iter()
                .rev()
                .for_each(|res| state.stack.push_value_on_stack(ShadowValue::<$M>::from(res)));

            let result_count = ress.ress_iter().count();
            // [shadow trap call]: call_to_imported_after
            unsafe { shadow_traps::call_to_imported_after(&mut CallToImportedAfterTrapContext { state, result_count }) };
        });
    }


    #[allow(non_snake_case)]
    fn enter_block_with_label_and_values(L: Label, values: Vec<ShadowValue<$M>>, shadow_stack: &mut Stack<$M>) {
        // https://webassembly.github.io/spec/core/exec/instructions.html#entering-xref-syntax-instructions-syntax-instr-mathit-instr-ast-with-label-l-and-values-xref-exec-runtime-syntax-val-mathit-val-ast
        // 1. Push `L` to the stack.
        shadow_stack.push_label_on_stack(L);
        // 2. Push the values `val^{*}` to the stack.
        shadow_stack.push_values_on_stack(values);
        // 2. Jump to the start of the instruction sequence `instr^{*}`.
        "handled by VM";
    }

    #[allow(non_snake_case)]
    fn exit_instr_with_label(shadow_stack: &mut Stack<$M>) {
        // https://webassembly.github.io/spec/core/exec/instructions.html#exiting-xref-syntax-instructions-syntax-instr-mathit-instr-ast-with-label-l
        // 1. Pop all values `val^{*}` from the top of the stack.
        let mut values: Vec<ShadowValue<$M>> = vec![];
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

    // https://webassembly.github.io/spec/core/exec/instructions.html#function-calls
    advice! { apply (function: WasmFunction, args: MutDynArgs, ress: MutDynResults) {

        if ShadowCallStackDepth::host_is_caller() {
            SHADOW_STATE.with_borrow_mut(|state| {
                handle_host_is_caller_setup(&args, &mut state.stack);
            });
        }

        if function.is_imported() {
            handle_call_to_imported(&function, &args, &ress);
            return;
        }

        SHADOW_STATE.with_borrow_mut(|state| {
            // [shadow trap call]: apply_before
            unsafe { shadow_traps::apply_before(&mut ApplyBeforeTrapContext { state, function: &function, args: &args, results: &ress }) };

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
            state.stack.assert_at_least_n_values_on_stack(n);
            //  7. Pop the values `val^{n}` from the stack.
            let actual_values: Vec<WasmValue> = args.args_iter().collect();
            let lifted_actual_values: Vec<ShadowValue<$M>> = actual_values.into_iter().map(ShadowValue::<$M>::from).collect();
            let shadow_values = (0..n).map(|_| state.stack.pop_value_from_stack()).collect::<Vec<ShadowValue<$M>>>().into_iter().rev().collect();
            debug_assert_eq!(&shadow_values, &lifted_actual_values);
            //  8. Let `F` be the frame `{module f.module, locals val^{n} (default_{t})^{*}}`.
            #[allow(non_snake_case)]
            let F = Frame::new(m, f.instr_f_idx.try_into().unwrap(), shadow_values);
            //  9. Push the activation of `F` with arity `m` to the stack.
            state.stack.push_activation_on_stack(F);
            // 10. Let `L` be the label whose arity is `m` and whose continuation is the end of the function.
            #[allow(non_snake_case)]
            let L = Label::new(m, LabelOrigin::Function(f.instr_f_idx.try_into().unwrap()));
            // 10. Enter the instruction sequence `instr^{*}` with label `L` and no values.
            enter_block_with_label_and_values(L, vec![], &mut state.stack);
        });

        ShadowCallStackDepth::increment_call_stack_depth();
        function.apply();
        ShadowCallStackDepth::decrement_call_stack_depth();

        if is_jump_flag_set() {
            set_jump_flag_false();
            return;
        }

        // else:
        SHADOW_STATE.with_borrow_mut(|state| {
            // Implicitly the `end` of a function is reached
            exit_instr_with_label(&mut state.stack);

            ///////////////////////////////
            // Returning from a function //
            ///////////////////////////////
            // https://webassembly.github.io/spec/core/exec/instructions.html#returning-from-a-function
            // 1. Let `F` be the current frame.
            #[allow(non_snake_case)]
            let F = state.stack.current_frame_mut();
            let function_index = F.function_index();
            // 2. Let `n` be the arity of the activation of `F`.
            let n = ress.resc.try_into().unwrap();
            // 3. Assert: due to validation, there are `n` values on the top of the stack.
            state.stack.assert_at_least_n_values_on_stack(n);
            // 4. Pop the results `val^{n}` from the stack.
            let mut values: Vec<ShadowValue<$M>> = (0..n).map(|_| state.stack.pop_value_from_stack()).collect();
            // 5. Assert: due to validation, the frame `F` is now on the top of the stack.
            let StackEntry::Frame(top_of_stack_frame) = state.stack.top_of_stack() else {
                panic!();
            };
            debug_assert!(top_of_stack_frame.function_index() == function_index);
            // 6. Pop the frame from the stack.
            #[allow(non_snake_case)]
            let _ = state.stack.pop_frame_from_stack();
            // 7. Push `val^{n}` back to the stack.
            while let Some(value) = values.pop() {
                state.stack.push_value_on_stack(value);
            }
            // 8. Jump to the instruction after the original call.
            "handled by VM";

            // [shadow trap call]: apply_after
            unsafe { shadow_traps::apply_after(&mut ApplyAfterTrapContext { state, function: &function, args: &args, results: &ress }) };
        });
    }}

    // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-control-mathsf-br-l
    fn br_with(l: usize, shadow_stack: &mut Stack<$M>) {
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
        let mut values: Vec<ShadowValue<$M>> = shadow_stack.pop_values_from_stack(n);
        // 6. Repeat `l + 1` times:
        #[allow(clippy::range_plus_one)]
        for _ in 0..(l + 1) {
            // a. While the top of the stack is a value, do:
            while matches!(shadow_stack.top_of_stack(), StackEntry::Value(_)) {
                // i. Pop the value from the stack.
                let _popped: ShadowValue<$M> = shadow_stack.pop_value_from_stack();
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
        // TODO: add shadow trap call
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

        SHADOW_STATE.with_borrow_mut(|state| {
            // [shadow trap call]: if_then_else_before
            unsafe { shadow_traps::if_then_else_before(&mut IfThenElseTrapContext {
                state, path: &path_continuation, input_count: &if_then_else_input_c, arity: &if_then_else_arity, location: &_location,
            }) };

            let arguments = if_then_else_input_c.value().try_into().unwrap();
            let results = if_then_else_arity.value().try_into().unwrap();
            let if_then_else_block_type = BlockType { origin: LabelOrigin::If, arguments, results };
            // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-control-mathsf-if-xref-syntax-instructions-syntax-blocktype-mathit-blocktype-xref-syntax-instructions-syntax-instr-mathit-instr-1-ast-xref-syntax-instructions-syntax-instr-control-mathsf-else-xref-syntax-instructions-syntax-instr-mathit-instr-2-ast-xref-syntax-instructions-syntax-instr-control-mathsf-end
            // 1. Assert: due to validation, a value of value type `i32` is on the top of the stack.
            debug_assert!(matches!(state.stack.top_of_stack(), &StackEntry::Value(ShadowValue { value: WasmValue::I32(..), .. })));
            // 2. Pop the value `i32.const c` from the stack.
            let c = path_continuation;
            let _shadow_c = state.stack.pop_value_from_stack();
            // 3. If `c` is non-zero, then:
            if c.is_then() {
                // a. Execute the block instruction `block blocktype instr^{*}_{1} end`.
                block_blocktype_instr_end(&if_then_else_block_type, &mut state.stack);
                // [shadow trap call]: if_then_else_after
                unsafe { shadow_traps::if_then_else_after(&mut IfThenElseTrapContext {
                    state, path: &c, input_count: &if_then_else_input_c, arity: &if_then_else_arity, location: &_location,
                }) };
                c
            // 4. Else:
            } else {
                // a. Execute the block instruction `block blocktype instr^{*}_{2} end`.
                block_blocktype_instr_end(&if_then_else_block_type, &mut state.stack);
                // [shadow trap call]: if_then_else_after
                unsafe { shadow_traps::if_then_else_after(&mut IfThenElseTrapContext {
                    state, path: &c, input_count: &if_then_else_input_c, arity: &if_then_else_arity, location: &_location,
                }) };
                c
            }
        })
    }}

    advice! { if_then (
            path_continuation: PathContinuation,
            if_then_input_c: IfThenInputCount,
            if_then_arity: IfThenArity,
            _location: Location,
        ) {

        SHADOW_STATE.with_borrow_mut(|state| {
            // [shadow trap call]: if_then_before
            unsafe { shadow_traps::if_then_before(&mut IfThenTrapContext {
                state, path: &path_continuation, input_count: &if_then_input_c, arity: &if_then_arity, location: &_location,
            }) };

            let arguments = if_then_input_c.value().try_into().unwrap();
            let results = if_then_arity.value().try_into().unwrap();
            let if_then_block_type = BlockType { origin: LabelOrigin::If, arguments, results };
            // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-control-mathsf-if-xref-syntax-instructions-syntax-blocktype-mathit-blocktype-xref-syntax-instructions-syntax-instr-mathit-instr-1-ast-xref-syntax-instructions-syntax-instr-control-mathsf-else-xref-syntax-instructions-syntax-instr-mathit-instr-2-ast-xref-syntax-instructions-syntax-instr-control-mathsf-end
            // 1. Assert: due to validation, a value of value type `i32` is on the top of the stack.
            debug_assert!(matches!(state.stack.top_of_stack(), &StackEntry::Value(ShadowValue { value: WasmValue::I32(..), .. })));
            // 2. Pop the value `i32.const c` from the stack.
            let c = path_continuation;
            let shadow_c = state.stack.pop_value_from_stack();
            debug_assert_eq!(shadow_c.value, WasmValue::from(c.value()));
            // If `c` is non-zero, then:
            if c.is_then() {
                // a. Execute the block instruction `block blocktype instr^{*}_{1} end`.
                block_blocktype_instr_end(&if_then_block_type, &mut state.stack);
                // [shadow trap call]: if_then_after
                unsafe { shadow_traps::if_then_after(&mut IfThenTrapContext {
                    state, path: &c, input_count: &if_then_input_c, arity: &if_then_arity, location: &_location,
                }) };
                c
            // 4. Else:
            } else {
                // a. Execute the block instruction `block blocktype instr^{*}_{2} end`.
                block_blocktype_instr_end(&if_then_block_type, &mut state.stack);
                // [shadow trap call]: if_then_after
                unsafe { shadow_traps::if_then_after(&mut IfThenTrapContext {
                    state, path: &c, input_count: &if_then_input_c, arity: &if_then_arity, location: &_location,
                }) };
                c
            }
        })
    }}

    advice! { if_then_else_post (_location: Location) {

        SHADOW_STATE.with_borrow_mut(|state| {
            // [shadow trap call]: if_then_else_post_before
            unsafe { shadow_traps::if_then_else_post_before(&mut IfThenElsePostTrapContext { state, location: &_location }) };

            exit_instr_with_label(&mut state.stack);

            // [shadow trap call]: if_then_else_post_after
            unsafe { shadow_traps::if_then_else_post_after(&mut IfThenElsePostTrapContext { state, location: &_location }) };
        });
    }}

    advice! { if_then_post (_location: Location) {

        SHADOW_STATE.with_borrow_mut(|state| {
            // [shadow trap call]: if_then_post_before
            unsafe { shadow_traps::if_then_post_before(&mut IfThenPostTrapContext { state, location: &_location }) };

            exit_instr_with_label(&mut state.stack);

            // [shadow trap call]: if_then_post_after
            unsafe { shadow_traps::if_then_post_after(&mut IfThenPostTrapContext { state, location: &_location }) };
        });
    }}

    advice! { br (branch_target_label: BranchTargetLabel, _location: Location) {

        SHADOW_STATE.with_borrow_mut(|state| {
            // [shadow trap call]: br_before
            unsafe { shadow_traps::br_before(&mut BrTrapContext { state, target: &branch_target_label, location: &_location }) };

            // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-control-mathsf-br-l
            br_with(branch_target_label.label().try_into().unwrap(), &mut state.stack);

            // [shadow trap call]: br_after
            unsafe { shadow_traps::br_after(&mut BrTrapContext { state, target: &branch_target_label, location: &_location }) };
        });
    }}

    advice! { br_if (
            path_continuation: ParameterBrIfCondition,
            target_label: ParameterBrIfLabel,
            _location: Location,
        ) {

        SHADOW_STATE.with_borrow_mut(|shadow_stack| {
            // [shadow trap call]: br_if_before
            unsafe { shadow_traps::br_if_before(&mut BrIfTrapContext {
                state: shadow_stack, condition: &path_continuation, target: &target_label, location: &_location,
            }) };

            // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-control-mathsf-br-if-l
            // 1. Assert: due to validation, a value of value type `i32` is on the top of the stack.
            debug_assert!(matches!(shadow_stack.stack.top_of_stack(), StackEntry::Value(ShadowValue { value: WasmValue::I32(..), .. })));
            // 2. Pop the value `i32.const c` from the stack.
            let c = path_continuation;
            let shadow_c = shadow_stack.stack.pop_value_from_stack();
            // 3. If `c` is non-zero, then:
            if c.is_then() {
                // a. Execute the instruction `br l`.
                debug_assert!(shadow_c.value.as_wasm_bool());
                br_with(target_label.label().try_into().unwrap(), &mut shadow_stack.stack);
            }
            // 4. Else:
            else {
                debug_assert!(!shadow_c.value.as_wasm_bool());
                // a. Do nothing.
            }

            // [shadow trap call]: br_if_after
            unsafe { shadow_traps::br_if_after(&mut BrIfTrapContext {
                state: shadow_stack, condition: &c, target: &target_label, location: &_location,
            }) };

            c
        })
    }}

    advice! { br_table (
            branch_table_target: BranchTableTarget,
            branch_table_effective: BranchTableEffective,
            branch_table_default: BranchTableDefault,
            _location: Location,
        ) {

        SHADOW_STATE.with_borrow_mut(|state| {
            // [shadow trap call]: br_table_before
            unsafe { shadow_traps::br_table_before(&mut BrTableTrapContext {
                state, target: &branch_table_target, effective: &branch_table_effective, default: &branch_table_default, location: &_location,
            }) };

            // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-control-mathsf-br-table-l-ast-l-n
            let _ = branch_table_default;
            // 1. Assert: due to validation, a value of value type `i32` is on the top of the stack.
            debug_assert!(matches!(state.stack.top_of_stack(), &StackEntry::Value(ShadowValue { value: WasmValue::I32(..), .. })));
            // 2. Pop the value `i32.const i` from the stack.
            let i: BranchTableTarget = branch_table_target;
            let _shadow_i = state.stack.pop_value_from_stack();
            // 3. If `i` is smaller than the length of `l*`, then:
            //      a. Let  `l_{i}` be the label `l^{*}[i]`.
            //      b. Execute the instruction `br l_{i}`.
            // 4. Else:
            //      a. Execute the instruction  `br l_{N}`.
            br_with(branch_table_effective.label().try_into().unwrap(), &mut state.stack);

            // [shadow trap call]: br_table_after
            unsafe { shadow_traps::br_table_after(&mut BrTableTrapContext {
                state, target: &i, effective: &branch_table_effective, default: &branch_table_default, location: &_location,
            }) };

            i
        })
    }}

    advice! { select (path_continuation: PathContinuation, _location: Location) {

        SHADOW_STATE.with_borrow_mut(|state| {
            // [shadow trap call]: select_before
            unsafe { shadow_traps::select_before(&mut SelectTrapContext { state, path: &path_continuation, location: &_location }) };

            // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-parametric-mathsf-select-t-ast
            // 1. Assert: due to validation, a value of value type `i32` is on the top of the stack.
            debug_assert!(matches!(state.stack.top_of_stack(), &StackEntry::Value(ShadowValue { value: WasmValue::I32(..), .. })));
            // 2. Pop the value `i32.const c` from the stack.
            let _shadow_c = state.stack.pop_value_from_stack();
            // 3. Assert: due to validation, two more values (of the same value type) are on the top of the stack.
            let (v2, v1) = state.stack.top_two_values_of_stack();
            debug_assert_eq!(v2.value.type_(), v1.value.type_());
            // 4. Pop the value `val_{2}` from the stack.
            let val_2 = state.stack.pop_value_from_stack();
            // 5. Pop the value `val_{1}` from the stack.
            let val_1 = state.stack.pop_value_from_stack();
            // 6. If `c` is not `0`, then:
            if path_continuation.is_then() {
                // a. Push the value `val_{1}` back to the stack.
                state.stack.push_value_on_stack(val_1);
            // Else:
            } else {
                // b. Push the value `val_{2}` back to the stack.
                state.stack.push_value_on_stack(val_2);
            }

            // [shadow trap call]: select_after
            unsafe { shadow_traps::select_after(&mut SelectTrapContext { state, path: &path_continuation, location: &_location }) };
            path_continuation
        })
    }}

    advice! { call_indirect pre (
            target_func: FunctionTableIndex,
            _func_table_ident: FunctionTable,
            _location: Location,
        ) {

        SHADOW_STATE.with_borrow_mut(|state| {
            // [shadow trap call]: call_indirect_pre
            unsafe { shadow_traps::call_indirect_pre(&mut CallIndirectPreTrapContext {
                state, target: &target_func, table: &_func_table_ident, location: &_location,
            }) };

            let FunctionTableIndex(i) = target_func;

            // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-control-mathsf-call-indirect-x-y
            // 01. Let `F` be the current frame.
            #[allow(non_snake_case)]
            let F = state.stack.current_frame_mut();
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
            debug_assert!(matches!(state.stack.top_of_stack(), StackEntry::Value(ShadowValue { value: WasmValue::I32(..), .. })));
            // 09. Pop the value `i32.const i` from the stack.
            let shadow_i = state.stack.pop_value_from_stack();
            debug_assert_eq!(shadow_i.value, i.into());
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
    }}

    advice! { call_indirect post (_target_func: FunctionTable, _location: Location) {

        SHADOW_STATE.with_borrow_mut(|state| {
            // [shadow trap call]: call_indirect_post
            unsafe { shadow_traps::call_indirect_post(&mut CallIndirectPostTrapContext { state, target: &_target_func, location: &_location }) };
        });

        "No particular semantics";
    }}

    advice! { call pre (_target_func: FunctionIndex, _location: Location) {

        SHADOW_STATE.with_borrow_mut(|state| {
            // [shadow trap call]: call_pre
            unsafe { shadow_traps::call_pre(&mut CallPreTrapContext { state, target: &_target_func, location: &_location }) };
        });

        "No particular semantics";
    }}

    advice! { call post (_target_func: FunctionIndex, _location: Location) {

        SHADOW_STATE.with_borrow_mut(|state| {
            // [shadow trap call]: call_post
            unsafe { shadow_traps::call_post(&mut CallPostTrapContext { state, target: &_target_func, location: &_location }) };
        });

        "No particular semantics";
    }}

    advice! { unary (unop: UnaryOperator, c_1: WasmValue, _location: Location) {
        SHADOW_STATE.with_borrow_mut(|state| {
            // https://webassembly.github.io/spec/core/exec/instructions.html#t-mathsf-xref-syntax-instructions-syntax-unop-mathit-unop
            // 1. Assert: due to validation, a value of value type `t` is on the top of the stack.
            debug_assert!(matches!(state.stack.top_of_stack(), StackEntry::Value(_)));
            // 2. Pop the value `t.const c_{1}` from the stack.
            let shadow_c_1 = state.stack.pop_value_from_stack();
            debug_assert_eq!(shadow_c_1.value, c_1);
            // 3. If `unop_{t}(c_{1})` is defined, then:
                // a. Let `c` be a possible result of computing `unop_{t}(c_{1})`.
                let mut c = ShadowValue::<$M>::from(unop.apply(c_1));
                // [shadow trap call]: unary
                unsafe { shadow_traps::unary(&mut UnaryTrapContext { state, op: &unop, operand: &shadow_c_1, result: &mut c, location: &_location }) };
                // b. Push the value `t.const c` to the stack.
                state.stack.push_value_on_stack(c.clone());
            // 4. Else:
                "skipped operation";
                // a. Trap.
            c.value
        })
    }}

    advice! { binary (
            binop: BinaryOperator,
            c_1: WasmValue,
            c_2: WasmValue,
            _location: Location,
        ) {
        SHADOW_STATE.with_borrow_mut(|state| {
            // 1. Assert: due to validation, two values of value type `t` are on the top of the stack.
            "handled by validation";
            // 2. Pop the value `t.const c_{2}` from the stack.
            let shadow_c_2 = state.stack.pop_value_from_stack();
            debug_assert_eq!(shadow_c_2.value, c_2);
            // 3. Pop the value `t.const c_{1}` from the stack.
            let shadow_c_1 = state.stack.pop_value_from_stack();
            debug_assert_eq!(shadow_c_1.value, c_1);
            // 4. If `binop_{t}(c_{1},c_{2})` is defined, then:
                // a. Let `c` be a possible result of computing `binop_{t}(c_{1},c_{2})`.
                let mut c = ShadowValue::<$M>::from(binop.apply(c_1, c_2));
                // [shadow trap call]: binary
                unsafe { shadow_traps::binary(&mut BinaryTrapContext { state, op: &binop, lhs: &shadow_c_1, rhs: &shadow_c_2, result: &mut c, location: &_location }) };
                // b. Push the value `t.const c` to the stack.
                state.stack.push_value_on_stack(c.clone());
            // 5. Else:
                "handled by VM";
                // a. Trap.
            c.value
        })
    }}

    advice! { drop (_location: Location) {
        SHADOW_STATE.with_borrow_mut(|state| {
            // [shadow trap call]: drop
            unsafe { shadow_traps::drop(&mut DropTrapContext { state, location: &_location }) };
            // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-parametric-mathsf-drop
            // Assert: due to validation, a value is on the top of the stack.
            debug_assert!(matches!(state.stack.top_of_stack(), StackEntry::Value(_)));
            // Pop the value `val` from the stack.
            let _ = state.stack.pop_value_from_stack();
        });
    }}

    // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-control-mathsf-return
    advice! { return_ (_location: Location) {

        SHADOW_STATE.with_borrow_mut(|state| {
            // [shadow trap call]: return_before
            unsafe { shadow_traps::return_before(&mut ReturnTrapContext { state, location: &_location }) };
        });

        SHADOW_STATE.with_borrow_mut(|state| {
            set_jump_flag_true();
            //  1. Let {F} be the current frame.
            #[allow(non_snake_case)]
            let F = state.stack.current_frame_mut();
            let f_identifier = F.function_index();
            //  2. Let {n} be the arity of {F}.
            let n = F.arity();
            //  3. Assert: due to validation, there are at least {n} values on the top of the stack.
            state.stack.assert_at_least_n_values_on_stack(n);
            //  4. Pop the results {val^n} from the stack.
            let mut results: Vec<ShadowValue<$M>> = (0..n).map(|_| state.stack.pop_value_from_stack()).collect();
            //  5. Assert: due to validation, the stack contains at least one frame.
            "skipped assertion";
            //  6. While the top of the stack is not a frame, do:
            while !matches!(state.stack.top_of_stack(), StackEntry::Frame(_)) {
                // a. Pop the top element from the stack.
                let _popped = state.stack.pop_stack();
            }
            //  7. Assert: the top of the stack is the frame {F}.
            let StackEntry::Frame(top_of_stack_frame) = state.stack.top_of_stack() else {
                panic!();
            };
            debug_assert!(top_of_stack_frame.function_index() == f_identifier);
            //  8. Pop the frame from the stack.
            #[allow(non_snake_case)]
            let F = state.stack.pop_frame_from_stack();
            let _ = F;
            //  9. Push {val^n} to the stack.
            while let Some(result) = results.pop() {
                state.stack.push_value_on_stack(result);
            }
            // 10. Jump to the instruction after the original call that pushed the frame.
            "handled by VM";
        });

        SHADOW_STATE.with_borrow_mut(|state| {
            // [shadow trap call]: return_after
            unsafe { shadow_traps::return_after(&mut ReturnTrapContext { state, location: &_location }) };
        });
    }}

    advice! { const_ (value: WasmValue, _location: Location) {

        SHADOW_STATE.with_borrow_mut(|state| {
            let mut shadow_value = ShadowValue::<$M>::from(value.clone());

            // [shadow trap call]: const_
            unsafe { shadow_traps::const_(&mut ConstTrapContext { state, value: &mut shadow_value, location: &_location }) };

            // https://webassembly.github.io/spec/core/exec/instructions.html#t-mathsf-xref-syntax-instructions-syntax-instr-numeric-mathsf-const-c
            // 1. Push the value `t.const c` to the stack.
            state.stack.push_value_on_stack(shadow_value.clone());
            value
        })
    }}

    fn local_set(x: usize, actual_value: &WasmValue, state: &mut ShadowState<$M>, index: LocalIndex, local_op: LocalOp, _location: Location) {
        //  https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-variable-mathsf-local-set-x
        // 1. Let `F` be the current frame.
        #[allow(non_snake_case)]
        let F = state.stack.current_frame_mut();
        // 2. Assert: due to validation, `F.locals[x]` exists.
        F.assert_local_exists(x);
        // 3. Assert: due to validation, a value is on the top of the stack.
        debug_assert!(matches!(state.stack.top_of_stack(), StackEntry::Value(_)));
        // 4. Pop the value `val` from the stack.
        let mut shadow_value = state.stack.pop_value_from_stack();
        debug_assert_eq!(&shadow_value.value, actual_value);
        // [shadow trap call]: local (with local_op = Set | Tee)
        unsafe { shadow_traps::local(&mut LocalTrapContext { state, value: &mut shadow_value, index: &index, op: &local_op, location: &_location }) };
        // 5. Replace `F.locals[x]` with the value `val`.
        #[allow(non_snake_case)]
        let F = state.stack.current_frame_mut();
        F.replace_local_with(x, shadow_value);
    }

    advice! { local (
            value: WasmValue,
            index: LocalIndex,
            local_op: LocalOp,
            _location: Location,
        ) {

        SHADOW_STATE.with_borrow_mut(|state| {
            let x: usize = index.value().try_into().unwrap();
            match local_op {
                LocalOp::Get => {
                    // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-variable-mathsf-local-get-x
                    // 1. Let `F` be the current frame.
                    #[allow(non_snake_case)]
                    let F = state.stack.current_frame_mut();
                    // 2. Assert: due to validation, `F.locals[x]` exists.
                    F.assert_local_exists(x);
                    // 3. Let `val` be the value `F.locals[x]`.
                    let mut shadow_val = F.get_locals(x, value.type_());
                    debug_assert_eq!(shadow_val.value, value);
                    // [shadow trap call]: local (with local_op = Get)
                    unsafe { shadow_traps::local(&mut LocalTrapContext { state, value: &mut shadow_val, index: &index, op: &local_op, location: &_location }) };
                    // 4. Push the value `val` to the stack.
                    state.stack.push_value_on_stack(shadow_val);
                },
                LocalOp::Set => {
                    // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-variable-mathsf-local-set-x
                    local_set(x, &value, state, index, local_op, _location);
                },
                LocalOp::Tee => {
                    // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-variable-mathsf-local-tee-x
                    // 1. Assert: due to validation, a value is on the top of the stack.
                    debug_assert!(matches!(state.stack.top_of_stack(), StackEntry::Value(_)));
                    // 2. Pop the value `val` from the stack.
                    let mut shadow_val = state.stack.pop_value_from_stack();
                    // 3. Push the value `val` to the stack.
                    state.stack.push_value_on_stack(shadow_val.clone());
                    // 4. Push the value `val` to the stack.
                    state.stack.push_value_on_stack(shadow_val);
                    // 5. Execute the instruction `local.set x`.
                    local_set(x, &value, state, index, local_op, _location);
                },
            }
            value
        })
    }}

    advice! { global (
            value: WasmValue,
            index: GlobalIndex,
            global_op: GlobalOp,
            _location: Location,
        ) {

        SHADOW_STATE.with_borrow_mut(|state| {
            let x: usize = index.value().try_into().unwrap();
            match global_op {
                GlobalOp::Get => {
                    // Let `F` be the current frame.
                    #[allow(non_snake_case)]
                    let F = state.stack.current_frame_mut();
                    let _ = F;
                    // Assert: due to validation, `F.module.globaladdrs[x]` exists.
                    "skipped assertion";
                    // Let `a` be the global address `F.module.globaladdrs[x]`.
                    let a = GlobalAddress::new(x);
                    // Assert: due to validation, `S.globals[a]` exists.
                    state.globals.assert_global_exists(&a);
                    // Let `glob` be the global instance `S.globals[a]`.
                    let glob = state.globals.global(&a);
                    // Let `val` be the value `glob.value`.
                    let lifted_value = ShadowValue::<$M>::from(value.clone());
                    let mut shadow_val = glob.value(value.type_(), &lifted_value);
                    assert_global_value::<$M>(&shadow_val, &lifted_value);
                    // [shadow trap call]: global (with global_op = Get)
                    unsafe { shadow_traps::global(&mut GlobalTrapContext { state, value: &mut shadow_val, index: &index, op: &global_op, location: &_location }) };
                    // Push the value `val` to the stack.
                    state.stack.push_value_on_stack(shadow_val);
                },
                GlobalOp::Set => {
                    // Let `F` be the current frame.
                    #[allow(non_snake_case)]
                    let F = state.stack.current_frame_mut();
                    let _ = F;
                    // Assert: due to validation, `F.module.globaladdrs[x]` exists.
                    "skipped assertion";
                    // Let `a` be the global address `F.module.globaladdrs[x]`.
                    let a = GlobalAddress::new(x);
                    // Assert: due to validation, `S.globals[a]` exists.
                    state.globals.assert_global_exists(&a);
                    // Assert: due to validation, a value is on the top of the stack.
                    debug_assert!(matches!(state.stack.top_of_stack(), StackEntry::Value(_)));
                    // Pop the value `val` from the stack.
                    let mut shadow_val = state.stack.pop_value_from_stack();
                    debug_assert_eq!(value, shadow_val.value);
                    // [shadow trap call]: global (with global_op = Set)
                    unsafe { shadow_traps::global(&mut GlobalTrapContext { state, value: &mut shadow_val, index: &index, op: &global_op, location: &_location }) };
                    // Let `glob` be the global instance `S.globals[a]`.
                    let glob = state.globals.global(&a);
                    // Replace `glob.value` with the value `val`.
                    glob.replace_value_with(shadow_val);
                },
            }
            value
        })
    }}

    advice! { load (
            store_index: LoadIndex,
            offset: LoadOffset,
            operation: LoadOperation,
            _location: Location,
        ) {

        SHADOW_STATE.with_borrow_mut(|state| {
            // TODO: link to Wasm spec
            // offset is a constant
            // store_index = dynamic address
            let shadow_pointer = state.stack.pop_value_from_stack();
            let pointer = WasmValue::from(store_index.value());
            debug_assert_eq!(pointer, shadow_pointer.value);
            let loaded_value = operation.perform(&store_index, &offset);
            let mut shadow_value = state.memory.load(&shadow_pointer, &offset, operation);
            assert_shadow_memory(&ShadowValue::<$M>::from(loaded_value), &shadow_value);
            // [shadow trap call]: load
            unsafe { shadow_traps::load(&mut LoadTrapContext { state, index: &store_index, offset: &offset, operation: &operation, result: &mut shadow_value, location: &_location }) };
            state.stack.push_value_on_stack(shadow_value.clone());
            shadow_value.value
        })
    }}

    advice! { store (
            store_index: StoreIndex,
            value: WasmValue,
            offset: StoreOffset,
            operation: StoreOperation,
            _location: Location,
        ) {

        SHADOW_STATE.with_borrow_mut(|state| {
            // TODO: link to Wasm spec
            // offset is a constant
            // store_index = dynamic address
            let pointer = WasmValue::from(store_index.value());
            // Value to write
            let mut shadow_value = state.stack.pop_value_from_stack();
            // Pointer
            let mut shadow_pointer = state.stack.pop_value_from_stack();
            debug_assert_eq!(pointer, shadow_pointer.value);
            debug_assert_eq!(value, shadow_value.value);
            // [shadow trap call]: store
            unsafe { shadow_traps::store(&mut StoreTrapContext { state, index: &store_index, value: &mut shadow_value, pointer: &mut shadow_pointer, offset: &offset, operation: &operation, location: &_location }) };
            // Perform write
            operation.perform(&store_index, &value, &offset);
            state.memory.store(&shadow_pointer, &shadow_value, &offset, operation);
        });
    }}

    advice! { memory_size (size: WasmValue, index: MemoryIndex, _location: Location) {

        SHADOW_STATE.with_borrow_mut(|state| {
            let mut shadow_size = ShadowValue::<$M>::from(size.clone());
            // [shadow trap call]: memory_size
            unsafe { shadow_traps::memory_size(&mut MemorySizeTrapContext { state, size: &mut shadow_size, index: &index, location: &_location }) };
            let _ = index;
            state.stack.push_value_on_stack(shadow_size.clone());
            shadow_size.value
        })
    }}

    advice! { memory_grow (amount: WasmValue, index: MemoryIndex, _location: Location) {

        SHADOW_STATE.with_borrow_mut(|state| {
            let mut shadow_amount = state.stack.pop_value_from_stack();
            debug_assert_eq!(shadow_amount.value, amount);
            // [shadow trap call]: memory_grow
            unsafe { shadow_traps::memory_grow(&mut MemoryGrowTrapContext { state, amount: &mut shadow_amount, index: &index, location: &_location }) };
            let grow_result = index.grow(shadow_amount.value);
            state.stack.push_value_on_stack(ShadowValue::<$M>::from(grow_result.clone()));
            grow_result
        })
    }}

    advice! { memory_init (_location: Location) {

        SHADOW_STATE.with_borrow_mut(|state| {
            // [shadow trap call]: memory_init
            unsafe { shadow_traps::memory_init(&mut MemoryInitTrapContext { state, location: &_location }) };
            // TODO: The shadow implementation could perhaps be implemented.
            let src = state.stack.pop_value_from_stack();
            let dst = state.stack.pop_value_from_stack();
            let len = state.stack.pop_value_from_stack();
            let _ = (src, dst, len);
        });
    }}

    advice! { memory_copy (_location: Location) {

        SHADOW_STATE.with_borrow_mut(|state| {
            // [shadow trap call]: memory_copy
            unsafe { shadow_traps::memory_copy(&mut MemoryCopyTrapContext { state, location: &_location }) };
            // TODO: The shadow implementation could perhaps be implemented.
            let src = state.stack.pop_value_from_stack();
            let dst = state.stack.pop_value_from_stack();
            let len = state.stack.pop_value_from_stack();
            let _ = (src, dst, len);
        });
    }}

    advice! { memory_fill (_location: Location) {

        SHADOW_STATE.with_borrow_mut(|state| {
            // [shadow trap call]: memory_fill
            unsafe { shadow_traps::memory_fill(&mut MemoryFillTrapContext { state, location: &_location }) };
            // TODO: The shadow implementation could perhaps be implemented.
            let src = state.stack.pop_value_from_stack();
            let dst = state.stack.pop_value_from_stack();
            let len = state.stack.pop_value_from_stack();
            let _ = (src, dst, len);
        });
    }}

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
    fn block_blocktype_instr_end(blocktype: &BlockType, shadow_stack: &mut Stack<$M>) {
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
        let val_m: Vec<_> = (0..m)
            .map(|_| shadow_stack.pop_value_from_stack())
            .rev()
            .collect();
        // 7. Enter the block `val^{m} instr^{*}` with label `L`.
        enter_block_with_label_and_values(L, val_m, shadow_stack);
    }

    advice! { block pre (
            block_input_count: BlockInputCount,
            block_arity: BlockArity,
            _location: Location,
        ) {

        SHADOW_STATE.with_borrow_mut(|state| {
            // [shadow trap call]: block_pre_before
            unsafe { shadow_traps::block_pre_before(&mut BlockPreTrapContext { state, input_count: &block_input_count, arity: &block_arity, location: &_location }) };

            let origin = LabelOrigin::Block;
            let arguments = block_input_count.value().try_into().unwrap();
            let results = block_arity.value().try_into().unwrap();
            block_blocktype_instr_end(&BlockType { origin, arguments, results }, &mut state.stack);

            // [shadow trap call]: block_pre_after
            unsafe { shadow_traps::block_pre_after(&mut BlockPreTrapContext { state, input_count: &block_input_count, arity: &block_arity, location: &_location }) };
        });
    }}

    advice! { block post (_location: Location) {

        SHADOW_STATE.with_borrow_mut(|state| {
            // [shadow trap call]: block_post_before
            unsafe { shadow_traps::block_post_before(&mut BlockPostTrapContext { state, location: &_location }) };

            exit_instr_with_label(&mut state.stack);

            // [shadow trap call]: block_post_after
            unsafe { shadow_traps::block_post_after(&mut BlockPostTrapContext { state, location: &_location }) };
        });
    }}

    advice! { loop_ pre (
            loop_input_count: LoopInputCount,
            loop_arity: LoopArity,
            _location: Location,
        ) {

        SHADOW_STATE.with_borrow_mut(|state| {
            // [shadow trap call]: loop_pre_before
            unsafe { shadow_traps::loop_pre_before(&mut LoopPreTrapContext { state, input_count: &loop_input_count, arity: &loop_arity, location: &_location }) };

            // https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-control-mathsf-loop-xref-syntax-instructions-syntax-blocktype-mathit-blocktype-xref-syntax-instructions-syntax-instr-mathit-instr-ast-xref-syntax-instructions-syntax-instr-control-mathsf-end
            // 1. Let `F` be the current frame.
            #[allow(non_snake_case)]
            let F = state.stack.current_frame_mut();
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
            state.stack.assert_at_least_n_values_on_stack(m);
            // 6. Pop the values `val^{m}` from the stack.
            let val_m: Vec<_> = (0..m).map(|_| state.stack.pop_value_from_stack()).rev().collect();
            // 7. Enter the block `val^{m} instr^{*}` with label `L`.
            enter_block_with_label_and_values(L, val_m, &mut state.stack);

            // [shadow trap call]: loop_pre_after
            unsafe { shadow_traps::loop_pre_after(&mut LoopPreTrapContext { state, input_count: &loop_input_count, arity: &loop_arity, location: &_location }) };
        });
    }}

    advice! { loop_ post (_location: Location) {

        SHADOW_STATE.with_borrow_mut(|state| {
            // [shadow trap call]: loop_post_before
            unsafe { shadow_traps::loop_post_before(&mut LoopPostTrapContext { state, location: &_location }) };

            exit_instr_with_label(&mut state.stack);

            // [shadow trap call]: loop_post_after
            unsafe { shadow_traps::loop_post_after(&mut LoopPostTrapContext { state, location: &_location }) };
        });
    }}
    };
}
