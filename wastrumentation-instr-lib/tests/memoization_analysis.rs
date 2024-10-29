// Rust STD
use std::time::Instant;
use std::{path::absolute, time::Duration};

use regex::Regex;
use rust_to_wasm_compiler::WasiSupport;
// Wastrumentation imports
use wastrumentation::{compiler::Compiles, Configuration, PrimaryTarget, Wastrumenter};
use wastrumentation_instr_lib::lib_compile::rust::{
    compiler::Compiler,
    options::{CompilerOptions, ManifestSource, RustSource, RustSourceCode},
};
use wastrumentation_instr_lib::lib_gen::analysis::rust::Hook;
use wastrumentation_instr_lib::lib_gen::analysis::rust::RustAnalysisSpec;

// Wasmtime imports
use wasmtime::{Instance, Module, Store};

use wastrumentation_static_analysis::immutable_functions_from_binary;

fn compile_input_program(manifest: impl Into<String>, source: impl Into<String>) -> Vec<u8> {
    Compiler::setup_compiler()
        .unwrap()
        .compile(&CompilerOptions {
            profile: rust_to_wasm_compiler::Profile::Release,
            source: RustSource::SourceCode(
                rust_to_wasm_compiler::WasiSupport::Disabled,
                ManifestSource(manifest.into()),
                RustSourceCode(source.into()),
            ),
        })
        .unwrap()
}

const INCREMENT_UPPER_BOUND: i32 = 20;
const EXPECTED_OUTCOME: i32 = -10485760;
const BENCHMARK_ITERATIONS: i32 = 30;

#[test]
fn test_basic() {
    ////////////////
    // 1. COMPILE //
    ////////////////
    let input_program = compile_input_program(
        MEMOIZATION_CANDIDATE_INPUT_MANIFEST,
        MEMOIZATION_CANDIDATE_INPUT_PROGRAM,
    );

    let report = report_memoization_benches_for(&input_program, 500_000, "compute_recursive");
    let time_elapsed_after_uninstrumented_call = report.uninstrumented_duration;
    let time_elapsed_after_instrumented_call = report.instrumented_duration;

    assert!(report.runtime_pure_function_calls.len() == 4);
    assert_eq!(report.runtime_target_functions, vec![4]);

    dbg!(report.cache_size_report);
    dbg!(report.cache_hit_report);

    // FINAL REPORT
    println!("Elapsed (uninstrumented): {time_elapsed_after_uninstrumented_call:.2?}",);
    println!("Elapsed (instrumented): {time_elapsed_after_instrumented_call:.2?}",);
}

struct MemoizationBenches {
    uninstrumented_duration: Duration,
    instrumented_duration: Duration,
    runtime_pure_function_calls: Vec<(u32, i32)>,
    runtime_target_functions: Vec<u32>,
    cache_size_report: i32,
    cache_hit_report: i32,
}

