// Rust STD
use std::path::absolute;

// Wastrumentation imports
use rust_to_wasm_compiler::WasiSupport;
use wastrumentation::{compiler::Compiles, Configuration, PrimaryTarget, Wastrumenter};
use wastrumentation_instr_lib::lib_compile::assemblyscript::compiler::Compiler as ASCompiler;
use wastrumentation_instr_lib::lib_compile::rust::{
    compiler::Compiler, options::RustSource::Manifest,
};
use wastrumentation_instr_lib::lib_gen::analysis::rust::Hook;
use wastrumentation_instr_lib::lib_gen::analysis::rust::RustAnalysisSpec;

// Wasmtime imports
use wasmtime::{Engine, Instance, Module, Store};

// Bring macro's in scope
mod wasmtime_macros;

const PATH_INPUT_ANLYSIS: &str = "./tests/analyses/rust/forward/Cargo.toml";

#[test]
fn forward_pacalc() {
    //////////////////////////
    // COMPILE & INSTRUMENT //
    //////////////////////////

    let analysis_compiler = Compiler::setup_compiler().expect("Setup Rust compiler");
    let instrumentation_compiler =
        ASCompiler::setup_compiler().expect("Setup AssemblyScript compiler");

    let source = Manifest(WasiSupport::Disabled, absolute(PATH_INPUT_ANLYSIS).unwrap());
    let hooks = Hook::all_hooks();
    let analysis = RustAnalysisSpec { source, hooks }.into();

    let configuration = Configuration {
        target_indices: None,
        primary_selection: Some(PrimaryTarget::Target),
    };

    // Read input program
    let input_program = include_bytes!("pacalc.wasm").to_vec();

    // Perform instrumentation
    let wastrumenter = Wastrumenter::new(instrumentation_compiler.into(), analysis_compiler.into());
    let wastrumented = wastrumenter
        .wastrument(&input_program, analysis, &configuration)
        .expect("Wastrumentation should succeed");

    /////////////////////
    // WASMTIME ENGINE //
    /////////////////////
    let engine = Engine::default();
    let mut store = Store::<()>::new(&engine, ());
    let module = Module::from_binary(&engine, &wastrumented).unwrap();
    let instance = Instance::new(&mut store, &module, &[]).unwrap();

    // Fetch & Invoke
    declare_fns_from_wasm!(instance, store, _start [] []);
    wasm_call!(store, _start);
}
