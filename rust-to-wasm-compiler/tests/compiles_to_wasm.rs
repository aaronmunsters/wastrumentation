use rayon::prelude::*;
use std::path::{absolute, Path};

use rust_to_wasm_compiler::{Profile, RustToWasmCompiler, WasiSupport};
use wasmtime::{Config, Engine, Instance, Module, Store, Val};

use indoc::indoc;

// TODO: merge the engine code that the tests below share

#[test]
fn compiles_example() {
    let engine = Engine::new(&Config::default()).unwrap();

    vec![
        "./tests/recursion-rs/Cargo.toml",
        "./tests/recursion-rs-minimal/Cargo.toml",
    ]
    .par_iter()
    .for_each(|path| {
        // Compile
        let wasm_module = RustToWasmCompiler::new()
            .unwrap()
            .compile(
                WasiSupport::Disabled,
                &absolute(Path::new(path)).unwrap(),
                Profile::Dev,
            )
            .unwrap();

        // Run compiled result
        let mut store = Store::new(&engine, ());
        let module = Module::new(&engine, wasm_module).unwrap();
        let instance = Instance::new(&mut store, &module, &[]).unwrap();
        let params = &mut [Val::I32(0)];
        let results = &mut [Val::I32(0)];

        params[0] = Val::I32(5);
        instance
            .get_func(&mut store, "factorial")
            .unwrap()
            .call(&mut store, params, results)
            .unwrap();

        assert_eq!(results[0].i32().unwrap(), 120);

        params[0] = Val::I32(10);
        instance
            .get_func(&mut store, "fibonacci")
            .unwrap()
            .call(&mut store, params, results)
            .unwrap();

        assert_eq!(results[0].i32().unwrap(), 89);
    })
}

#[test]
fn compiles_using_sources() {
    let manifest_source = indoc! { r#"
        [package]
        name = "rust-wasp-call-stack"
        version = "0.1.0"
        edition = "2021"
        [lib]
        crate-type = ["cdylib"]
        [workspace]
    "# };
    let rust_source = indoc! { r#"
        #[no_mangle]
        pub extern "C" fn return_one_two_three_four() -> i32 { 1234 }
    "# };

    let engine = Engine::new(&Config::default()).unwrap();

    // Compile
    let wasm_module = RustToWasmCompiler::new()
        .unwrap()
        .compile_source(
            WasiSupport::Disabled,
            manifest_source,
            rust_source,
            Profile::Dev,
        )
        .unwrap();

    // Run compiled result
    let mut store = Store::new(&engine, ());
    let module = Module::new(&engine, wasm_module).unwrap();
    let instance = Instance::new(&mut store, &module, &[]).unwrap();
    let results = &mut [Val::I32(0)];
    instance
        .get_func(&mut store, "return_one_two_three_four")
        .unwrap()
        .call(&mut store, &[], results)
        .unwrap();

    assert_eq!(results[0].i32().unwrap(), 1234);
}
