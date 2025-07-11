use std::fs::File;
use std::io::Read;
use std::path::absolute;
use std::time::Instant;

use indoc::indoc;
use rust_to_wasm_compiler::WasiSupport;

// Wastrumentation imports
use wastrumentation::{compiler::Compiles, Configuration, PrimaryTarget, Wastrumenter};
use wastrumentation_instr_lib::lib_compile::assemblyscript::compiler::Compiler as AssemblyScriptCompiler;
use wastrumentation_instr_lib::lib_compile::rust::{
    compiler::Compiler as RustCompiler,
    options::{ManifestSource, RustSource, RustSourceCode},
};
use wastrumentation_instr_lib::lib_gen::analysis::rust::{Hook, RustAnalysisSpec};
use wastrumentation_static_analysis::immutable_functions_from_binary;

// Wasmtime imports
use wasmtime::{Engine, Instance, Module, Store};

const BENCHMARK_ITERATIONS: i32 = 2; // CHANGE to a higher number when executing on 'release'!

/*
Execution on release:
    cargo test --release --package wastrumentation-instr-lib --test memoization_test_wasmr3 -- test_basic --exact --show-output
*/

// TODO: change this test that it either:
// * downloads the input programs if not found (-> internet connection dependent tests; flakey)
// * includes the input programs in the repo (-> grows the repo size with +50mb)
// TODO: either this is enabled (& configurable) or it should be moved?
#[ignore]
// #[test]
#[allow(unused)]
fn test_basic() {
    for name in [
        //
        // Memoization does not apply:
        "factorial",
        "game-of-life",
        "rtexviewer",
        "jqkungfu",
        "parquet",
        "rtexpacker",
        "ffmpeg",
        "pathfinding",
        "sandspiel",
        "commanderkeen",
        "riconpacker",
        "guiicons",
        "mandelbrot",
        //
        // Memoization does apply, and speedup / slowdown is most significant
        "boa",
        "multiplyDouble",
        "fib",
        "multiplyInt",
        "pacalc",
        "funky-kart",
        //
        // Memoization does apply, but speedup / slowdown is not significant
        "figma-startpage",
        "hydro",
        "jsc",
        "rguilayout",
        "bullet",
        "sqlgui",
        "rfxgen",
        "rguistyler",
    ] {
        // Read input program
        let mut input_program: Vec<u8> = Vec::new();
        File::open(absolute(format!("../benchmarking-node/wasmr3-python/working-dir/wasm-r3/benchmarks/{name}/{name}.wasm")).unwrap())
            .unwrap()
            .read_to_end(&mut input_program)
            .unwrap();

        let report = report_memoization_benches_for(name, &input_program, 1, "_start");
        report.summarize();
    }
}

mod memoization_analysis;
use memoization_analysis::MemoizationBenches;

impl MemoizationBenches {
    fn summarize(&self) {
        let Self {
            uninstrumented_duration,
            instrumented_duration,
            pure_functions,
            runtime_pure_function_calls,
            runtime_target_functions,
            cache_hit_report,
            cache_size_report,
            ..
        } = self;

        if runtime_target_functions.is_empty() {
            println!("This input program does not apply; not reporting further details.",);
            return;
        }

        println!("Elapsed (uninstrumented): {uninstrumented_duration:.2?}",);
        println!("Elapsed (instrumented): {instrumented_duration:.2?}",);

        let slowdown: u128 =
            instrumented_duration.as_micros() / uninstrumented_duration.as_micros();
        let relation_report = if instrumented_duration < uninstrumented_duration {
            "🐇 - SPEEDUP"
        } else {
            "🐢 - SLOWDOWN"
        };
        println!("{relation_report} ({slowdown})");
        println!("Cache hit report: {cache_hit_report}, cache_size_report: {cache_size_report}");
        println!("# of pure functions: {}", pure_functions.len());
        println!(
            "# of runtime target functions: {}",
            runtime_target_functions.len()
        );
        for (runtime_pure_function, calls) in runtime_pure_function_calls {
            if *calls > 0 {
                println!(" => Pure function {runtime_pure_function} was called {calls} times");
            };
        }
    }
}