// TODO: change Vec into &[u8]
fn report_memoization_benches_for(
    input_program: &Vec<u8>,
    threshold_pure_f: i32,
    entry_point: &'_ str,
) -> MemoizationBenches {
    ///////////////////////////////////////////////
    // 2. BENCH & ASSERT UNINSTRUMENTED BEHAVIOR //
    ///////////////////////////////////////////////

    // Execute & check instrumentation
    let mut store = Store::<()>::default();
    let module = Module::from_binary(store.engine(), &input_program).unwrap();
    let instance = Instance::new(&mut store, &module, &[]).unwrap();

    // Fetch `fib` export
    let compute_recursive = instance
        .get_typed_func::<i32, i32>(&mut store, entry_point)
        .unwrap();

    let timestamp_before_uninstrumented_call = Instant::now();

    let mut total: i32 = 0; // will be initialized in loop

    // Invoke few times
    for _ in 0..BENCHMARK_ITERATIONS {
        total = (0..INCREMENT_UPPER_BOUND)
            .map(|i| compute_recursive.call(&mut store, i).unwrap())
            .reduce(i32::wrapping_add)
            .unwrap();
    }

    let time_elapsed_after_uninstrumented_call = timestamp_before_uninstrumented_call.elapsed();

    // Assert correct outcome
    assert_eq!(total, EXPECTED_OUTCOME);

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
    let analysis_compiler = Compiler::setup_compiler().expect("Setup Rust compiler");
    let instrumentation_compiler = Compiler::setup_compiler().expect("Setup Rust compiler");

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
    let mut store = Store::<()>::default();
    let module = Module::from_binary(store.engine(), &wastrumented).unwrap();
    let instance = Instance::new(&mut store, &module, &[]).unwrap();

    // Fetch `fib` export
    let compute_recursive = instance
        .get_typed_func::<i32, i32>(&mut store, entry_point)
        .unwrap();

    // Invoke few times
    let total: i32 = (0..INCREMENT_UPPER_BOUND)
        .map(|i| compute_recursive.call(&mut store, i).unwrap())
        .reduce(i32::wrapping_add)
        .unwrap();

    // Assert correct outcome
    assert_eq!(total, EXPECTED_OUTCOME);

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

    let analysis_pure_function_profiler_manifest = format!(
        r#"
        package.name = "rust-analysis-pure-function-profiler"
        package.version = "0.1.0"
        package.edition = "2021"
        lib.crate-type = ["cdylib"]
        dependencies.wee_alloc = "0.4.5"
        dependencies.wastrumentation-rs-stdlib = {{ path = "{wastrumentation_rs_stdlib_path}", features = [
            "std",
        ] }}
        dependencies.lazy_static = "1.5.0"
        dependencies.ordered-float = "4.2.2"
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
        RustSourceCode(MEMOIZATION_ANALYSIS.into()),
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
    let mut store = Store::<()>::default();
    let module = Module::from_binary(store.engine(), &wastrumented).unwrap();
    let instance = Instance::new(&mut store, &module, &[]).unwrap();

    // Fetch `fib` export
    let compute_recursive = instance
        .get_typed_func::<i32, i32>(&mut store, entry_point)
        .unwrap();

    let timestamp_before_instrumented_call = Instant::now();
    let mut total: i32 = 0; // will be initialized in loop

    // Invoke few times
    for _ in 0..BENCHMARK_ITERATIONS {
        total = (0..INCREMENT_UPPER_BOUND)
            .map(|i| compute_recursive.call(&mut store, i).unwrap())
            .reduce(i32::wrapping_add)
            .unwrap();
    }

    let time_elapsed_after_instrumented_call = timestamp_before_instrumented_call.elapsed();

    // Assert correct outcome
    assert_eq!(total, EXPECTED_OUTCOME);

    // Fetch `cache statistics` export
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
    }
}

const MEMOIZATION_CANDIDATE_INPUT_MANIFEST: &str = r#"
package.name = "rust-memoization-candidate"
package.version = "0.1.0"
package.edition = "2021"
lib.crate-type = ["cdylib"]
profile.release.strip = true
profile.release.lto = true
profile.release.panic = "abort"
[workspace]
"#;

const MEMOIZATION_CANDIDATE_INPUT_PROGRAM: &str = r#"
use std::{f64::consts::PI, i32};

/// Calculates the sine of an angle in radians.
#[no_mangle]
pub extern "C" fn sin(x: f64) -> f64 {
    // Normalize the angle to the range [-π, π]
    let x = x % (2.0 * PI);

    // Use the Taylor series approximation for small angles
    if x.abs() < 0.1 {
        return x - x.powi(3) / 6.0 + x.powi(5) / 120.0;
    }

    // Use the reduction formula to reduce the angle to the range [-π/4, π/4]
    let quadrant = (x / (PI / 2.0)).floor() as i32;
    let x = x - (quadrant as f64) * (PI / 2.0);

    // Use the Taylor series approximation for the reduced angle
    let sin_x = x - x.powi(3) / 6.0 + x.powi(5) / 120.0;

    // Apply the appropriate sign based on the quadrant
    match quadrant {
        0 | 1 => sin_x,
        2 | 3 => -sin_x,
        _ => unreachable!(),
    }
}

/// Calculates the cosine of an angle in radians.
#[no_mangle]
pub extern "C" fn cos(x: f64) -> f64 {
    // Use the identity cos(x) = sin(x + π/2)
    sin(x + PI / 2.0)
}

/// Calculates the tangent of an angle in radians.
#[no_mangle]
pub extern "C" fn tan(x: f64) -> f64 {
    // Use the identity tan(x) = sin(x) / cos(x)
    sin(x) / cos(x)
}

