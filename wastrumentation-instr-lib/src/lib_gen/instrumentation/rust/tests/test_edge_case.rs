use super::*;
use wasmtime::{Engine, Instance, Module, Store};
use WasmType::{I32, I64};

#[test]
fn test_edge() {
    ////////////////////
    //// GENERATION ////
    ////////////////////

    // Input program signatures
    let signatures = &[Signature {
        return_types: vec![I32, I64],
        argument_types: vec![],
    }];
    let (ManifestSource(manifest_source), RustSourceCode(rust_source_code)) =
        generate_lib(signatures);

    println!("{rust_source_code}");

    /////////////////////
    //// COMPILATION ////
    /////////////////////
    let compiler = rust_to_wasm_compiler::RustToWasmCompiler::new().unwrap();
    let instrumentation_wasm_library = compiler
        .compile_source(
            WasiSupport::Disabled,
            &manifest_source,
            &rust_source_code,
            Profile::Dev,
        )
        .unwrap();

    ///////////////////
    //// EXECUTION ////
    ///////////////////
    let engine = Engine::default();
    let mut store = Store::new(&engine, ());
    let module = Module::from_binary(&engine, &instrumentation_wasm_library).unwrap();

    for export in module.exports() {
        println!("{}", export.name());
    }

    let instance = Instance::new(&mut store, &module, &[]).unwrap();
    let _ = instance;

    // TODO:
    // simulate library apply pre
    // simulate library apply post
    // FIXME:
    //
}
