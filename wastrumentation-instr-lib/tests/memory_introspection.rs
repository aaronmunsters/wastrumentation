// Rust STD
use std::path::absolute;

// Wastrumentation imports
use rust_to_wasm_compiler::WasiSupport;
use wastrumentation::{compiler::Compiles, Configuration, PrimaryTarget, Wastrumenter};
use wastrumentation_instr_lib::lib_compile::assemblyscript::compiler::Compiler as ASCompiler;
use wastrumentation_instr_lib::lib_compile::rust::options::{ManifestSource, RustSourceCode};
use wastrumentation_instr_lib::lib_compile::rust::{
    compiler::Compiler,
    options::{CompilerOptions, RustSource},
};
use wastrumentation_instr_lib::lib_gen::analysis::rust::Hook;
use wastrumentation_instr_lib::lib_gen::analysis::rust::RustAnalysisSpec;

// Wasmtime imports
use wasmtime::{Config, Engine, Instance, Linker, Module, Store};
use wasmtime_wasi::preview1::{self, WasiP1Ctx};
use wasmtime_wasi::WasiCtxBuilder;

// Bring macros in scope
mod wasmtime_macros;

const INPUT_PROGRAM_SOURCE: &str = r#"
#[no_mangle] pub extern "C" fn reserve_bytes(n: i64) {
    let memory = vec![0u8; n.try_into().unwrap()];
    std::mem::forget(memory);
}
"#;

const INPUT_PROGRAM_MANIFEST: &str = r#"
package.name = "rust-denan-input-program"
package.version = "0.1.0"
package.edition = "2021"
lib.crate-type = ["cdylib"]
profile.release.strip = true
profile.release.lto = true
profile.release.panic = "abort"
[workspace]
"#;

const ANALYSIS_SOURCE_CODE: &str = r#"
use wastrumentation_rs_stdlib::*;
static mut LAST_MEM_READ: i32 = 0;
#[no_mangle] pub extern "C" fn report_last_mem_size() -> i32 {
    unsafe { LAST_MEM_READ } }
advice! { apply (func: WasmFunction, _args: MutDynArgs, _ress: MutDynResults) {
        func.apply();
        unsafe { LAST_MEM_READ = base_memory_size(0) }; } }
"#;

fn compile_input_program() -> Vec<u8> {
    Compiler::setup_compiler()
        .unwrap()
        .compile(&CompilerOptions {
            profile: rust_to_wasm_compiler::Profile::Dev,
            source: RustSource::SourceCode(
                WasiSupport::Disabled,
                ManifestSource(INPUT_PROGRAM_MANIFEST.into()),
                RustSourceCode(INPUT_PROGRAM_SOURCE.into()),
            ),
        })
        .unwrap()
}

fn get_analysis_manifest_source() -> String {
    let wastrumentation_rs_stdlib =
        absolute("./tests/analyses/rust/wastrumentation-rs-stdlib").unwrap();
    let wastrumentation_rs_stdlib = wastrumentation_rs_stdlib.to_string_lossy();
    format!(
        r#"
        package.name = "rust-wasp-call-stack"
        package.version = "0.1.0"
        package.edition = "2021"
        lib.crate-type = ["cdylib"]
        dependencies.wee_alloc = "0.4.5"
        dependencies.wastrumentation-rs-stdlib = {{ path = "{wastrumentation_rs_stdlib}", features = ["std"] }}
        profile.release.strip = true
        profile.release.lto = true
        profile.release.panic = "abort"
        [workspace]
        "#
    )
}

#[test]
fn test_analysis() {
    /////////////
    // COMPILE //
    /////////////
    let input_program = compile_input_program();

    let mut store = Store::<()>::default();
    let module = Module::from_binary(store.engine(), &input_program).unwrap();
    let instance = Instance::new(&mut store, &module, &[]).unwrap();

    declare_fns_from_wasm! { instance, store, reserve_bytes [i64] [] };
    wasm_call! {store, reserve_bytes, 1};
    wasm_call! {store, reserve_bytes, 5_000_000};

    ////////////////
    // INSTRUMENT //
    ////////////////
    let analysis_compiler = Compiler::setup_compiler().expect("Setup Rust compiler");
    let instrumentation_compiler =
        ASCompiler::setup_compiler().expect("Setup AssemblyScript compiler");

    let source = RustSource::SourceCode(
        WasiSupport::Enabled,
        ManifestSource(get_analysis_manifest_source()),
        RustSourceCode(ANALYSIS_SOURCE_CODE.into()),
    );
    let hooks = vec![Hook::GenericApply].into_iter().collect();
    let analysis = RustAnalysisSpec { source, hooks }.into();

    let configuration = Configuration {
        target_indices: None,
        // NOTE: here, the primary target [IS] important since
        // the reported `base_memory_size(0)` in the analysis
        // must target the input program!
        primary_selection: Some(PrimaryTarget::Target),
    };

    let wastrumenter = Wastrumenter::new(instrumentation_compiler.into(), analysis_compiler.into());
    let wastrumented = wastrumenter
        .wastrument(&input_program, analysis, &configuration)
        .expect("Wastrumentation should succeed");

    /////////////////////
    // WASMTIME ENGINE //
    /////////////////////

    let stdout = wasmtime_wasi::pipe::MemoryOutputPipe::new(usize::MAX);
    let stderr = wasmtime_wasi::pipe::MemoryOutputPipe::new(usize::MAX);

    // Construct the wasm engine
    let mut config = Config::new();
    config
        .wasm_backtrace(true)
        .wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Enable);
    let engine = Engine::new(&config).unwrap();

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
    let module = Module::from_binary(&engine, &wastrumented).unwrap();
    linker.module(&mut store, "main", &module).unwrap();

    declare_fns_from_linker! { linker, store, "main",
        reserve_bytes        [i64] [   ],
        report_last_mem_size [   ] [i32],
    };

    // Before first call, memory is 'empty'
    assert_eq!(wasm_call! {store, report_last_mem_size}, 0);
    wasm_call! {store, reserve_bytes, 1};
    // after first call, needed memory is reserved [__...__]
    assert_eq!(wasm_call! {store, report_last_mem_size}, 18);
    wasm_call! {store, reserve_bytes, 10};
    // subsequent call does not grow beyond rsrvd  [__...__]
    assert_eq!(wasm_call! {store, report_last_mem_size}, 18);
    (0..5000).for_each(|_| wasm_call! {store, reserve_bytes, 10});
    // sufficient subsequent calls do grow beyond: [__...__+]
    assert_eq!(wasm_call! {store, report_last_mem_size}, 19);
    (0..5000).for_each(|_| wasm_call! {store, reserve_bytes, 10});
    // sufficient subsequent calls do grow beyond: [__...__++]
    assert_eq!(wasm_call! {store, report_last_mem_size}, 20);
    wasm_call! {store, reserve_bytes, 500_000};
    // many more subsequent calls do grow beyond:  [__...__++++++++++]
    assert_eq!(wasm_call! {store, report_last_mem_size}, 28);
}
