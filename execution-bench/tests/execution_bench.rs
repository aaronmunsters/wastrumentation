use wasi_common::pipe::WritePipe;
use wasmtime::*;
use wasmtime_wasi::sync::WasiCtxBuilder;

#[test]
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
    assert_eq!(
        stdout_content,
        r#"Hello from the start!
Passed argument: 10
Passed argument: 9
Passed argument: 8
Passed argument: 7
Passed argument: 6
Passed argument: 5
Passed argument: 4
Passed argument: 3
Passed argument: 2
Passed argument: 1
Passed argument: 1, result: 1
Passed argument: 2, result: 2
Passed argument: 3, result: 6
Passed argument: 4, result: 24
Passed argument: 5, result: 120
Passed argument: 6, result: 720
Passed argument: 7, result: 5040
Passed argument: 8, result: 40320
Passed argument: 9, result: 362880
Passed argument: 10, result: 3628800
"#
    );
}
