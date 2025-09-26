use asc_compiler_rs::compiler::Compiler as AssemblyscriptCompiler;
use asc_compiler_rs::options::CompilerOptions as AssemblyscriptCompilerOptions;
use wastrumentation::compiler::Compiles;
use wastrumentation::{Configuration, Wastrumenter};

use wastrumentation_lang_assemblyscript::generate::analysis::WaspAnalysisSpec;
use wastrumentation_lang_rust::compile::compiler::Compiler as RustCompiler;

use wasmtime::*;

const SOURCE_CODE_WASP: &str = r#"
    (aspect
        (global >>>GUEST>>>
            export let call_count: i32 = 0;
            export let call_depth_max: i32 = 0;
            let call_stack: i32 = 0;

            function max<T>(a: T, b: T): T {
                return a > b ? a : b;
            }
        <<<GUEST<<<)

        (advice apply (func    WasmFunction)
                      (args    MutDynArgs)
                      (results MutDynResults) >>>GUEST>>>
            // Before call:
            // [1] Increment call stack size
            // [2] Ensure highest call stack size is recorded
            // [3] Ensure call count is incremented
            // After call:
            // [4] Ensure call count is decremented


            /* [1] */
            call_stack += 1;
            /* [2] */
            call_depth_max = max(call_depth_max, call_stack);
            /* [3] */
            call_count += 1;
            func.apply();
            /* [4] */
            call_stack -= 1;

        <<<GUEST<<<))"#;

const SOURCE_CODE_INPUT: &str = r#"
    export function fib(n: i32): i32 {
        return n <= 2 ? 1 : fib(n - 1) + fib(n - 2);
    }"#;

#[test]
fn example_instrumentation_rust() {
    let as_compiler = AssemblyscriptCompiler::setup_compiler().unwrap();
    let as_compiler_options = AssemblyscriptCompilerOptions::default_for(SOURCE_CODE_INPUT);

    let input_program = as_compiler.compile(&as_compiler_options).unwrap();

    let wasp_analysis_spec = (&WaspAnalysisSpec {
        wasp_source: SOURCE_CODE_WASP.into(),
    })
        .try_into()
        .unwrap();

    let rust_compiler = RustCompiler::setup_compiler().unwrap();
    let instrumented_input = Wastrumenter::new(Box::new(rust_compiler), Box::new(as_compiler))
        .wastrument(
            &input_program,
            wasp_analysis_spec,
            &Configuration::default(),
        )
        .unwrap();

    // Execute & check instrumentation
    let mut store = Store::<()>::default();
    let module = Module::from_binary(store.engine(), &instrumented_input).unwrap();
    let instance = Instance::new(&mut store, &module, &[]).unwrap();

    // Fetch `fib` export
    let fib = instance
        .get_typed_func::<i32, i32>(&mut store, "fib")
        .unwrap();

    let call_to_fib_result = fib.call(&mut store, 10).unwrap();

    assert_eq!(call_to_fib_result, 55);

    let expected_globals = [("call_count", 109), ("call_depth_max", 9)];
    for (global_name, expected_value) in expected_globals {
        let global = instance.get_global(&mut store, global_name).unwrap();
        let actual_value = global.get(&mut store).i32().unwrap();
        assert_eq!(expected_value, actual_value);
    }
}

#[test]
fn example_instrumentation_wasp() {
    let assemblyscript_compiler_options =
        AssemblyscriptCompilerOptions::default_for(SOURCE_CODE_INPUT.to_string());
    let assemblyscript_compiler1 = AssemblyscriptCompiler::setup_compiler().unwrap();
    let assemblyscript_compiler2 = AssemblyscriptCompiler::setup_compiler().unwrap();

    // TODO: better to create a second one? Perhaps use Rc? Or maybe Arc?

    let input_program = assemblyscript_compiler1
        .compile(&assemblyscript_compiler_options)
        .unwrap();

    let wasp_analysis_spec = (&WaspAnalysisSpec {
        wasp_source: SOURCE_CODE_WASP.into(),
    })
        .try_into()
        .unwrap();

    let instrumented_input = Wastrumenter::new(
        Box::new(assemblyscript_compiler1),
        Box::new(assemblyscript_compiler2),
    )
    .wastrument(
        &input_program,
        wasp_analysis_spec,
        &Configuration::default(),
    )
    .unwrap();

    // Execute & check instrumentation
    let mut store = Store::<()>::default();
    let module = Module::from_binary(store.engine(), &instrumented_input).unwrap();
    let instance = Instance::new(&mut store, &module, &[]).unwrap();

    // Fetch `fib` export
    let fib = instance
        .get_typed_func::<i32, i32>(&mut store, "fib")
        .unwrap();

    let call_to_fib_result = fib.call(&mut store, 10).unwrap();

    assert_eq!(call_to_fib_result, 55);

    let expected_globals = [("call_count", 109), ("call_depth_max", 9)];
    for (global_name, expected_value) in expected_globals {
        let global = instance.get_global(&mut store, global_name).unwrap();
        let actual_value = global.get(&mut store).i32().unwrap();
        assert_eq!(expected_value, actual_value);
    }
}
