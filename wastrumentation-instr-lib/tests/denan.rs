// Rust STD
use std::path::absolute;

// Wastrumentation imports
use rust_to_wasm_compiler::WasiSupport;
use wastrumentation::{compiler::Compiles, Configuration, PrimaryTarget, Wastrumenter};
use wastrumentation_instr_lib::lib_compile::assemblyscript::compiler::Compiler as ASCompiler;
use wastrumentation_instr_lib::lib_compile::rust::options::{ManifestSource, RustSourceCode};
use wastrumentation_instr_lib::lib_compile::rust::{
    compiler::Compiler,
    options::{CompilerOptions, RustSource, RustSource::Manifest},
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
fn g(a: f32, b: f32, c: f64, d: f64, e: f32) -> (f32, f64) {
    (a + b + e, d + c)
}

#[no_mangle] pub extern "C" fn identity_f32(n: f32) -> f32 { n }
#[no_mangle] pub extern "C" fn identity_f64(n: f64) -> f64 { n }
#[no_mangle] pub extern "C" fn f() -> f32 {
    let (f32, f64) = g(
        core::f32::NAN, // f32
        200.0000000000, // f32
        core::f64::NAN, // f64
        core::f64::NAN, // f64
        -10.0000000000, // f32
    );
    f32 + (f64 as f32)
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

fn compile_input_program() -> Vec<u8> {
    Compiler::setup_compiler()
        .unwrap()
        .compile(&CompilerOptions {
            profile: rust_to_wasm_compiler::Profile::Release,
            source: RustSource::SourceCode(
                WasiSupport::Disabled,
                ManifestSource(INPUT_PROGRAM_MANIFEST.into()),
                RustSourceCode(INPUT_PROGRAM_SOURCE.into()),
            ),
        })
        .unwrap()
}

const PATH_INPUT_ANLYSIS: &str = "./tests/analyses/rust/denan/Cargo.toml";

#[test]
fn test_analysis() {
    /////////////
    // COMPILE //
    /////////////
    let input_program = compile_input_program();

    let mut store = Store::<()>::default();
    let module = Module::from_binary(store.engine(), &input_program).unwrap();
    let instance = Instance::new(&mut store, &module, &[]).unwrap();

    declare_fns_from_wasm! {instance, store,
        f            [   ] [f32],
        identity_f32 [f32] [f32],
        identity_f64 [f64] [f64],
    };

    assert!(wasm_call! {store, f}.is_nan());
    assert_eq!(wasm_call! {store, identity_f32, 0.0}, 0.0);
    assert_eq!(wasm_call! {store, identity_f32, 9.9}, 9.9);
    assert!(wasm_call! {store, identity_f32, f32::NAN}.is_nan());
    assert_eq!(wasm_call! {store, identity_f64, 0.0}, 0.0);
    assert_eq!(wasm_call! {store, identity_f64, 9.9}, 9.9);
    assert!(wasm_call! {store, identity_f64, f64::NAN}.is_nan());

    // CHECK uninstrumentd

    ////////////////
    // INSTRUMENT //
    ////////////////
    let analysis_compiler = Compiler::setup_compiler().expect("Setup Rust compiler");
    let instrumentation_compiler =
        ASCompiler::setup_compiler().expect("Setup AssemblyScript compiler");

    let source = Manifest(WasiSupport::Enabled, absolute(PATH_INPUT_ANLYSIS).unwrap());
    let hooks = vec![
        Hook::GenericApply,
        Hook::Unary,
        Hook::Binary,
        Hook::Const,
        Hook::Local,
        Hook::Global,
        Hook::Load,
        Hook::Store,
    ]
    .into_iter()
    .collect();
    let analysis = RustAnalysisSpec { source, hooks }.into();

    let configuration = Configuration {
        target_indices: None,
        primary_selection: Some(PrimaryTarget::Analysis),
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
        f            [   ] [f32],
        identity_f32 [f32] [f32],
        identity_f64 [f64] [f64],
    };

    // Invoke & assert `denan` did its job!
    assert_eq!(wasm_call! {store, f}, 0.0);
    assert_eq!(wasm_call! {store, identity_f32, 0.0}, 0.0);
    assert_eq!(wasm_call! {store, identity_f32, 9.9}, 9.9);
    assert_eq!(wasm_call! {store, identity_f32, f32::NAN}, 0.0);
    assert_eq!(wasm_call! {store, identity_f64, 0.0}, 0.0);
    assert_eq!(wasm_call! {store, identity_f64, 9.9}, 9.9);
    assert_eq!(wasm_call! {store, identity_f64, f64::NAN}, 0.0);
}
