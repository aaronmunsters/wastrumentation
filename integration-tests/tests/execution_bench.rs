use test_conf::{InstrumentationResult, WasmValue};
use wasi_common::pipe::WritePipe;
use wasmer::wat2wasm;
use wasmtime::*;
use wasmtime_wasi::sync::WasiCtxBuilder;
use wastrumentation_instr_lib::std_lib_compile::{
    assemblyscript::compiler_options::{
        CompilerOptions as AssemblScriptCompilerOptions, OptimizationStrategy, RuntimeStrategy,
    },
    CompilerOptions,
};

use crate::test_conf::{CallYields, GlobalValueEquals, TestConfiguration};
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

struct WatModule(pub Vec<u8>);

// TODO: change to TryInto
impl Into<WatModule> for &TestConfiguration {
    fn into(self) -> WatModule {
        let mut path = PathBuf::from_str(TEST_RELATIVE_PATH).unwrap();
        path.push(&self.input_program);

        let content = read(&path).expect(&format!("Could not open {}", path.display()));
        match &self.input_program_type {
            test_conf::InputProgramType::Wat => self.into_wat(&content),
            test_conf::InputProgramType::AssemblyScript => self.into_assemblyscript(&content),
        }
    }
}

#[derive(Default)]
struct AbortStore {
    abort_count: i32,
}

impl Into<Val> for &WasmValue {
    fn into(self) -> Val {
        match self {
            &WasmValue::I32(v) => Val::I32(v),
            &WasmValue::F32(v) => Val::F32(v),
            &WasmValue::I64(v) => Val::I64(v),
            &WasmValue::F64(v) => Val::F64(v),
        }
    }
}

impl WasmValue {
    fn assert_equals_wasmtime(&self, wasmtime_value: &Val) {
        match self {
            WasmValue::I32(v) => assert_eq!(wasmtime_value.unwrap_i32(), *v),
            WasmValue::F32(v) => assert_eq!(wasmtime_value.unwrap_f32(), *v as f32),
            WasmValue::I64(v) => assert_eq!(wasmtime_value.unwrap_i64(), *v),
            WasmValue::F64(v) => assert_eq!(wasmtime_value.unwrap_f64(), *v as f64),
        }
    }

    fn assert_equals_wasmtime_values(expected: &[WasmValue], actual: &[Val]) {
        for (expected, actual) in expected.iter().zip(actual) {
            expected.assert_equals_wasmtime(actual);
        }
    }
}

impl TestConfiguration {
    fn as_wasmtime_values(values: &[WasmValue]) -> Vec<Val> {
        values.iter().map(Into::into).collect()
    }

    fn wasmtime_args(&self) -> Vec<Val> {
        Self::as_wasmtime_values(&self.arguments)
    }

    fn wasmtime_expected_results(&self) -> Vec<Val> {
        Self::as_wasmtime_values(&self.results)
    }

    fn assert_behavior(&self) {
        let WatModule(input_program) = self.into();
        Self::assert_uninstrumented(self, &input_program);
        Self::assert_instrumented(self, &input_program);
    }

    fn assert_uninstrumented(&self, input_program_wasm: &[u8]) {
        let engine = Engine::new(&Config::default()).unwrap();
        let mut store = Store::new(&engine, ());
        let module = Module::from_binary(&engine, &input_program_wasm).unwrap();
        let instance = Instance::new(&mut store, &module, &[]).unwrap();

        let func = instance
            .get_func(&mut store, &self.input_entry_point)
            .expect(&format!(
                "Cannot retrieve func {} from input program {}",
                &self.input_entry_point,
                &self.input_program.display(),
            ));

        let params = self.wasmtime_args();
        let mut actual_results = self.wasmtime_expected_results();

        func.call(store, &params, &mut actual_results).unwrap();
        WasmValue::assert_equals_wasmtime_values(&self.results, &actual_results);
    }

