use anyhow::Result;
use wasmtime::*;

fn main() -> Result<()> {
    // First the wasm module needs to be compiled. This is done with a global
    // "compilation environment" within an `Engine`. Note that engines can be
    // further configured through `Config` if desired instead of using the
    // default like this is here.
    println!("Compiling module...");

    let engine: Engine = Engine::new(Config::default().wasm_multi_memory(true))?;
    let module = Module::from_file(&engine, "./merged.wasm")?;

    // After a module is compiled we create a `Store` which will contain
    // instantiated modules and other items like host functions. A Store
    // contains an arbitrary piece of host information, and we use `MyState`
    // here.
    println!("Initializing...");
    let mut store = Store::new(&engine, ());

    // Our wasm module we'll be instantiating requires one imported function.
    // the function takes no parameters and returns no results. We create a host
    // implementation of that function here, and the `caller` parameter here is
    // used to get access to our original `MyState` value.
    println!("Creating abort...");
    let abort_type = wasmtime::FuncType::new(
        [
            wasmtime::ValType::I32,
            wasmtime::ValType::I32,
            wasmtime::ValType::I32,
            wasmtime::ValType::I32,
        ],
        [],
    );
    let abort_function = Func::new(&mut store, abort_type, |_, _params, _results| {
        panic!("Wasm aborted!");
    });

    // Once we've got that all set up we can then move to the instantiation
    // phase, pairing together a compiled module as well as a set of imports.
    // Note that this is where the wasm `start` function, if any, would run.
    println!("Instantiating module...");
    let instance = Instance::new(&mut store, &module, &[abort_function.into()])?;

    // Next we poke around a bit to extract the `run` function from the module.
    println!("Extracting export...");
    let run = instance.get_typed_func::<(i32, i32), i32>(&mut store, "add-two")?;

    // And last but not least we can call it!
    println!("Calling export...");
    let res = run.call(&mut store, (10, 20))?;

    println!("Got back a {}", res);
    Ok(())
}
