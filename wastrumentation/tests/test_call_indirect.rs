use std::path::Path;

use wasmtime::{Engine, Instance, Module, Store};
use wastrumentation::wastrument;

#[test]
fn test_call_indirect() {
    let wasm_binary = wat::parse_file(Path::new(
        "./tests/input-programs/wat/call_indirect_wast_example.wat",
    ))
    .unwrap();

    let wasp_source = include_str!("./analysis/call_indirect.wasp");
    let wasm_binary = wastrument(&wasm_binary, wasp_source).unwrap();

    let engine = Engine::default();
    let module = Module::from_binary(&engine, &wasm_binary).unwrap();
    module
        .imports()
        .for_each(|i| println!("Import: {} -> {}", i.module(), i.name()));
    let mut store = Store::new(&engine, ());
    let instance = Instance::new(&mut store, &module, &[]).unwrap();

    let test_f = instance.get_func(&mut store, "copy-t0-to-t1").unwrap();

    let check_t0 = instance
        .get_typed_func::<i32, i32>(&mut store, "check_t0")
        .unwrap();

    test_f.call(&mut store, &[], &mut []).unwrap();
    assert_eq!(check_t0.call(&mut store, 0).unwrap(), 0);
}