    fn assert_instrumented(&self, input_program_wasm: &[u8]) {
        let input_program_wasm = Vec::from(input_program_wasm);
        for instrumentation_conf in &self.instrumentation_configurations {
            let mut analysis_path = PathBuf::from_str(TEST_RELATIVE_PATH).unwrap();
            analysis_path.push(&instrumentation_conf.analysis);

            let input_analysis =
                read_to_string(&analysis_path).expect(&format!("Could not open {analysis_path:?}"));

            let instrumented_input =
                wastrumentation::wastrument(&input_program_wasm, &input_analysis)
                    .expect("Instrumentation pass failed");

            // 4. execute instrumented input program
            let mut store = Store::<AbortStore>::default();
            let module = Module::from_binary(store.engine(), &instrumented_input).unwrap();

            let env_abort = Func::wrap(
                &mut store,
                |mut caller: Caller<'_, AbortStore>, _: i32, _: i32, _: i32, _: i32| {
                    caller.data_mut().abort_count += 1;
                },
            );

            let instance = Instance::new(&mut store, &module, &[env_abort.into()]).unwrap();

            assert_eq!(store.data().abort_count, 0);
            env_abort
                .call(
                    &mut store,
                    &[Val::I32(0), Val::I32(0), Val::I32(0), Val::I32(0)],
                    &mut [],
                )
                .unwrap();
            assert_eq!(store.data().abort_count, 1);

            // Check instrumentation result
            let func = instance
                .get_func(&mut store, &self.input_entry_point)
                .expect(&format!(
                    "Cannot retrieve func {} from input program {}",
                    &self.input_entry_point,
                    self.input_program.display(),
                ));

            let params = self.wasmtime_args();
            let mut actual_results = self.wasmtime_expected_results();

            func.call(&mut store, &params, &mut actual_results).unwrap();

            // 5. check if output of instrumented input program matches
            WasmValue::assert_equals_wasmtime_values(&self.results, &actual_results);

            assert_eq!(store.data().abort_count, 1);

            for instrumentation_configuration in &instrumentation_conf.instrumentation_results {
                instrumentation_configuration.assert_outcome(&instance, &mut store);
            }
        }
    }

    fn into_wat(&self, content: &[u8]) -> WatModule {
        let content: Vec<u8> = wat2wasm(&content)
            .expect("wat2wasm of input program failed")
            .into();
        WatModule(content)
    }

    fn into_assemblyscript(&self, content: &[u8]) -> WatModule {
        let source_code = String::from_utf8(content.to_vec()).unwrap();
        let compiler_options = AssemblScriptCompilerOptions {
            source_code,
            optimization_strategy: OptimizationStrategy::O3,
            enable_bulk_memory: false,
            enable_sign_extension: false,
            enable_nontrapping_f2i: false,
            enable_export_memory: self.wasi_enabled,
            enable_wasi_shim: self.wasi_enabled,
            runtime: RuntimeStrategy::Minimal,
        };
        let content = compiler_options.compile().module().unwrap();
        WatModule(content)
    }
}

impl InstrumentationResult {
    fn assert_outcome(&self, instance: &Instance, store: &mut Store<AbortStore>) {
        match self {
            Self::CallYields(call_yields) => call_yields.assert_outcome(instance, store),
            Self::GlobalValueEquals(global_value_equals) => {
                global_value_equals.assert_outcome(instance, store)
            }
        }
    }
}

impl CallYields {
    fn assert_outcome(&self, instance: &Instance, store: &mut Store<AbortStore>) {
        let Self {
            call,
            arguments,
            result,
        } = self;
        let call = instance.get_func(store.as_context_mut(), call).unwrap();
        let params = TestConfiguration::as_wasmtime_values(&arguments);
        let mut actual_results = TestConfiguration::as_wasmtime_values(&result);

        // Perform call
        call.call(store.as_context_mut(), &params, &mut actual_results)
            .unwrap();

        WasmValue::assert_equals_wasmtime_values(&self.result, &actual_results);
    }
}

impl GlobalValueEquals {
    fn assert_outcome(&self, instance: &Instance, store: &mut Store<AbortStore>) {
        let Self { identifier, result } = self;
        let global = instance
            .get_global(&mut store.as_context_mut(), &identifier)
            .unwrap()
            .get(store.as_context_mut());
        result.assert_equals_wasmtime(&global);
    }
}

// TODO: implement using Wasi, writing to buffers etc.
#[allow(dead_code)]
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
