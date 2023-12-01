use anyhow::Result;
use wasmtime::*;
use wasmtime_wasi::sync::WasiCtxBuilder;

fn main() -> Result<()> {
    // First the wasm module needs to be compiled. This is done with a global
    // "compilation environment" within an `Engine`. Note that engines can be
    // further configured through `Config` if desired instead of using the
    // default like this is here.
    println!("Compiling module...");

    let engine: Engine = Engine::new(Config::default().wasm_multi_memory(true))?;
    let mut linker = Linker::new(&engine);
    linker.allow_unknown_exports(true);
    wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;

    // Create a WASI context and put it in a Store; all instances in the store
    // share this context. `WasiCtxBuilder` provides a number of ways to
    // configure what the target program will have access to.
    let wasi = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_args()?
        .build();
    let mut store = Store::new(&engine, wasi);

    // Instantiate our module with the imports we've created, and run it.
    let module = Module::from_file(&engine, "./merged.wasm")?;
    linker.module(&mut store, "main", &module)?;
    linker
        .get_default(&mut store, "main")?
        .typed::<(), ()>(&store)?
        .call(&mut store, ())?;

    let mut results = [Val::I32(0)];

    if let Some(Extern::Func(function)) = linker.get(&mut store, "main", "add-two") {
        function.call(&mut store, &[Val::I32(10), Val::I32(20)], &mut results)?
    }

    dbg!("Results are {}", results);
    Ok(())
}
