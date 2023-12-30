use wabt::wat2wasm;
use wasi_common::pipe::WritePipe;
use wasmtime::*;
use wasmtime_wasi::sync::WasiCtxBuilder;

#[test]
fn test_function_call() {
    let source_target_wat_fib = r#"
    (module
        (export "fib" (func $fib))
        (func $fib (param $0 i32) (result i32)
            local.get $0
            i32.const 1
            i32.le_s
            if (result i32)
                i32.const 1
            else
                local.get $0
                i32.const 1
                i32.sub
                call $fib
                local.get $0
                i32.const 2
                i32.sub
                call $fib
                i32.add
            end
        )
    )"#;
    let source_target_wasm_fib = wat2wasm(source_target_wat_fib).unwrap();

    let engine = Engine::new(&Config::default()).unwrap();
    let mut store = Store::new(&engine, ());
    let module = Module::from_binary(&engine, &source_target_wasm_fib).unwrap();
    let instance = Instance::new(&mut store, &module, &[]).unwrap();
    let func = instance
        .get_typed_func::<(i32,), (i32,)>(&mut store, "fib")
        .unwrap();

    assert_eq!(func.call(store, (10,)).unwrap(), (89,));
}

// #[test]
fn test() {
    let engine: Engine = Engine::new(Config::default().wasm_multi_memory(true)).unwrap();
    let mut linker = Linker::new(&engine);
    linker.allow_unknown_exports(true);
    wasmtime_wasi::add_to_linker(&mut linker, |s| s).unwrap();

    // Generate STDOUT for checkout output later on
    let stdout = WritePipe::new_in_memory();

    let wasi = WasiCtxBuilder::new()
        .stdout(Box::new(stdout.clone()))
        .build();
    let mut store = Store::new(&engine, wasi);

    // Instantiate our module with the imports we've created, and run it.
    let module = Module::from_file(&engine, "../merged.wasm").unwrap();

    linker.module(&mut store, "main", &module).unwrap();
    linker
        .get_default(&mut store, "main")
        .unwrap()
        .typed::<(), ()>(&store)
        .unwrap()
        .call(&mut store, ())
        .unwrap();

    let mut results = [Val::I32(i32::default())];

    if let Some(Extern::Func(function)) = linker.get(&mut store, "main", "add-two") {
        function
            .call(&mut store, &[Val::I32(10), Val::I32(20)], &mut results)
            .unwrap()
    };

    // ensuring store is dropped will flush the stdout buffer
    drop(store);

    assert_eq!(
        results.get(0).unwrap().i32().unwrap(),
        10 * 9 * 8 * 7 * 6 * 5 * 4 * 3 * 2 * 1
    );

    let stdout_stream = stdout.try_into_inner().unwrap().into_inner();
    let stdout_content = String::from_utf8(stdout_stream).unwrap();
    assert_eq!(stdout_content, r#""#);
}
