use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
// Rust STD
use std::time::Instant;
use std::{path::absolute, time::Duration};

use regex::Regex;
use rust_to_wasm_compiler::WasiSupport;
// Wastrumentation imports
use wastrumentation::{compiler::Compiles, Configuration, PrimaryTarget, Wastrumenter};
use wastrumentation_instr_lib::lib_compile::assemblyscript::compiler::Compiler as AssemblyScriptCompiler;
use wastrumentation_instr_lib::lib_compile::rust::{
    compiler::Compiler as RustCompiler,
    options::{ManifestSource, RustSource, RustSourceCode},
};
use wastrumentation_instr_lib::lib_gen::analysis::rust::Hook;
use wastrumentation_instr_lib::lib_gen::analysis::rust::RustAnalysisSpec;

// Wasmtime imports
use wasmtime::{Engine, Instance, Module, Store};

use wastrumentation_static_analysis::immutable_functions_from_binary;

const BENCHMARK_ITERATIONS: i32 = 2; // CHANGE to a higher number when executing on 'release'!

// "commanderkeen", // CRASH on Wastrumentation: `Invalid input WebAssembly code at offset 3779369: data count section required`
// "jsc",           // CRASH on Wastrumentation: `Invalid input WebAssembly code at offset 4656677: data count section required`
// "pacalc",        // CRASH on Wastrumentation: `Invalid input WebAssembly code at offset 257212: data count section required`
// "rguilayout",    // CRASH on Wastrumentation: `Invalid input WebAssembly code at offset 431386: data count section required`
// "riconpacker",   // CRASH on Wastrumentation: `Invalid input WebAssembly code at offset 429128: data count section required`
// "bullet",        // CRASH on Wastrumentation: `Invalid input WebAssembly code at offset 580908: data count section required`
// "sqlgui",        // CRASH on Wastrumentation: `Invalid input WebAssembly code at offset 652424: data count section required`
// "funky-kart",    // CRASH on Wastrumentation: `Invalid input WebAssembly code at offset 681830: data count section required`
// "guiicons",      // CRASH on Wastrumentation: `Invalid input WebAssembly code at offset 303497: data count section required`
// "rfxgen",        // CRASH on Wastrumentation: `Invalid input WebAssembly code at offset 393847: data count section required`
// "rguistyler",    // CRASH on Wastrumentation: `Invalid input WebAssembly code at offset 442789: data count section required`
// "mandelbrot",    // CRASH on Wastrumentation: `Invalid input WebAssembly code at offset 72321: data count section required`

