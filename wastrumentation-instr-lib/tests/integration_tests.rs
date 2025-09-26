// Rust STD
use std::path::absolute;

use indoc::formatdoc;
// Wastrumentation imports
use rust_to_wasm_compiler::{Profile, WasiSupport};
use wastrumentation::{compiler::Compiles, Configuration, PrimaryTarget, Wastrumenter};
use wastrumentation_lang_assemblyscript::compile::compiler::Compiler as ASCompiler;
use wastrumentation_lang_rust::compile::options::{ManifestSource, RustSource, RustSourceCode};
use wastrumentation_lang_rust::compile::{compiler::Compiler, options::RustSource::Manifest};
use wastrumentation_lang_rust::generate::analysis::Hook;
use wastrumentation_lang_rust::generate::analysis::RustAnalysisSpec;

// Wasmtime imports
use wasmtime::{Config, Engine, Instance, Linker, Module, Store};
use wasmtime_wasi::p1::{self as preview1, WasiP1Ctx};
use wasmtime_wasi::WasiCtxBuilder;

// Bring macros in scope
mod wasmtime_macros;

mod integration_util;
use integration_util::*;

const TAINT_INPUT_SOURCE: &str = r#"
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
"#;

#[test]
fn test_analysis_denan() {
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
    const SOURCE: Source = Source::Rust(
        INPUT_PROGRAM_SOURCE,
        WasiSupport::Disabled,
        Profile::Release,
    );

    /////////////
    // COMPILE //
    /////////////
    let input_program = SOURCE.to_input_program();

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

    const PATH_INPUT_ANLYSIS: &str = "./tests/analyses/rust/denan/Cargo.toml";
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

    let stdout = wasmtime_wasi::p2::pipe::MemoryOutputPipe::new(usize::MAX);
    let stderr = wasmtime_wasi::p2::pipe::MemoryOutputPipe::new(usize::MAX);

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

#[test]
fn test_analysis_forward_guiicons() {
    //////////////////////////
    // COMPILE & INSTRUMENT //
    //////////////////////////

    let analysis_compiler = Compiler::setup_compiler().expect("Setup Rust compiler");
    let instrumentation_compiler =
        ASCompiler::setup_compiler().expect("Setup AssemblyScript compiler");

    const PATH_INPUT_ANLYSIS: &str = "./tests/analyses/rust/memory-tracing/Cargo.toml";
    let source = Manifest(WasiSupport::Disabled, absolute(PATH_INPUT_ANLYSIS).unwrap());
    let hooks = vec![Hook::Load, Hook::Store].into_iter().collect();
    let analysis = RustAnalysisSpec { source, hooks }.into();

    let configuration = Configuration {
        target_indices: None,
        primary_selection: Some(PrimaryTarget::Target),
    };

    // Read input program
    let input_program = include_bytes!("guiicons.wasm").to_vec();

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
    declare_fns_from_wasm!(instance, store, total_accesses [] [i64]);

    wasm_call!(store, _start);
    assert_eq!(wasm_call!(store, total_accesses), 78790915);
}

#[test]
fn test_analysis_safe_heap() {
    const SOURCE: Source =
        Source::Rust(TAINT_INPUT_SOURCE, WasiSupport::Disabled, Profile::Release);

    // TODO: Currently this test is missing a misaligned memory read!
    //////////////////////////
    // COMPILE & INSTRUMENT //
    //////////////////////////

    let analysis_compiler = Compiler::setup_compiler().expect("Setup Rust compiler");
    let instrumentation_compiler =
        ASCompiler::setup_compiler().expect("Setup AssemblyScript compiler");

    const PATH_INPUT_ANLYSIS: &str = "./tests/analyses/rust/safe-heap/Cargo.toml";
    let source = Manifest(WasiSupport::Enabled, absolute(PATH_INPUT_ANLYSIS).unwrap());
    let hooks = vec![Hook::Load, Hook::Store].into_iter().collect();
    let analysis = RustAnalysisSpec { source, hooks }.into();

    let configuration = Configuration {
        target_indices: None,
        primary_selection: Some(PrimaryTarget::Analysis),
    };

    let input_program = SOURCE.to_input_program();
    let wastrumenter = Wastrumenter::new(instrumentation_compiler.into(), analysis_compiler.into());
    let wastrumented = wastrumenter
        .wastrument(&input_program, analysis, &configuration)
        .expect("Wastrumentation should succeed");

    /////////////////////
    // WASMTIME ENGINE //
    /////////////////////

    let stdout = wasmtime_wasi::p2::pipe::MemoryOutputPipe::new(usize::MAX);
    let stderr = wasmtime_wasi::p2::pipe::MemoryOutputPipe::new(usize::MAX);

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

#[test]
fn test_analysis_memory_introspection() {
    const INPUT_PROGRAM_SOURCE: &str = r#"
    #[no_mangle] pub extern "C" fn reserve_bytes(n: i64) {
        let memory = vec![0u8; n.try_into().unwrap()];
        std::mem::forget(memory);
    }
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

    const SOURCE: Source = Source::Rust(
        INPUT_PROGRAM_SOURCE,
        WasiSupport::Disabled,
        rust_to_wasm_compiler::Profile::Dev,
    );
    /////////////
    // COMPILE //
    /////////////
    let input_program = SOURCE.to_input_program();

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

    let manifest_source = {
        let wastrumentation_rs_stdlib =
            absolute("./tests/analyses/rust/wastrumentation-rs-stdlib").unwrap();
        let wastrumentation_rs_stdlib = wastrumentation_rs_stdlib.to_string_lossy();

        formatdoc! { r#"
            package.name = "rust-wasp-call-stack"
            package.version = "0.1.0"
            package.edition = "2021"
            lib.crate-type = ["cdylib"]
            dependencies.wee_alloc = "0.4.5"
            dependencies.wastrumentation-rs-stdlib = {{ path = "{wastrumentation_rs_stdlib}", features = ["std"] }}
            profile.release.strip = true
            profile.release.lto = true
            profile.release.panic = "abort"
            [workspace]"#
        }
    };

    let source = RustSource::SourceCode(
        WasiSupport::Enabled,
        ManifestSource(manifest_source),
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

    let stdout = wasmtime_wasi::p2::pipe::MemoryOutputPipe::new(usize::MAX);
    let stderr = wasmtime_wasi::p2::pipe::MemoryOutputPipe::new(usize::MAX);

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

#[test]
fn test_analysis_forward_pacalc() {
    //////////////////////////
    // COMPILE & INSTRUMENT //
    //////////////////////////

    let analysis_compiler = Compiler::setup_compiler().expect("Setup Rust compiler");
    let instrumentation_compiler =
        ASCompiler::setup_compiler().expect("Setup AssemblyScript compiler");

    const PATH_INPUT_ANLYSIS: &str = "./tests/analyses/rust/forward/Cargo.toml";
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

#[test]
fn test_analysis_logging() {
    const SOURCE: Source =
        Source::Rust(TAINT_INPUT_SOURCE, WasiSupport::Disabled, Profile::Release);

    const EXPECTED_ANALYSIS_STDOUT: &str = indoc::indoc! { r#"
    [ANALYSIS:] apply (pre) WasmFunction {
        uninstr_idx: 0,
        sig_pointer: 1179632,
    }(RuntimeValues {
        argc: 1,
        resc: 1,
        sigv: 1179632,
        signature_types: [
            I32,
            I32,
        ],
    })
    [ANALYSIS:] block pre [block_input_count: BlockInputCount(0), block_arity: BlockArity(0)], location: Location { instr_index: 0, funct_index: 0 }
    [ANALYSIS:] local generic I32(
        123,
    ) @ LocalIndex(
        0,
    ) : Get, location: Location { instr_index: 0, funct_index: 1 }
    [ANALYSIS:] br_if ParameterBrIfCondition(
        123,
    ) to ParameterBrIfLabel(
        0,
    ), location: Location { instr_index: 0, funct_index: 2 }
    [ANALYSIS:] const_ generic I32(
        0,
    ), location: Location { instr_index: 0, funct_index: 6 }
    [ANALYSIS:] const_ generic I32(
        0,
    ), location: Location { instr_index: 0, funct_index: 7 }
    [ANALYSIS:] load generic I32Load @ (CONST LoadOffset(
        1048576,
    ) + LoadIndex(
        0,
    )) -> I32(
        0,
    ), location: Location { instr_index: 0, funct_index: 8 }
    [ANALYSIS:] const_ generic I32(
        4,
    ), location: Location { instr_index: 0, funct_index: 9 }
    [ANALYSIS:] binary generic I32Add I32(
        0,
    ) I32(
        4,
    ), location: Location { instr_index: 0, funct_index: 10 }
    [ANALYSIS:] store generic I32Store @ (CONST StoreOffset(
        1048576,
    ) + StoreIndex(
        0,
    )) <- I32(
        4,
    ), location: Location { instr_index: 0, funct_index: 11 }
    [ANALYSIS:] local generic I32(
        123,
    ) @ LocalIndex(
        0,
    ) : Get, location: Location { instr_index: 0, funct_index: 12 }
    [ANALYSIS:] const_ generic I32(
        20,
    ), location: Location { instr_index: 0, funct_index: 13 }
    [ANALYSIS:] binary generic I32Mul I32(
        123,
    ) I32(
        20,
    ), location: Location { instr_index: 0, funct_index: 14 }
    [ANALYSIS:] const_ generic I32(
        35,
    ), location: Location { instr_index: 0, funct_index: 15 }
    [ANALYSIS:] binary generic I32Add I32(
        2460,
    ) I32(
        35,
    ), location: Location { instr_index: 0, funct_index: 16 }
    [ANALYSIS:] const_ generic I32(
        1,
    ), location: Location { instr_index: 0, funct_index: 17 }
    [ANALYSIS:] binary generic I32ShrS I32(
        2495,
    ) I32(
        1,
    ), location: Location { instr_index: 0, funct_index: 18 }
    [ANALYSIS:] local generic I32(
        1247,
    ) @ LocalIndex(
        0,
    ) : Tee, location: Location { instr_index: 0, funct_index: 19 }
    [ANALYSIS:] local generic I32(
        1247,
    ) @ LocalIndex(
        0,
    ) : Get, location: Location { instr_index: 0, funct_index: 20 }
    [ANALYSIS:] binary generic I32Mul I32(
        1247,
    ) I32(
        1247,
    ), location: Location { instr_index: 0, funct_index: 21 }
    [ANALYSIS:] apply (post) WasmFunction {
        uninstr_idx: 0,
        sig_pointer: 1179632,
    }(RuntimeValues {
        argc: 1,
        resc: 1,
        sigv: 1179632,
        signature_types: [
            I32,
            I32,
        ],
    }) = RuntimeValues {
        argc: 1,
        resc: 1,
        sigv: 1179632,
        signature_types: [
            I32,
            I32,
        ],
    }
    "# };

    //////////////////////////
    // COMPILE & INSTRUMENT //
    //////////////////////////

    let analysis_compiler = Compiler::setup_compiler().expect("Setup Rust compiler");
    let instrumentation_compiler = Compiler::setup_compiler().expect("Setup Rust compiler");

    let input_program = SOURCE.to_input_program();

    let source = Manifest(
        rust_to_wasm_compiler::WasiSupport::Enabled,
        absolute("./tests/analyses/rust/logging/Cargo.toml").unwrap(),
    );

    let hooks = Hook::all_hooks();
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
    let module = Module::from_binary(&engine, &wastrumented).unwrap();

    linker.module(&mut store, "main", &module).unwrap();

    // Get & invoke function
    declare_fns_from_linker! {linker, store, "main", f [i32] [i32]}

    assert_eq!(wasm_call!(store, f, 123), 1555009);

    assert_eq!(
        EXPECTED_ANALYSIS_STDOUT,
        String::from_utf8_lossy(&stdout.contents())
    );

    assert_eq!(wasm_call!(store, f, 0), 12345);
}
