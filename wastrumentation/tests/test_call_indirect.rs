use std::path::Path;

use wasmtime::{Engine, Extern, Func, FuncType, Instance, Module, Store, Val, ValType};
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
    let f0 = Extern::Func(Func::new(
        &mut store,
        FuncType::new(&engine, vec![], vec![ValType::I32]),
        |_, _, res| {
            res[0] = Val::I32(0);
            Ok({})
        },
    ));
    let f1 = Extern::Func(Func::new(
        &mut store,
        FuncType::new(&engine, vec![], vec![ValType::I32]),
        |_, _, res| {
            res[0] = Val::I32(1);
            Ok({})
        },
    ));
    let f2 = Extern::Func(Func::new(
        &mut store,
        FuncType::new(&engine, vec![], vec![ValType::I32]),
        |_, _, res| {
            res[0] = Val::I32(2);
            Ok({})
        },
    ));
    let f3 = Extern::Func(Func::new(
        &mut store,
        FuncType::new(&engine, vec![], vec![ValType::I32]),
        |_, _, res| {
            res[0] = Val::I32(3);
            Ok({})
        },
    ));
    let f4 = Extern::Func(Func::new(
        &mut store,
        FuncType::new(&engine, vec![], vec![ValType::I32]),
        |_, _, res| {
            res[0] = Val::I32(4);
            Ok({})
        },
    ));
    let instance = Instance::new(&mut store, &module, &[f0, f1, f2, f3, f4]).unwrap();

    let test_f = instance.get_func(&mut store, "test").unwrap();

    let check_t0 = instance
        .get_typed_func::<i32, i32>(&mut store, "check_t0")
        .unwrap();

    // let check_t1 = instance
    //     .get_typed_func::<i32, i32>(&mut store, "check_t1")
    //     .unwrap();

    test_f.call(&mut store, &[], &mut []).unwrap();
    // assert!((check_t0.call(&mut store, 0).is_err()));
    // assert!((check_t0.call(&mut store, 1).is_err()));
    // assert_eq!(check_t0.call(&mut store, 2).unwrap(), 3);
    // assert_eq!(check_t0.call(&mut store, 3).unwrap(), 1);
    // assert_eq!(check_t0.call(&mut store, 4).unwrap(), 4);
    // assert_eq!(check_t0.call(&mut store, 5).unwrap(), 1);
    // assert!((check_t0.call(&mut store, 6).is_err()));
    // assert!((check_t0.call(&mut store, 7).is_err()));
    // assert!((check_t0.call(&mut store, 8).is_err()));
    // assert!((check_t0.call(&mut store, 9).is_err()));
    // assert!((check_t0.call(&mut store, 10).is_err()));
    // assert!((check_t0.call(&mut store, 11).is_err()));
    assert_eq!(check_t0.call(&mut store, 12).unwrap(), 7);
    // assert_eq!(check_t0.call(&mut store, 13).unwrap(), 5);
    // assert_eq!(check_t0.call(&mut store, 14).unwrap(), 2);
    // assert_eq!(check_t0.call(&mut store, 15).unwrap(), 3);
    // assert_eq!(check_t0.call(&mut store, 16).unwrap(), 6);
    // assert!((check_t0.call(&mut store, 17).is_err()));
    // assert!((check_t0.call(&mut store, 18).is_err()));
    // assert!((check_t0.call(&mut store, 19).is_err()));
    // assert!((check_t0.call(&mut store, 20).is_err()));
    // assert!((check_t0.call(&mut store, 21).is_err()));
    // assert!((check_t0.call(&mut store, 22).is_err()));
    // assert!((check_t0.call(&mut store, 23).is_err()));
    // assert!((check_t0.call(&mut store, 24).is_err()));
    // assert!((check_t0.call(&mut store, 25).is_err()));
    // assert!((check_t0.call(&mut store, 26).is_err()));
    // assert!((check_t0.call(&mut store, 27).is_err()));
    // assert!((check_t0.call(&mut store, 28).is_err()));
    // assert!((check_t0.call(&mut store, 29).is_err()));
    // assert!((check_t1.call(&mut store, 0).is_err()));
    // assert!((check_t1.call(&mut store, 1).is_err()));
    // assert!((check_t1.call(&mut store, 2).is_err()));
    // assert_eq!(check_t1.call(&mut store, 3).unwrap(), 1);
    // assert_eq!(check_t1.call(&mut store, 4).unwrap(), 3);
    // assert_eq!(check_t1.call(&mut store, 5).unwrap(), 1);
    // assert_eq!(check_t1.call(&mut store, 6).unwrap(), 4);
    // assert!((check_t1.call(&mut store, 7).is_err()));
    // assert!((check_t1.call(&mut store, 8).is_err()));
    // assert!((check_t1.call(&mut store, 9).is_err()));
    // assert!((check_t1.call(&mut store, 10).is_err()));
    // assert!((check_t1.call(&mut store, 11).is_err()));
    // assert_eq!(check_t1.call(&mut store, 12).unwrap(), 3);
    // assert_eq!(check_t1.call(&mut store, 13).unwrap(), 1);
    // assert_eq!(check_t1.call(&mut store, 14).unwrap(), 4);
    // assert_eq!(check_t1.call(&mut store, 15).unwrap(), 1);
    // assert!((check_t1.call(&mut store, 16).is_err()));
    // assert!((check_t1.call(&mut store, 17).is_err()));
    // assert!((check_t1.call(&mut store, 18).is_err()));
    // assert!((check_t1.call(&mut store, 19).is_err()));
    // assert!((check_t1.call(&mut store, 20).is_err()));
    // assert!((check_t1.call(&mut store, 21).is_err()));
    // assert_eq!(check_t1.call(&mut store, 22).unwrap(), 7);
    // assert_eq!(check_t1.call(&mut store, 23).unwrap(), 5);
    // assert_eq!(check_t1.call(&mut store, 24).unwrap(), 2);
    // assert_eq!(check_t1.call(&mut store, 25).unwrap(), 3);
    // assert_eq!(check_t1.call(&mut store, 26).unwrap(), 6);
    // assert!((check_t1.call(&mut store, 27).is_err()));
    // assert!((check_t1.call(&mut store, 28).is_err()));
    // assert!((check_t1.call(&mut store, 29).is_err()));
}
