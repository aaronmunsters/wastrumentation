// Rust STD
use std::path::absolute;

// Wastrumentation imports
use rust_to_wasm_compiler::WasiSupport;
use wastrumentation::{compiler::Compiles, Configuration, PrimaryTarget, Wastrumenter};

// Wasmtime imports
use wasmtime::{Config, Engine, Linker, Module, Store};

use wasmtime_wasi::{p1::WasiP1Ctx, WasiCtxBuilder};

// use wasm_opt::{Feature, OptimizationOptions, Pass};
use wastrumentation_lang_rust::{
    compile::{compiler::Compiler, options::RustSource},
    generate::analysis::{Hook, RustAnalysisSpec},
};

const PATH_INPUT_PROGRAM: &str = "./tests/input-programs/wat/tables.wat";
const PATH_INPUT_ANALYSIS: &str = "./tests/analyses/rust/logging/Cargo.toml";

#[test]
fn test_analysis() {
    let analysis_compiler = Compiler::setup_compiler().expect("Setup Rust compiler");
    let instrumentation_compiler = Compiler::setup_compiler().expect("Setup Rust compiler");

    let source = RustSource::Manifest(WasiSupport::Enabled, absolute(PATH_INPUT_ANALYSIS).unwrap());
    let hooks = Hook::all_hooks();
    let analysis = RustAnalysisSpec { source, hooks }.into();

    let configuration = Configuration {
        target_indices: None,
        primary_selection: Some(PrimaryTarget::Target),
    };

    // compile wat to wasm
    let input_program = wat::parse_file(PATH_INPUT_PROGRAM).unwrap();

    let wastrumenter = Wastrumenter::new(instrumentation_compiler.into(), analysis_compiler.into());
    let wastrumented = wastrumenter
        .wastrument(&input_program, analysis, &configuration)
        .expect("Wastrumentation should succeed");

    // let mut file = std::fs::File::create("./wastrumented.wasm").unwrap();
    // std::io::Write::write_all(&mut file, &wastrumented).unwrap();

    /////////////////////
    // WASMTIME ENGINE //
    /////////////////////

    // Construct the wasm engine
    let mut config = Config::new();
    config
        .wasm_backtrace(true)
        .wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Enable);
    let engine = Engine::new(&config).unwrap();

    let mut linker: Linker<WasiP1Ctx> = Linker::new(&engine);
    wasmtime_wasi::p1::add_to_linker_sync(&mut linker, |t| t).unwrap();

    let wasi_ctx = WasiCtxBuilder::new()
        .inherit_stdout()
        .inherit_stderr()
        .build_p1();
    let mut store = Store::new(&engine, wasi_ctx);

    // Note: This is a module built against the preview1 WASI API.
    let module = Module::from_binary(&engine, &wastrumented).unwrap();
    linker.module(&mut store, "main", &module).unwrap();

    // Get function
    let entry_point_function = &linker
        .get(&mut store, "main", "main")
        .unwrap()
        .into_func()
        .unwrap()
        .typed::<(), i32>(&store)
        .unwrap();

    // Benchmark
    // Begin time
    let start = std::time::Instant::now();
    // Invoke
    match entry_point_function.call(&mut store, ()) {
        Err(err) => {
            println!("Error: {err}");
            println!("STDOUT and STDERR were inherited to the host terminal.");
        }
        Ok(res) => {
            println!("STDOUT and STDERR were inherited to the host terminal.");
            println!("Success, outcome = {res:?} (is this (i32.const 2)) ?");
            let end = std::time::Instant::now();
            let duration = end.duration_since(start);
            println!("Execution time: {:?}", duration);
        }
    }
}
