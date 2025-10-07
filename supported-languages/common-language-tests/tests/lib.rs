use wasmtime::{Engine, Instance, Module, Store};
use wastrumentation::compiler::{Compiles, DefaultCompilerOptions};
use wastrumentation::wasm_constructs::Signature;
use wastrumentation::wasm_constructs::WasmType::{I32, I64};

include!("wasmtime_macros.rs");

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

    declare_fns_from_wasm! { instance, store,
        // wastrumentation_stack_load_i32  [i32, i32]      [i32],
        // wastrumentation_stack_load_i64  [i32, i32]      [i64],
        // wastrumentation_stack_store_i32 [i32, i32, i32] [],
        // wastrumentation_stack_store_i64 [i32, i64, i32] [],
        allocate_ret_i32_i64_arg        []              [i32],
        load_ret0_ret_i32_i64_arg       [i32]           [i32],
        load_ret1_ret_i32_i64_arg       [i32]           [i64],
        store_ret0_ret_i32_i64_arg      [i32, i32]      [],
        store_ret1_ret_i32_i64_arg      [i32, i64]      [],
        // free_values_ret_i32_i64_arg     [i32]           [],
        store_rets_ret_i32_i64_arg      [i32, i32, i64] [],
        allocate_types_ret_i32_i64_arg  []              [i32],
        free_types_ret_i32_i64_arg      [i32]           [],
    };

    // <empty>
    let values_buff_ptr_1 = wasm_call!(store, allocate_ret_i32_i64_arg);
    // [_i32_, _i64_]
    wasm_call! {store, store_rets_ret_i32_i64_arg, (values_buff_ptr_1, 123, 456)};
    // [_123_, _456_]
    assert_eq! {wasm_call!(store, load_ret0_ret_i32_i64_arg, values_buff_ptr_1), 123};
    assert_eq! {wasm_call!(store, load_ret1_ret_i32_i64_arg, values_buff_ptr_1), (456)};
    // [_123_, _456_] [_0_, _3_]
    let types_buff_ptr_1 = wasm_call! {store, allocate_types_ret_i32_i64_arg, ()};
    // [_123_, _456_]
    assert!(matches!(
        wasm_call!(store, free_types_ret_i32_i64_arg, types_buff_ptr_1),
        ()
    ));

    let values_buff_ptr_2 = wasm_call!(store, allocate_ret_i32_i64_arg);
    // [_123_, _456_] [_i32_, _i64_]
    wasm_call! {store, store_ret0_ret_i32_i64_arg, (values_buff_ptr_2, 123)};
    // [_123_, _456_] [_123_, _i64_]
    wasm_call! {store, store_ret1_ret_i32_i64_arg, (values_buff_ptr_2, 456)};
    // [_123_, _456_] [_123_, _456_]
    assert_eq! {wasm_call!(store, load_ret0_ret_i32_i64_arg, values_buff_ptr_2), 123};
    assert_eq! {wasm_call!(store, load_ret1_ret_i32_i64_arg, values_buff_ptr_2), 456};
}

#[test]
fn test_edge_rust() {
    use rust_to_wasm_compiler::WasiSupport; // FIXME: this dependency is bad :/
    use wastrumentation_lang_rust::compile::compiler::Compiler;
    use wastrumentation_lang_rust::compile::options::CompilerOptions;
    use wastrumentation_lang_rust::compile::options::RustSource;
    use wastrumentation_lang_rust::generate::instrumentation::generate_lib;

    // Generate input program signatures
    let signatures = get_use_case_signature();
    let generated_lib = generate_lib(&signatures);
    let (manifest_source, rust_source_code) = generated_lib;

    // Compile
    let compiler = Compiler::setup_compiler().unwrap();
    let compiler_options = CompilerOptions::default_for(RustSource::SourceCode(
        WasiSupport::Disabled,
        manifest_source,
        rust_source_code,
    ));
    let instrumentation_wasm_library = compiler.compile(&compiler_options).unwrap();

    // Assert execution
    assert_lib_with_use_case(instrumentation_wasm_library);
}

#[test]
fn test_edge_webassembly() {
    use wastrumentation_lang_webassembly::compile::compiler::Compiler;
    use wastrumentation_lang_webassembly::compile::options::CompilerOptions;
    use wastrumentation_lang_webassembly::compile::options::WebAssemblySource;
    use wastrumentation_lang_webassembly::generate::instrumentation::generate_lib;

    // Generate input program signatures
    let signatures = get_use_case_signature();
    let generated_lib = generate_lib(&signatures);

    // Compile
    let compiler = Compiler::setup_compiler().unwrap();
    let compiler_options = CompilerOptions::default_for(WebAssemblySource::Wat(generated_lib));
    let instrumentation_wasm_library = compiler.compile(&compiler_options).unwrap();

    // Assert execution
    assert_lib_with_use_case(instrumentation_wasm_library);
}

#[test]
fn test_edge_assemblyscript() {
    use wastrumentation_lang_assemblyscript::compile::compiler::Compiler;
    use wastrumentation_lang_assemblyscript::compile::options::CompilerOptions;
    use wastrumentation_lang_assemblyscript::generate::instrumentation::generate_lib;

    // Generate input program signatures
    let signatures = get_use_case_signature();
    let generated_lib = generate_lib(&signatures);

    // Compile
    let compiler = Compiler::setup_compiler().unwrap();
    let compiler_options = CompilerOptions::default_for(generated_lib);
    let instrumentation_wasm_library = compiler.compile(&compiler_options).unwrap();

    // Assert execution
    assert_lib_with_use_case(instrumentation_wasm_library);
}
