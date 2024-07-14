use rayon::prelude::*;
use std::path::{absolute, Path};

use rust_to_wasm_compiler::RustToWasmCompiler;
use wasmtime::{Config, Engine, Instance, Module, Store, Val};

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
            .compile(&absolute(Path::new(path)).unwrap())
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
