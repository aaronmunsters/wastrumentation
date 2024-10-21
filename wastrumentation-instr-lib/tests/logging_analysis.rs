// Rust STD
use std::path::absolute;

// Wastrumentation imports
use wastrumentation::{compiler::Compiles, Configuration, PrimaryTarget, Wastrumenter};
use wastrumentation_instr_lib::lib_compile::rust::{
    compiler::Compiler,
    options::{CompilerOptions, ManifestSource, RustSource, RustSource::Manifest, RustSourceCode},
};
use wastrumentation_instr_lib::lib_gen::analysis::rust::Hook;
use wastrumentation_instr_lib::lib_gen::analysis::rust::RustAnalysisSpec;

// Wasmtime imports
use wasmtime::{Engine, Linker, Module, Store};
use wasmtime_wasi::preview1::{self, WasiP1Ctx};
use wasmtime_wasi::WasiCtxBuilder;

const MANIFEST_SOURCE: &str = r#"
package.name = "demo"
lib.crate-type = ["cdylib"]
profile.release.strip = true
profile.release.lto = true
profile.release.panic = "abort"
[workspace]
"#;

const RUST_SOURCE_HELLO_WORLD: &str = r#"
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

const EXPECTED_ANALYSIS_STDOUT: &str = indoc::indoc! { r#"
[ANALYSIS:] apply (pre) WasmFunction {
    uninstr_idx: 1,
    sig_pointer: 1179640,
}(RuntimeValues {
    argc: 1,
    resc: 1,
    sigv: 1179640,
    signature_types: [
        I32,
        I32,
    ],
    signature_offsets: [
        0,
        4,
    ],
})
[ANALYSIS:] block pre [block_input_count: BlockInputCount(0), block_arity: BlockArity(0)], location: Location { instr_index: 1, funct_index: 0 }
[ANALYSIS:] local generic I32(
    123,
) @ LocalIndex(
    0,
) : Get, location: Location { instr_index: 1, funct_index: 1 }
[ANALYSIS:] br_if ParameterBrIfCondition(
    123,
) to ParameterBrIfLabel(
    0,
), location: Location { instr_index: 1, funct_index: 2 }
[ANALYSIS:] const_ generic I32(
    0,
), location: Location { instr_index: 1, funct_index: 6 }
[ANALYSIS:] const_ generic I32(
    0,
), location: Location { instr_index: 1, funct_index: 7 }
[ANALYSIS:] load generic I32Load @ (CONST LoadOffset(
    1048576,
) + LoadIndex(
    0,
)) -> I32(
    0,
), location: Location { instr_index: 1, funct_index: 8 }
[ANALYSIS:] const_ generic I32(
    4,
), location: Location { instr_index: 1, funct_index: 9 }
[ANALYSIS:] binary generic I32Add I32(
    0,
) I32(
    4,
), location: Location { instr_index: 1, funct_index: 10 }
[ANALYSIS:] store generic I32Store @ (CONST StoreOffset(
    1048576,
) + StoreIndex(
    0,
)) <- I32(
    4,
), location: Location { instr_index: 1, funct_index: 11 }
[ANALYSIS:] local generic I32(
    123,
) @ LocalIndex(
    0,
) : Get, location: Location { instr_index: 1, funct_index: 12 }
[ANALYSIS:] const_ generic I32(
    20,
), location: Location { instr_index: 1, funct_index: 13 }
[ANALYSIS:] binary generic I32Mul I32(
    123,
) I32(
    20,
), location: Location { instr_index: 1, funct_index: 14 }
[ANALYSIS:] const_ generic I32(
    35,
), location: Location { instr_index: 1, funct_index: 15 }
[ANALYSIS:] binary generic I32Add I32(
    2460,
) I32(
    35,
), location: Location { instr_index: 1, funct_index: 16 }
[ANALYSIS:] const_ generic I32(
    1,
), location: Location { instr_index: 1, funct_index: 17 }
[ANALYSIS:] binary generic I32ShrS I32(
    2495,
) I32(
    1,
), location: Location { instr_index: 1, funct_index: 18 }
[ANALYSIS:] local generic I32(
    1247,
) @ LocalIndex(
    0,
) : Tee, location: Location { instr_index: 1, funct_index: 19 }
[ANALYSIS:] local generic I32(
    1247,
) @ LocalIndex(
    0,
) : Get, location: Location { instr_index: 1, funct_index: 20 }
[ANALYSIS:] binary generic I32Mul I32(
    1247,
) I32(
    1247,
), location: Location { instr_index: 1, funct_index: 21 }
[ANALYSIS:] apply (post) WasmFunction {
    uninstr_idx: 1,
    sig_pointer: 1179640,
}(RuntimeValues {
    argc: 1,
    resc: 1,
    sigv: 1179640,
    signature_types: [
        I32,
        I32,
    ],
    signature_offsets: [
        0,
        4,
    ],
}) = RuntimeValues {
    argc: 1,
    resc: 1,
    sigv: 1179640,
    signature_types: [
        I32,
        I32,
    ],
    signature_offsets: [
        0,
        4,
    ],
}
"# };

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

#[test]
fn test_analysis() {
    //////////////////////////
    // COMPILE & INSTRUMENT //
    //////////////////////////

    let analysis_compiler = Compiler::setup_compiler().expect("Setup Rust compiler");
    let instrumentation_compiler = Compiler::setup_compiler().expect("Setup Rust compiler");

    let input_program = compile_input_program(MANIFEST_SOURCE, RUST_SOURCE_HELLO_WORLD);

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

    let stdout = wasmtime_wasi::pipe::MemoryOutputPipe::new(usize::MAX);
    let stderr = wasmtime_wasi::pipe::MemoryOutputPipe::new(usize::MAX);

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

    // Get function
    let f_function = &linker
        .get(&mut store, "main", "f")
        .unwrap()
        .into_func()
        .unwrap()
        .typed::<i32, i32>(&store)
        .unwrap();

    // Invoke
    let result_from_123 = f_function.call(&mut store, 123).unwrap();
    assert_eq!(result_from_123, 1555009);

    assert_eq!(
        EXPECTED_ANALYSIS_STDOUT,
        String::from_utf8_lossy(&stdout.contents())
    );

    let result_from_123 = f_function.call(&mut store, 0).unwrap();
    assert_eq!(result_from_123, 12345);
}