#[no_mangle]
pub extern "C" fn fibonacci(n: i32) -> i32 {
    if n < 2 {
        1
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

#[no_mangle]
pub extern "C" fn compute_recursive(n: i32) -> i32 {
    let mut total: f64 = 0.0;
    for _ in 0..1000 {
        total += tan(total) + f64::from(fibonacci(n));
    }

    let [_, _, _, _, l1, l2, l3, l4] = total.to_le_bytes();
    i32::from_le_bytes([l1, l2, l3, l4])
}
"#;

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

const MEMOIZATION_ANALYSIS: &str = r#"
extern crate wastrumentation_rs_stdlib;

use lazy_static::lazy_static;
use ordered_float::OrderedFloat;
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::Mutex;
use wastrumentation_rs_stdlib::{advice, MutDynArgs, MutDynResults, WasmFunction, WasmValue};

// Global cache structure using a Mutex and AtomicUsize for thread safety
lazy_static! {
    static ref CACHE: Mutex<HashMap<CacheKey, Vec<WasmValueEq>>> = Mutex::new(HashMap::new());
    static ref CACHE_SIZE: AtomicUsize = AtomicUsize::new(0);
}

#[no_mangle]
pub extern "C" fn CACHE_SIZE_REPORT() -> i32 {
    CACHE.lock().unwrap().len() as i32
}

static mut CACHE_HITS: i32 = 0;

#[no_mangle]
pub extern "C" fn CACHE_HIT_REPORT() -> i32 {
    unsafe { CACHE_HITS }
}

// Define a CacheKey structure to uniquely identify cached function calls
#[derive(Eq, PartialEq, Hash)]
struct CacheKey {
    instr_f_idx: i32,
    args: Vec<WasmValueEq>,
}

#[derive(Eq, PartialEq, Hash, Clone)]
enum WasmValueEq {
    I32(i32),
    F32(OrderedFloat<f32>),
    I64(i64),
    F64(OrderedFloat<f64>),
}

impl Into<WasmValueEq> for WasmValue {
    fn into(self) -> WasmValueEq {
        match self {
            WasmValue::I32(v) => WasmValueEq::I32(v),
            WasmValue::F32(v) => WasmValueEq::F32(v.into()),
            WasmValue::I64(v) => WasmValueEq::I64(v),
            WasmValue::F64(v) => WasmValueEq::F64(v.into()),
        }
    }
}

impl Into<WasmValue> for &WasmValueEq {
    fn into(self) -> WasmValue {
        match self {
            WasmValueEq::I32(v) => WasmValue::I32(*v),
            WasmValueEq::F32(OrderedFloat(v)) => WasmValue::F32(*v),
            WasmValueEq::I64(v) => WasmValue::I64(*v),
            WasmValueEq::F64(OrderedFloat(v)) => WasmValue::F64(*v),
        }
    }
}

fn cache_hit(key: &CacheKey) -> bool {
    CACHE.lock().unwrap().contains_key(&key)
}

fn cache_retrieve(key: &CacheKey, results: &mut MutDynResults) {
    if let Some(cached_results) = CACHE.lock().unwrap().get(&key) {
        for (index, wasm_value) in cached_results.iter().enumerate() {
            results.set_res(i32::try_from(index).unwrap(), wasm_value.into());
        }
    } else {
        unreachable!()
    }
}

fn cache_insert(key: CacheKey, results: &MutDynResults) {
    let mut cached_results: Vec<WasmValueEq> =
        Vec::with_capacity(usize::try_from(results.resc).unwrap());

    for index in 0..results.resc {
        cached_results.push(results.get_res(index).into())
    }

    if cached_results.len() != usize::try_from(results.resc).unwrap() {
        unreachable!()
    }

    CACHE.lock().unwrap().insert(key, cached_results);
}

advice! { apply
    (func: WasmFunction, args: MutDynArgs, results: MutDynResults) {
        let mut wasm_value_vec: Vec<WasmValueEq> = Vec::with_capacity(usize::try_from(args.argc).unwrap());

        for index in 0..args.argc {
            wasm_value_vec.push(args.get_arg(index).into())
        }

        let key = CacheKey {
            instr_f_idx: func.instr_f_idx,
            args: wasm_value_vec,
        };

        if cache_hit(&key) {
            unsafe { CACHE_HITS += 1 };
            cache_retrieve(&key, &mut results);
        } else {
            func.apply();
            cache_insert(key, &results);
        }
    }
}
"#;