// TODO: change Vec into &[u8]
fn report_memoization_benches_for(
    input_program_name: &'_ str,
    input_program: &[u8],
    threshold_pure_f: i32,
    entry_point: &'_ str,
) -> MemoizationBenches {
    ///////////////////////////////////////////////
    // 2. BENCH & ASSERT UNINSTRUMENTED BEHAVIOR //
    ///////////////////////////////////////////////

    // Execute & check instrumentation
    println!("Running uninstrumented variant: {input_program_name}");
    let timestamp_before_uninstrumented_call = Instant::now();

    let engine = Engine::default();
    let module = Module::from_binary(&engine, input_program).unwrap();

    // Invoke few times
    for _ in 0..BENCHMARK_ITERATIONS {
        let mut store: Store<()> = Store::<()>::new(&engine, ());
        let instance = Instance::new(&mut store, &module, &[]).unwrap();

        // Fetch `fib` export
        let compute_recursive = instance
            .get_typed_func::<(), ()>(&mut store, entry_point)
            .unwrap();

        compute_recursive.call(&mut store, ()).unwrap();
    }

    let time_elapsed_after_uninstrumented_call = timestamp_before_uninstrumented_call.elapsed();

    ///////////////////////////
    // 3. FIND PURE FUNCTONS //
    ///////////////////////////
    let immutable_set = immutable_functions_from_binary(input_program).unwrap();

    ///////////////////////////
    // 4. PROFILE PURE FUNCS //
    ///////////////////////////
    let map_size = immutable_set.len();

    let pure_functions_profiler_program = format!(
        indoc! { r#"
            #![no_std]
            use core::ptr::addr_of_mut;
            use wastrumentation_rs_stdlib::*;

            const MAP_SIZE: usize = {map_size};
            static mut MAP: &mut [i32] = &mut [0; MAP_SIZE]; // Maps [FunctionIndex -> CallCount]

            #[no_mangle]
            pub extern "C" fn get_calls_for(index: i32) -> i32 {{
                unsafe {{ MAP[index as usize] }}
            }}

            advice! {{ apply (func: WasmFunction, _args: MutDynArgs, _results: MutDynResults) {{
                    let map = unsafe {{ addr_of_mut!(MAP).as_mut().unwrap() }};

                    match func.instr_f_idx {{
                        {map_increment_instructions}
                        _ => (), // core::panic!(),
                    }}

                    func.apply();
                }}
            }}"#
        },
        map_size = map_size,
        map_increment_instructions = immutable_set
            .iter()
            .enumerate()
            .map(|(map_index, function_index)| {
                format!("            {function_index} => map[{map_index}] = map[{map_index}] + 1,",)
            })
            .collect::<Vec<String>>()
            .join("\n")
            .to_string(),
    );

    // Perform profiling instrumentation
    let analysis_compiler = RustCompiler::setup_compiler().expect("Setup Rust compiler");
    let instrumentation_compiler =
        AssemblyScriptCompiler::setup_compiler().expect("Setup AssemblyScript compiler");

    let analysis_pure_function_profiler_manifest = format!(
        r#"
        package.name = "rust-analysis-pure-function-profiler"
        package.version = "0.1.0"
        package.edition = "2021"
        lib.crate-type = ["cdylib"]
        dependencies.wee_alloc = "0.4.5"
        dependencies.wastrumentation-rs-stdlib = {{ path = "{wastrumentation_rs_stdlib_path}" }}
        profile.release.strip = true
        profile.release.lto = true
        profile.release.panic = "abort"
        [workspace]
    "#,
        wastrumentation_rs_stdlib_path =
            absolute("./tests/analyses/rust/wastrumentation-rs-stdlib")
                .unwrap()
                .to_str()
                .unwrap()
    );

    let source = RustSource::SourceCode(
        WasiSupport::Disabled,
        ManifestSource(analysis_pure_function_profiler_manifest),
        RustSourceCode(pure_functions_profiler_program),
    );

    let hooks = vec![Hook::GenericApply].into_iter().collect();
    let analysis = RustAnalysisSpec { source, hooks }.into();

    let configuration = Configuration {
        target_indices: Some(immutable_set.iter().copied().collect()),
        primary_selection: Some(PrimaryTarget::Analysis),
    };

    let wastrumenter = Wastrumenter::new(instrumentation_compiler.into(), analysis_compiler.into());
    let wastrumented = wastrumenter
        .wastrument(input_program, analysis, &configuration)
        .expect("Wastrumentation should succeed");

    // Perform profiling instrumentation
    let engine = Engine::default();
    let module = Module::from_binary(&engine, &wastrumented).unwrap();

    let mut store: Store<()> = Store::<()>::new(&engine, ());
    let instance = Instance::new(&mut store, &module, &[]).unwrap();

    // Fetch entry-point export
    let compute_recursive = instance
        .get_typed_func::<(), ()>(&mut store, entry_point)
        .unwrap();

    // Invoke one time
    compute_recursive.call(&mut store, ()).unwrap();

    // Fetch `get_calls_for` export
    let get_calls_for = instance
        .get_typed_func::<i32, i32>(&mut store, "get_calls_for")
        .unwrap();

    // Retrieve runtime info ...
    let runtime_profiled_pure_functions_calls: Vec<(u32, i32)> = immutable_set
        .iter()
        .enumerate()
        .map(|(map_index, function_index)| {
            let runtime_calls = get_calls_for
                .call(&mut store, i32::try_from(map_index).unwrap())
                .unwrap();
            (*function_index, runtime_calls)
        })
        .collect();

    let pure_functions_of_interest: Vec<u32> = runtime_profiled_pure_functions_calls
        .iter()
        .filter_map(|(function_index, runtime_calls)| {
            (*runtime_calls > threshold_pure_f).then_some(*function_index)
        })
        .collect();

    ////////////////////////////////////////////////////////
    // 5. INSTRUMENT MEMOIZATION FOR THRESHOLD PURE FUNCS //
    ////////////////////////////////////////////////////////

    let source = RustSource::Manifest(
        WasiSupport::Disabled,
        absolute("./tests/analyses/rust/pure-functions-memoization/Cargo.toml").unwrap(),
    );
    let hooks = vec![Hook::GenericApply].into_iter().collect();
    let analysis = RustAnalysisSpec { source, hooks }.into();

    let configuration = Configuration {
        target_indices: Some(pure_functions_of_interest.clone()),
        primary_selection: Some(PrimaryTarget::Analysis),
    };

    let wastrumented = wastrumenter
        .wastrument(input_program, analysis, &configuration)
        .expect("Wastrumentation should succeed");

    // Perform profiling instrumentation
    let timestamp_before_instrumented_call = Instant::now();

    let engine = Engine::default();
    let module = Module::from_binary(&engine, &wastrumented).unwrap();

    // Invoke few times
    for _ in 0..BENCHMARK_ITERATIONS {
        let mut store: Store<()> = Store::<()>::new(&engine, ());
        let instance = Instance::new(&mut store, &module, &[]).unwrap();

        // Fetch entry-point export
        let compute_recursive = instance
            .get_typed_func::<(), ()>(&mut store, entry_point)
            .unwrap();

        compute_recursive.call(&mut store, ()).unwrap();
    }

    let time_elapsed_after_instrumented_call = timestamp_before_instrumented_call.elapsed();

    // Fetch `cache statistics` export
    let engine = Engine::default();
    let module = Module::from_binary(&engine, &wastrumented).unwrap();
    let mut store: Store<()> = Store::<()>::new(&engine, ());
    let instance = Instance::new(&mut store, &module, &[]).unwrap();

    // Fetch entry-point export
    let compute_recursive = instance
        .get_typed_func::<(), ()>(&mut store, entry_point)
        .unwrap();

    compute_recursive.call(&mut store, ()).unwrap();

    let cache_size_report = instance
        .get_typed_func::<(), i32>(&mut store, "CACHE_SIZE_REPORT")
        .unwrap()
        .call(&mut store, ())
        .unwrap();

    let cache_hit_report = instance
        .get_typed_func::<(), i32>(&mut store, "CACHE_HIT_REPORT")
        .unwrap()
        .call(&mut store, ())
        .unwrap();

    MemoizationBenches {
        uninstrumented_duration: time_elapsed_after_uninstrumented_call,
        instrumented_duration: time_elapsed_after_instrumented_call,
        runtime_pure_function_calls: runtime_profiled_pure_functions_calls,
        runtime_target_functions: pure_functions_of_interest,
        cache_size_report,
        cache_hit_report,
        pure_functions: immutable_set,
    }
}
