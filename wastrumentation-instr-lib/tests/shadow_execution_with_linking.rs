// Rust STD
use std::path::absolute;

// Wastrumentation imports
use rust_to_wasm_compiler::WasiSupport;
use wastrumentation::analysis::ProcessedAnalysis;
use wastrumentation::{compiler::Compiles, Configuration, PrimaryTarget, Wastrumenter};
use wastrumentation_instr_lib::lib_compile::rust::Rust;
use wastrumentation_instr_lib::lib_compile::rust::{
    compiler::Compiler, options::RustSource::Manifest,
};
use wastrumentation_instr_lib::lib_gen::analysis::rust::Hook;
use wastrumentation_instr_lib::lib_gen::analysis::rust::RustAnalysisSpec;

// Wasmtime imports
use wasmtime::{Engine, Linker, Module, Store};
use wasmtime_wasi::preview1::{self, WasiP1Ctx};
use wasmtime_wasi::WasiCtxBuilder;

const WAT_MGM: &str = r#"
(module $Mgm
  (memory (export "memory") 2) ;; initial size is 1
  (func (export "grow") (result i32) (memory.grow (i32.const 1)))
)
"#;

const WAT_MGIM1: &str = r#"
(module $Mgim1
  ;; imported memory limits should match, because external memory size is 2 now
  (memory (export "memory") (import "grown-memory" "memory") 2)
  (func (export "grow") (result i32) (memory.grow (i32.const 1)))
)
"#;

const PATH_INPUT_ANLYSIS: &str = "./tests/analyses/rust/shadow-execution-analysis/Cargo.toml";

fn compile_input_programs() -> (Vec<u8>, Vec<u8>) {
    let wat_mgm = wat::parse_str(WAT_MGM).unwrap();
    let wat_mgim1 = wat::parse_str(WAT_MGIM1).unwrap();
    (wat_mgm, wat_mgim1)
}

#[test]
fn test_analysis() {
    //////////////////////////
    // COMPILE & INSTRUMENT //
    //////////////////////////

    let analysis_compiler = Compiler::setup_compiler().expect("Setup Rust compiler");
    let instrumentation_compiler = Compiler::setup_compiler().expect("Setup Rust compiler");

    let analysis = || {
        let source = Manifest(WasiSupport::Enabled, absolute(PATH_INPUT_ANLYSIS).unwrap());
        let hooks = Hook::all_hooks();
        let analysis: ProcessedAnalysis<Rust> = RustAnalysisSpec { source, hooks }.into();
        analysis
    };

    let configuration = Configuration {
        target_indices: None,
        primary_selection: Some(PrimaryTarget::Analysis),
    };

    let (wat_mgm, wat_mgim1) = compile_input_programs();
    let wastrumenter = Wastrumenter::new(instrumentation_compiler.into(), analysis_compiler.into());

    let wastrumented_mgm = wastrumenter
        .wastrument(&wat_mgm, analysis(), &configuration)
        .unwrap();
    // let _ = wastrumented_mgm;
    // let wastrumented_mgm = wat_mgm; // TODO: remove this line?
    let wastrumented_mgim1 = wastrumenter
        .wastrument(&wat_mgim1, analysis(), &configuration)
        .unwrap();
    // let _ = wastrumented_mgim1;
    // let wastrumented_mgim1 = wat_mgim1; // TODO: remove this line?

    /////////////////////
    // WASMTIME ENGINE //
    /////////////////////

    let stdout = wasmtime_wasi::p2::pipe::MemoryOutputPipe::new(usize::MAX);
    let stderr = wasmtime_wasi::p2::pipe::MemoryOutputPipe::new(usize::MAX);

    // Construct the wasm engine
    let engine = Engine::default();

    // Add the WASI preview1 API to the linker (will be implemented in terms of the preview2 API)
    let mut linker: Linker<WasiP1Ctx> = Linker::new(&engine);
    preview1::add_to_linker_sync(&mut linker, |t| t).unwrap();

    // Add capabilities (e.g. filesystem access) to the WASI preview2 context here.
    // Here only stdio is inherited, but see docs of `WasiCtxBuilder` for more.
    let wasi_ctx = WasiCtxBuilder::new()
        .stdout(stdout.clone())
        .stderr(stderr.clone())
        .build_p1();

    let mut store = Store::new(&engine, wasi_ctx);

    // Note: This is a module built against the preview1 WASI API.
    let module_mgm = Module::from_binary(&engine, &wastrumented_mgm).unwrap();
    let module_mgim1 = Module::from_binary(&engine, &wastrumented_mgim1).unwrap();

    linker
        .module(&mut store, "grown-memory", &module_mgm)
        .unwrap();

    let grown_memory_inner_memory = &linker
        .get(&mut store, "grown-memory", "memory")
        .unwrap()
        .into_memory()
        .unwrap();

    let get_mem_size = || grown_memory_inner_memory.size(&store);
    println!("mem size: {}", get_mem_size());

    // Invoke `grow`
    let grown_memory_grow = &linker
        .get(&mut store, "grown-memory", "grow")
        .unwrap()
        .into_func()
        .unwrap()
        .typed::<(), i32>(&store)
        .unwrap()
        .call(&mut store, ())
        .unwrap();

    println!("grown_memory_grow: {grown_memory_grow:?}");

    linker
        .module(&mut store, "grown-imported-memory", &module_mgim1)
        .unwrap();

    // Invoke `grow`
    let grown_imported_memory = &linker
        .get(&mut store, "grown-imported-memory", "grow")
        .unwrap()
        .into_func()
        .unwrap()
        .typed::<(), i32>(&store)
        .unwrap()
        .call(&mut store, ());

    // Invoke
    match grown_imported_memory {
        Err(err) => {
            println!("Error: {err}");
            println!("STDOUT: \n{}", String::from_utf8_lossy(&stdout.contents()));
            println!("STDERR: \n{}", String::from_utf8_lossy(&stderr.contents()));
        }
        Ok(res) => {
            println!("STDOUT: \n{}", String::from_utf8_lossy(&stdout.contents()));
            println!("Success, outcome = {res} (is this (i32.const 2)) ?");
        }
    };
}
