use std::collections::HashSet;

use crate::stack_library::StackLibrary;
use wasabi_wasm::Element;
use wasabi_wasm::ElementMode;
use wasabi_wasm::Function;
use wasabi_wasm::FunctionType;
use wasabi_wasm::Idx;
use wasabi_wasm::Instr;
use wasabi_wasm::Instr::{Call, CallIndirect, Const, End, Local, RefFunc};
use wasabi_wasm::Limits;
use wasabi_wasm::LocalOp;
use wasabi_wasm::Module;
use wasabi_wasm::RefType;
use wasabi_wasm::Table;
use wasabi_wasm::Val;
use wasabi_wasm::ValType;

use crate::analysis::{WasmExport, WasmImport};
use crate::AssemblyScriptProgram;

use super::FunctionTypeConvertible;

pub const INSTRUMENTATION_STACK_MODULE: &str = "wastrumentation_stack";
pub const INSTRUMENTATION_ANALYSIS_MODULE: &str = "WASP_ANALYSIS";
pub const INSTRUMENTATION_INSTRUMENTED_MODULE: &str = "instrumented_input";

#[allow(clippy::too_many_lines)]
pub fn instrument(
    module: &mut Module,
    pre_instrumentation_function_indices: &HashSet<Idx<Function>>,
    wasp_exported_generic_apply_trap: &WasmExport,
    wasp_imported_generic_apply_base: &WasmImport,
) -> AssemblyScriptProgram {
    // 0. GENERATE GENERIC APPLY
    let generic_apply_index = module.add_function_import(
        wasp_exported_generic_apply_trap.as_function_type(),
        INSTRUMENTATION_ANALYSIS_MODULE.into(),
        wasp_exported_generic_apply_trap.name.to_string(),
    );

    // 1. GENERATE IMPORTS FOR INSTRUMENTATION STACK LIBRARY
    let StackLibrary {
        assemblyscript_code,
        signature_import_links,
    } = StackLibrary::from_module(module, pre_instrumentation_function_indices);

    // 2. Generate function instrumentation functionality
    let apply_table_index = module.tables.len();
    let mut apply_table_funs = vec![];

    for function_index in pre_instrumentation_function_indices {
        let target_function_type = module.function(*function_index).type_;
        let target_function_locals: Vec<ValType> = module
            .function(*function_index)
            .locals()
            .map(|(_, l)| l.type_)
            .collect();
        let target_function_body = module
            .function(*function_index)
            .code()
            .unwrap()
            .body
            .clone();
        let stack_library_for_target = signature_import_links
            .get(&target_function_type)
            .expect("Imported");

        // 1. Generate "uninstrumented" function
        let uninstrumented_index = module.add_function(
            target_function_type,
            target_function_locals,
            target_function_body,
        );

        // 2. Generate "base apply" function
        let signature_buffer_pointer_type = ValType::I32;
        let apply_type = FunctionType::new(&[signature_buffer_pointer_type], &[]);

        let local_get_stack_ptr = || Local(LocalOp::Get, Idx::from(0_usize));
        let call_base = Call(uninstrumented_index);
        let call_stack_store_rets: Instr = Call(stack_library_for_target.ret_store_all);

        let mut apply_instructions: Vec<Instr> = Vec::new();

        apply_instructions.push(local_get_stack_ptr());

        for load_call in &stack_library_for_target.arg_load_n {
            apply_instructions.push(local_get_stack_ptr());
            apply_instructions.push(Call(*load_call));
        }

        apply_instructions.extend_from_slice(&[call_base, call_stack_store_rets, End]);

        let apply_index = module.add_function(
            apply_type,
            vec![signature_buffer_pointer_type],
            apply_instructions,
        );

        let apply_table_index = apply_table_funs.len();
        apply_table_funs.push(apply_index);

        // 3. Change the original function to call into analysis apply
        let original_function = module.function_mut(*function_index);
        let stack_ptr_local = original_function.add_fresh_local(ValType::I32);
        let stack_ptr_types_local = original_function.add_fresh_local(ValType::I32);

        let push_args_on_stack: Vec<Instr> = target_function_type
            .inputs()
            .iter()
            .enumerate()
            .map(|(index, _int_type)| Local(LocalOp::Get, index.into()))
            .collect();
        let call_allocate_values_buffer = Call(stack_library_for_target.allocate_values_buffer);
        let local_set_values_buffer_ptr = Local(LocalOp::Set, stack_ptr_local);
        let call_allocate_types_buffer = Call(stack_library_for_target.allocate_types_buffer);
        let local_set_types_buffer_ptr = Local(LocalOp::Set, stack_ptr_types_local);

        let argc = Const(Val::I32(
            i32::try_from(target_function_type.inputs().len()).unwrap(),
        ));
        let resc = Const(Val::I32(
            i32::try_from(target_function_type.results().len()).unwrap(),
        ));
        let const_apply_table_index = Const(Val::I32(i32::try_from(apply_table_index).unwrap()));
        let local_get_stack_ptr = || Local(LocalOp::Get, stack_ptr_local);
        let local_get_stack_types_ptr = Local(LocalOp::Get, stack_ptr_types_local);
        let call_generic_apply = Call(generic_apply_index);
        let call_free_values_buffer = Call(stack_library_for_target.free_values_buffer);
        let call_free_types_buffer = Call(stack_library_for_target.free_types_buffer);

        let mut instrumented_body = Vec::new();
        instrumented_body.extend(push_args_on_stack);
        instrumented_body.push(call_allocate_values_buffer);
        instrumented_body.push(local_set_values_buffer_ptr);
        instrumented_body.push(call_allocate_types_buffer);
        instrumented_body.push(local_set_types_buffer_ptr);
        instrumented_body.extend_from_slice(&[
            // Prep call generic apply
            const_apply_table_index,   // f_apply : i32
            argc,                      // argc    : i32
            resc,                      // resc    : i32
            local_get_stack_ptr(),     // sigv    : i32
            local_get_stack_types_ptr, // sigtypv : i32
            call_generic_apply,
        ]);

        for load_call in &stack_library_for_target.ret_load_n {
            instrumented_body.push(local_get_stack_ptr());
            instrumented_body.push(Call(*load_call));
        }

        instrumented_body.push(call_free_values_buffer);
        instrumented_body.push(call_free_types_buffer);
        instrumented_body.push(End);
        original_function.code_mut().unwrap().body = instrumented_body;
    }

    let apply_count = u32::try_from(apply_table_funs.len()).unwrap();
    let apply_table_idx = module.tables.len();
    module.tables.push(Table {
        limits: Limits {
            initial_size: apply_count,
            max_size: Some(apply_count),
        },
        import: None,
        ref_type: RefType::FuncRef,
        export: vec![],
    });

    let apply_table_funs_refs: Vec<Vec<Instr>> = apply_table_funs
        .iter()
        .map(|idx| vec![RefFunc(*idx), End])
        .collect();

    module.elements.push(Element {
        typ: RefType::FuncRef,
        init: apply_table_funs_refs,
        mode: ElementMode::Active {
            table: apply_table_idx.into(),
            offset: vec![Const(Val::I32(0)), End],
        },
    });

    // 2. Generate 'call_base'
    let call_base_idx = module.add_function(
        wasp_imported_generic_apply_base.as_function_type(),
        vec![],
        vec![
            Local(LocalOp::Get, 1_usize.into()), // f_apply
            Local(LocalOp::Get, 0_usize.into()), // sigv
            CallIndirect(
                FunctionType::new(&[ValType::I32], &[]),
                apply_table_index.into(),
            ),
            End,
        ],
    );

    module
        .function_mut(call_base_idx)
        .export
        .push(wasp_imported_generic_apply_base.name.to_string());

    AssemblyScriptProgram {
        content: assemblyscript_code,
    }
}
