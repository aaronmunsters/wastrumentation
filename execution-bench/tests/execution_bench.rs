use test_conf::WasmValue;
use wabt::wat2wasm;
use wasi_common::pipe::WritePipe;
use wasmtime::*;
use wasmtime_wasi::sync::WasiCtxBuilder;

use crate::test_conf::TestConfiguration;
use std::{
    fs::{read, read_to_string},
    path::PathBuf,
    str::FromStr,
};

mod test_conf;

const TEST_RELATIVE_PATH: &'static str = "./tests/";

#[test]
fn test_integration_configurations() {
    let test_configurations_json = include_str!("test-configurations.json");
    let test_configurations: Vec<TestConfiguration> =
        serde_json::from_str(test_configurations_json).unwrap();
    for test_configuration in test_configurations {
        test_configuration.assert_behavior();
    }
}

enum WasmProgram {
    Text(String),
    Binary(Vec<u8>),
}

struct WasmValues(Vec<WasmValue>);

impl Into<Vec<Val>> for WasmValues {
    fn into(self) -> Vec<Val> {
        let WasmValues(values) = self;
        values
            .iter()
            .map(|v| match v {
                WasmValue::I32(v) => Val::I32(*v),
                WasmValue::F32(v) => Val::F32(*v),
                WasmValue::I64(v) => Val::I64(*v),
                WasmValue::F64(v) => Val::F64(*v),
            })
            .collect()
    }
}

impl TestConfiguration {
    fn assert_behavior(&self) {
        let mut input_program_path = PathBuf::from_str(TEST_RELATIVE_PATH).unwrap();
        input_program_path.push(&self.input_program);

        let input_program: WasmProgram = if input_program_path.extension().unwrap().eq("wat") {
            WasmProgram::Text(
                read_to_string(&input_program_path)
                    .expect(format!("Could not open {}", input_program_path.display()).as_str()),
            )
        } else {
            WasmProgram::Binary(
                read(&input_program_path)
                    .expect(format!("Could not open {}", input_program_path.display()).as_str()),
            )
        };

        let mut analysis_path = PathBuf::from_str(TEST_RELATIVE_PATH).unwrap();
        analysis_path.push(&self.analysis);
        let input_analysis = read_to_string(&analysis_path)
            .expect(format!("Could not open {}", analysis_path.display()).as_str());

        // 1. execute input program
        let input_program_wasm = match input_program {
            WasmProgram::Text(input_program) => {
                wat2wasm(input_program).expect("wat2wasm of input program")
            }
            WasmProgram::Binary(input_program) => input_program,
        };

        // 2. check if input matches
        let engine = Engine::new(&Config::default()).unwrap();
        let mut store = Store::new(&engine, ());
        let module = Module::from_binary(&engine, &input_program_wasm).unwrap();
        let instance = Instance::new(&mut store, &module, &[]).unwrap();

        let func = instance
            .get_func(&mut store, &self.input_entry_point)
            .expect(
                format!(
                    "Cannot retrieve func {} from input program {}",
                    &self.input_entry_point,
                    input_program_path.display()
                )
                .as_str(),
            );

        let params: Vec<Val> = WasmValues(self.arguments.clone()).into();
        let expected_results: Vec<Val> = WasmValues(self.results.clone()).into();
        let mut results: Vec<Val> = WasmValues(self.results.clone()).into();

        func.call(store, &params, &mut results).unwrap();

        for (expected, actual) in expected_results.iter().zip(&results) {
            match (expected, actual) {
                (Val::I32(e), Val::I32(a)) => assert_eq!(e, a),
                (Val::I64(e), Val::I64(a)) => assert_eq!(e, a),
                (Val::F32(e), Val::F32(a)) => assert_eq!(e, a),
                (Val::F64(e), Val::F64(a)) => assert_eq!(e, a),
                (Val::V128(e), Val::V128(a)) => assert_eq!(e, a),
                _ => panic!(),
            };
        }

        // 3. instrument input program
        let _ = input_analysis; // TODO:

        // 4. execute instrumented input program
        // 5. check if input of instrumented input program matches
        // 6. check if input of instrumentation matches
    }
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
