use super::*;
use crate::lib_compile::assemblyscript::{
    compiler::Compiler as AssemblyScriptCompiler,
    options::CompilerOptions as AssemblyScriptCompilerOptions,
};
use rust_to_wasm_compiler::RustToWasmCompiler as RustCompiler;
use wasmtime::{Engine, Instance, Module, Store};
use WasmType::{I32, I64};

macro_rules! declare_functions_from {
    ($instance:ident, $store:ident,
        $($function_name:ident $function_name_lit:literal [$($input_type:ident),*] [$($output_type:ident),*]),+ $(,)?) => {
            $(
                let $function_name = $instance.get_typed_func::<($($input_type,)*), ($($output_type,)*)>(&mut $store, $function_name_lit).unwrap();
            )*
        };
}

macro_rules! wasm_call {
    ($store:ident, $func_name:ident, $args:expr) => {
        $func_name.call(&mut $store, $args).unwrap()
    };
}

fn get_use_case_signature() -> Vec<Signature> {
    vec![Signature {
        return_types: vec![I32, I64],
        argument_types: vec![],
    }]
}

fn assert_lib_with_use_case(instrumentation_wasm_library: Vec<u8>) {
    ///////////////////
    //// EXECUTION ////
    ///////////////////
    let engine = Engine::default();
    let mut store: Store<()> = Store::new(&engine, ());
    let module = Module::from_binary(&engine, &instrumentation_wasm_library).unwrap();
    let instance = Instance::new(&mut store, &module, &[]).unwrap();

    declare_functions_from! { instance, store,
        // wastrumentation_stack_load_i32   "wastrumentation_stack_load_i32" [i32, i32]      [i32],
        // wastrumentation_stack_load_i64   "wastrumentation_stack_load_i64" [i32, i32]      [i64],
        // wastrumentation_stack_store_i32  "wastrumentation_stack_store_i32" [i32, i32, i32] [],
        // wastrumentation_stack_store_i64  "wastrumentation_stack_store_i64" [i32, i64, i32] [],
        allocate_ret_i32_i64_arg         "allocate_ret_i32_i64_arg" []              [i32],
        load_ret0_ret_i32_i64_arg        "load_ret0_ret_i32_i64_arg" [i32]           [i32],
        load_ret1_ret_i32_i64_arg        "load_ret1_ret_i32_i64_arg" [i32]           [i64],
        store_ret0_ret_i32_i64_arg       "store_ret0_ret_i32_i64_arg" [i32, i32]      [],
        store_ret1_ret_i32_i64_arg       "store_ret1_ret_i32_i64_arg" [i32, i64]      [],
        // free_values_ret_i32_i64_arg      "free_values_ret_i32_i64_arg" [i32]           [],
        store_rets_ret_i32_i64_arg       "store_rets_ret_i32_i64_arg" [i32, i32, i64] [],
        allocate_types_ret_i32_i64_arg   "allocate_types_ret_i32_i64_arg" []              [i32],
        free_types_ret_i32_i64_arg       "free_types_ret_i32_i64_arg" [i32]           [],
    };

    // <empty>
    let (values_buff_ptr_1,) = wasm_call!(store, allocate_ret_i32_i64_arg, ());
    // [_i32_, _i64_]
    wasm_call! {store, store_rets_ret_i32_i64_arg, (values_buff_ptr_1, 123, 456)};
    // [_123_, _456_]
    assert_eq! {wasm_call!(store, load_ret0_ret_i32_i64_arg, (values_buff_ptr_1,)), (123,)};
    assert_eq! {wasm_call!(store, load_ret1_ret_i32_i64_arg, (values_buff_ptr_1,)), (456,)};
    // [_123_, _456_] [_0_, _3_]
    let (types_buff_ptr_1,) = wasm_call! {store, allocate_types_ret_i32_i64_arg, ()};
    // [_123_, _456_]
    assert!(matches!(
        wasm_call!(store, free_types_ret_i32_i64_arg, (types_buff_ptr_1,)),
        ()
    ));

    let (values_buff_ptr_2,) = wasm_call!(store, allocate_ret_i32_i64_arg, ());
    // [_123_, _456_] [_i32_, _i64_]
    wasm_call! {store, store_ret0_ret_i32_i64_arg, (values_buff_ptr_2, 123)};
    // [_123_, _456_] [_123_, _i64_]
    wasm_call! {store, store_ret1_ret_i32_i64_arg, (values_buff_ptr_2, 456)};
    // [_123_, _456_] [_123_, _456_]
    assert_eq! {wasm_call!(store, load_ret0_ret_i32_i64_arg, (values_buff_ptr_2,)), (123,)};
    assert_eq! {wasm_call!(store, load_ret1_ret_i32_i64_arg, (values_buff_ptr_2,)), (456,)};
}

#[test]
fn test_edge_rust() {
    // Generate input program signatures
    let signatures = get_use_case_signature();
    let generated_lib = generate_lib(&signatures);
    println!("{generated_lib:?}");

    // Compile
    let compiler = RustCompiler::new().unwrap();
    let (ManifestSource(manifest_source), RustSourceCode(rust_source_code)) = generated_lib;
    let instrumentation_wasm_library = compiler
        .compile_source(
            WasiSupport::Disabled,
            &manifest_source,
            &rust_source_code,
            Profile::Dev,
        )
        .unwrap();

    let _ = instrumentation_wasm_library;

    // Assert execution
    // FIXME:
    // assert_lib_with_use_case(instrumentation_wasm_library);
}

#[test]
fn test_edge_assemblyscript() {
    // Generate input program signatures
    let signatures = get_use_case_signature();
    let generated_lib = super::super::super::assemblyscript::generate_lib(&signatures);
    println!("{generated_lib}");

    // Compile
    let compiler = AssemblyScriptCompiler::new().unwrap();
    let instrumentation_wasm_library = compiler
        .compile(&AssemblyScriptCompilerOptions::default_for(generated_lib))
        .unwrap();

    // Assert execution
    assert_lib_with_use_case(instrumentation_wasm_library);
}
