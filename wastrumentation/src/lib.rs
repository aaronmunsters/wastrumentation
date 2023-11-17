use wasabi_wasm::Function;
use wasabi_wasm::FunctionType;
use wasabi_wasm::Idx;
use wasabi_wasm::Instr;
use wasabi_wasm::Instr::*;
use wasabi_wasm::LocalOp;
use wasabi_wasm::Module;
use wasabi_wasm::Val;
use wasabi_wasm::ValType;

mod stack_library;

pub const INSTRUMENTATION_STACK_MODULE: &str = "wastrumentation_stack";
pub const INSTRUMENTATION_ANALYSIS_MODULE: &str = "analysis";

use crate::stack_library::StackLibrary;

// TODO: tests:
// - hashing function types does not collide on similar signature
// TODO: depend on pointcut/joinpoint

pub fn instrument(module: &mut Module) {
    let pre_instrumentation_function_indices: Vec<Idx<Function>> =
        module.functions().map(|(idx, _)| idx).collect();

    // 0. GENERATE GENERIC APPLY todo! make this useful
    let generic_apply_index = module.add_function_import(
        FunctionType::new(&[ValType::I32, ValType::I32, ValType::I32], &[]),
        INSTRUMENTATION_ANALYSIS_MODULE.into(),
        "generic_apply".into(),
    );

    // 1. GENERATE IMPORTS FOR INSTRUMENTATION STACK LIBRARY
    let stack_library = StackLibrary::from_module(module, &pre_instrumentation_function_indices);

    for function_index in pre_instrumentation_function_indices {
        let target_function_type = module.function(function_index).type_;
        let target_function_locals: Vec<ValType> = module
            .function(function_index)
            .locals()
            .map(|(_, l)| l.type_)
            .collect();
        let target_function_body = module.function(function_index).code().unwrap().body.clone();
        let stack_library_for_target = stack_library.get(&target_function_type).expect("Imported");

        // 1. Generate "uninstrumented" function
        let uninstrumented_index = module.add_function(
            target_function_type,
            target_function_locals,
            target_function_body,
        );

        // 2. Generate "apply" function
        let signature_buffer_pointer_type = ValType::I32;
        let apply_type = FunctionType::new(&[signature_buffer_pointer_type], &[]);

        let local_get_stack_ptr = || Local(LocalOp::Get, Idx::from(0 as usize));
        let call_base = Call(uninstrumented_index);
        let call_stack_store_rets: Instr = Call(stack_library_for_target.ret_store_all);

        let mut apply_instructions: Vec<Instr> = Vec::new();

        apply_instructions.push(local_get_stack_ptr());

        for load_call in &stack_library_for_target.arg_load_n {
            apply_instructions.push(local_get_stack_ptr());
            apply_instructions.push(Call(*load_call))
        }

        apply_instructions.extend_from_slice(&[call_base, call_stack_store_rets, End]);

        let apply_index = module.add_function(
            apply_type,
            vec![signature_buffer_pointer_type],
            apply_instructions,
        );
        let apply_table_index = module
            .tables
            .first_mut()
            .unwrap()
            .elements
            .first_mut()
            .unwrap()
            .functions
            .len();
        module
            .tables
            .first_mut()
            .unwrap()
            .elements
            .first_mut()
            .unwrap()
            .functions
            .push(apply_index);

        // 3. Change the original function to call into apply
        let original_function = module.function_mut(function_index);
        let stack_ptr_local = original_function.add_fresh_local(ValType::I32);

        let push_args_on_stack: Vec<Instr> = target_function_type
            .inputs()
            .iter()
            .enumerate()
            .map(|(index, _int_type)| Local(LocalOp::Get, index.into()))
            .collect();
        let call_allocate = Call(stack_library_for_target.allocate);
        let local_set_stack_ptr = Local(LocalOp::Set, stack_ptr_local);
        let argc = Const(Val::I32(
            (target_function_type.inputs().len() + target_function_type.results().len()) as i32,
        ));
        let const_apply_table_index = Const(Val::I32(apply_table_index as i32));
        let local_get_stack_ptr = || Local(LocalOp::Get, stack_ptr_local);
        let call_generic_apply = Call(generic_apply_index);
        let call_free = Call(stack_library_for_target.free);

        let mut instrumented_body = Vec::new();
        instrumented_body.extend(push_args_on_stack);
        instrumented_body.extend_from_slice(&[
            call_allocate,
            local_set_stack_ptr,
            const_apply_table_index,
            argc, // todo! unused?
            local_get_stack_ptr(),
            call_generic_apply,
        ]);

        for load_call in &stack_library_for_target.ret_load_n {
            instrumented_body.push(local_get_stack_ptr());
            instrumented_body.push(Call(*load_call))
        }

        instrumented_body.extend_from_slice(&[call_free, End]);
        original_function.code_mut().unwrap().body = instrumented_body;
    }

    // 2. Generate 'call_base'
    let call_base_idx = module.add_function(
        FunctionType::new(&[ValType::I32, ValType::I32], &[]),
        vec![],
        vec![
            Local(LocalOp::Get, (1 as usize).into()),
            Local(LocalOp::Get, (0 as usize).into()),
            CallIndirect(FunctionType::new(&[ValType::I32], &[]), (0 as usize).into()),
            End,
        ],
    );

    module
        .function_mut(call_base_idx)
        .export
        .push("call_base".into());
}