/*
Execution on release:
    cargo test --release --package wastrumentation-instr-lib --test memoization_test_wasmr3 -- test_basic --exact --show-output
*/
#[test]
fn test_basic() {
    for name in [
        "factorial",
        "figma-startpage",
        "game-of-life",
        "rtexviewer",
        "jqkungfu",
        "parquet",
        "rtexpacker",
        "hydro",
        "boa",
        "ffmpeg",
        "pathfinding",
        "sandspiel",
        "multiplyDouble",
        "fib",
        "multiplyInt",
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

struct MemoizationBenches {
    uninstrumented_duration: Duration,
    instrumented_duration: Duration,
    pure_functions: HashSet<u32>,
    runtime_pure_functions_calls: Vec<(u32, i32)>,
    runtime_target_functions: Vec<u32>,
    cache_size_report: i32,
    cache_hit_report: i32,
}

impl MemoizationBenches {
    fn summarize(&self) {
        let Self {
            uninstrumented_duration,
            instrumented_duration,
            pure_functions,
            runtime_pure_functions_calls,
            runtime_target_functions,
            cache_hit_report,
            cache_size_report,
            ..
        } = self;

        if runtime_target_functions.len() == 0 {
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
        for (runtime_pure_function, calls) in runtime_pure_functions_calls {
            if *calls > 0 {
                println!(" => Pure function {runtime_pure_function} was called {calls} times");
            };
        }
    }
}

// TODO: change Vec into &[u8]
fn report_memoization_benches_for(
    input_program_name: &'_ str,
    input_program: &Vec<u8>,
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
    let module = Module::from_binary(&engine, &input_program).unwrap();

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
    let immutable_set = immutable_functions_from_binary(&input_program).unwrap();

    ///////////////////////////
    // 4. PROFILE PURE FUNCS //
    ///////////////////////////
    let map_size = immutable_set.len();

    let map_target =
        Regex::new(r"(const MAP_SIZE: usize = )(\d+)(; \/\/ <TO_CODE_GEN \{MAP_SIZE\}>)").unwrap();
    //                   <-----------G1----------><-G2-><---------------G3--------------->

    //                                                 <-G1-->
    let map_increment_target = Regex::new(r"((\s)*)\/\/ <TO_CODE_GEN \{MAP_INCREMENT\}>").unwrap();

    assert!(
        map_target.is_match(PURE_FUNCTIONS_PROFILER_PROGRAM),
        "Could not find gen location to allocate static map: ${map_target}"
    );
    assert!(
        map_increment_target.is_match(PURE_FUNCTIONS_PROFILER_PROGRAM),
        "Could not find gen location to inject increment instructions: ${map_target}"
    );

    let pure_functions_profiler_program = ([
        // Rewrite the allocation map into a concrete allocation
        (
            map_target,
            Box::new(|capture: &regex::Captures| {
                format!(
                    "{map_size_assignment} {map_size};",
                    map_size_assignment = &capture[1]
                )
            }),
        ),
        // Rewrite the function call with a map increment
        (
            map_increment_target,
            Box::new(|capture: &regex::Captures| {
                let space_group = &capture[1];
                let map_increment_code = immutable_set
                    .iter()
                    .enumerate()
                    .map(|(map_index, function_index)| {
                        format!(
                        "{space_group}{function_index} => map[{map_index}] = map[{map_index}] + 1,"
                    )
                    })
                    .collect::<Vec<String>>()
                    .join("\n");
                format!("{map_increment_code}")
            }),
        ),
    ]
        as [(Regex, Box<dyn Fn(&regex::Captures) -> String>); 2])
        .iter()
        .fold(
            String::from(PURE_FUNCTIONS_PROFILER_PROGRAM),
            |analysis_source_code, (regex, replacer)| {
                regex.replace_all(&analysis_source_code, replacer).into()
            },
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
        target_indices: Some(immutable_set.iter().map(|v| *v).collect()),
        primary_selection: Some(PrimaryTarget::Analysis),
    };

    let wastrumenter = Wastrumenter::new(instrumentation_compiler.into(), analysis_compiler.into());
    let wastrumented = wastrumenter
        .wastrument(&input_program, analysis, &configuration)
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
        runtime_pure_functions_calls: runtime_profiled_pure_functions_calls,
        runtime_target_functions: pure_functions_of_interest,
        cache_size_report,
        cache_hit_report,
        pure_functions: immutable_set,
    }
}

const PURE_FUNCTIONS_PROFILER_PROGRAM: &str = r#"
#![no_std]

/// WARNING: This program is still a template.
///          This  program  contains  template
///          snippets  like  <TO_CODE_GEN {x}>
///          that   will  be  filled  in  with
///          results from the  static analysis
///          that  comes before  this  dynamic
///          analysis execution.

extern crate wastrumentation_rs_stdlib;
use wastrumentation_rs_stdlib::*;

const MAP_SIZE: usize = 0; // <TO_CODE_GEN {MAP_SIZE}>
static mut MAP: &mut [i32] = &mut [0; MAP_SIZE]; // Maps [FunctionIndex -> CallCount]

#[no_mangle]
pub extern "C" fn get_calls_for(index: i32) -> i32 {
    unsafe { MAP[index as usize] }
}

advice! { apply (func: WasmFunction, _args: MutDynArgs, _results: MutDynResults) {
        let map = unsafe { MAP.as_mut() };

        match func.instr_f_idx {
            // <TO_CODE_GEN {MAP_INCREMENT}>
            _ => (), // core::panic!(),
        }

        func.apply();
    }
}
"#;
