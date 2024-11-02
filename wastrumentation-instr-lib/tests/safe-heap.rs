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
use wasmtime::{Config, Engine, Linker, Module, Store};
use wasmtime_wasi::preview1::{self, WasiP1Ctx};
use wasmtime_wasi::WasiCtxBuilder;

// Bring macro's in scope
mod wasmtime_macros;

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

const PATH_INPUT_ANLYSIS: &str = "./tests/analyses/rust/safe-heap/Cargo.toml";

#[test]
fn test_analysis() {
    // TODO: Currently this test is missing a misaligned memory read!
    //////////////////////////
    // COMPILE & INSTRUMENT //
    //////////////////////////

    let analysis_compiler = Compiler::setup_compiler().expect("Setup Rust compiler");
    let instrumentation_compiler =
        ASCompiler::setup_compiler().expect("Setup AssemblyScript compiler");

    let source = Manifest(WasiSupport::Enabled, absolute(PATH_INPUT_ANLYSIS).unwrap());
    let hooks = vec![Hook::Load, Hook::Store].into_iter().collect();
    let analysis = RustAnalysisSpec { source, hooks }.into();

    let configuration = Configuration {
        target_indices: None,
        primary_selection: Some(PrimaryTarget::Analysis),
    };

    let input_program = compile_input_program();
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

    // Fetch & Invoke
    declare_fns_from_linker! { linker, store, "main", f [i32] [i32]};
    assert_eq!(wasm_call! {store, f, 10}, 13689);
}

const INPUT_PROGRAM_SOURCE: &str = r#"
static mut GLOBAL_COUNT_TWO: i32 = 0;

#[no_mangle]
pub extern "C" fn g(input: i32) -> i32 {
    unsafe { GLOBAL_COUNT_TWO += 2 };
    let a = input + 1;
    let b = a * 10;
    let c = b - 25;
    #[allow(arithmetic_overflow)]
    return c >> 1;
}

#[no_mangle]
pub extern "C" fn f(input: i32) -> i32 {
    if input == 0 {
        return 12345;
    };
    let x = 2 * input;
    let y = 5 + x;
    let z = g(y) * g(y);
    return z;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f() {
        assert_eq!(1555009, f(123));
    }
}
"#;

const INPUT_PROGRAM_MANIFEST: &str = r#"
[package]
name = "rust-taint-input-program"
version = "0.1.0"
edition = "2021"
[lib]
crate-type = ["cdylib"]
[profile.release]
strip = true
lto = true
panic = "abort"
[dependencies]
[workspace]
"#;
