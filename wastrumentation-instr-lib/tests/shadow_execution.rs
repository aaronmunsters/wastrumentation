// Rust STD
use std::path::{absolute, PathBuf};

// Wastrumentation imports
use rust_to_wasm_compiler::WasiSupport;
use wastrumentation::{compiler::Compiles, Configuration, PrimaryTarget, Wastrumenter};
use wastrumentation_instr_lib::lib_compile::assemblyscript::compiler::Compiler as ASCompiler;
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

fn compile_input_program(input_program: impl Into<PathBuf>) -> Vec<u8> {
    Compiler::setup_compiler()
        .unwrap()
        .compile(&CompilerOptions {
            profile: rust_to_wasm_compiler::Profile::Release,
            source: RustSource::Manifest(
                WasiSupport::Disabled,
                absolute(input_program.into()).unwrap(),
            ),
        })
        .unwrap()
}

const PATH_INPUT_PROGRAM: &str = "./tests/input-programs/rust/rust-taint-input-program/Cargo.toml";
const PATH_INPUT_ANLYSIS: &str = "./tests/analyses/rust/shadow-execution-analysis/Cargo.toml";

#[test]
fn test_analysis() {
    //////////////////////////
    // COMPILE & INSTRUMENT //
    //////////////////////////

    let analysis_compiler = Compiler::setup_compiler().expect("Setup Rust compiler");
    let instrumentation_compiler =
        ASCompiler::setup_compiler().expect("Setup AssemblyScript compiler");
    // let instrumentation_compiler = Compiler::setup_compiler().expect("Setup Rust compiler");

    let source = Manifest(WasiSupport::Enabled, absolute(PATH_INPUT_ANLYSIS).unwrap());
    let hooks = Hook::all_hooks();
    let analysis = RustAnalysisSpec { source, hooks }.into();

    let configuration = Configuration {
        target_indices: None,
        primary_selection: Some(PrimaryTarget::Analysis),
    };

    let input_program = compile_input_program(PATH_INPUT_PROGRAM);
    let wastrumenter = Wastrumenter::new(instrumentation_compiler.into(), analysis_compiler.into());
    let wastrumented = wastrumenter
        .wastrument(&input_program, analysis, &configuration)
        .expect("Wastrumentation should succeed");

    // let mut file = std::fs::File::create_new("./wastrumented.wasm").unwrap();
    // std::io::Write::write_all(&mut file, &wastrumented).unwrap();

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

    // Get function
    let entry_point_function = &linker
        .get(&mut store, "main", "f")
        .unwrap()
        .into_func()
        .unwrap()
        .typed::<i32, i32>(&store)
        .unwrap();

    // Invoke
    match entry_point_function.call(&mut store, 10) {
        Err(err) => {
            println!("Error: {err}");
            println!("STDOUT: \n{}", String::from_utf8_lossy(&stdout.contents()));
            println!("STDERR: \n{}", String::from_utf8_lossy(&stderr.contents()));
        }
        Ok(res) => {
            println!("STDOUT: \n{}", String::from_utf8_lossy(&stdout.contents()));
            println!("Success, outcome = {res:?} (is this (i32.const 2)) ?");
        }
    };
}

#[test]
fn test_analysis_branch() {
    const WASM_CODE: &str = r#"
    (module
        (export "main" (func $main))
        (func (;0;) $add (param $a i32) (param $b i32) (result i32)
            local.get $a
            local.get $b
            i32.add
        )
        (func (;1;) $main (param $a i32) (result i32)
            block              ;; if $a < 10 { return 100 }
                i32.const 10
                local.get $a
                i32.lt_s
                br_if 0
                i32.const 100
                return
            end                ;; else { $add(111, $a); return $a }
            i32.const 111
            local.get $a
            call $add
            drop
            local.get $a
        )
    )
    "#;

    //////////////////////////
    // COMPILE & INSTRUMENT //
    //////////////////////////

    let analysis_compiler = Compiler::setup_compiler().expect("Setup Rust compiler");
    let instrumentation_compiler =
        ASCompiler::setup_compiler().expect("Setup AssemblyScript compiler");
    // let instrumentation_compiler = Compiler::setup_compiler().expect("Setup Rust compiler");

    let source = Manifest(WasiSupport::Enabled, absolute(PATH_INPUT_ANLYSIS).unwrap());
    let hooks = Hook::all_hooks();
    let analysis = RustAnalysisSpec { source, hooks }.into();

    let configuration = Configuration {
        target_indices: None,
        primary_selection: Some(PrimaryTarget::Analysis),
    };

    let input_program = wat::parse_bytes(WASM_CODE.as_bytes()).unwrap().to_vec();
    let wastrumenter = Wastrumenter::new(instrumentation_compiler.into(), analysis_compiler.into());
    let wastrumented = wastrumenter
        .wastrument(&input_program, analysis, &configuration)
        .expect("Wastrumentation should succeed");

    // let mut file = std::fs::File::create_new("./wastrumented.wasm").unwrap();
    // std::io::Write::write_all(&mut file, &wastrumented).unwrap();

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

    // Get function
    let entry_point_function = &linker
        .get(&mut store, "main", "main")
        .unwrap()
        .into_func()
        .unwrap()
        .typed::<i32, i32>(&store)
        .unwrap();

    // Invoke
    match entry_point_function.call(&mut store, 15) {
        Err(err) => {
            println!("Error: {err}");
            println!("STDOUT: \n{}", String::from_utf8_lossy(&stdout.contents()));
            println!("STDERR: \n{}", String::from_utf8_lossy(&stderr.contents()));
        }
        Ok(res) => {
            println!("STDOUT: \n{}", String::from_utf8_lossy(&stdout.contents()));
            println!("Success, outcome = {res:?} (is this (i32.const 2)) ?");
        }
    };
}
