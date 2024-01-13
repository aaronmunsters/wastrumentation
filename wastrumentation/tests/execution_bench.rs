use rayon::prelude::*;
use test_conf::{AssemblyScript, PostExecutionAssertion, WasmValue};
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

const TEST_RELATIVE_PATH: &str = "./tests/";

#[test]
fn test_integration_configurations() {
    let test_configurations_json = include_str!("test-configurations.json");
    let test_configurations: Vec<TestConfiguration> =
        serde_json::from_str(test_configurations_json).unwrap();
    test_configurations
        .par_iter()
        .for_each(|test_configuration| test_configuration.assert_behavior());
}

struct WatModule(pub Vec<u8>);

// TODO: change to TryInto
impl From<&TestConfiguration> for WatModule {
    fn from(test_configuration: &TestConfiguration) -> Self {
        let mut path = PathBuf::from_str(TEST_RELATIVE_PATH).unwrap();
        path.push(&test_configuration.input_program.path);

        let content = read(&path).unwrap_or_else(|_| panic!("Could not open {}", path.display()));
        match &test_configuration.input_program.r#type {
            test_conf::ProgramType::Wat => test_configuration.compile_as_wat(&content),
            test_conf::ProgramType::AssemblyScript(AssemblyScript { wasi_enabled }) => {
                test_configuration.compile_as_assemblyscript(&content, *wasi_enabled)
            }
        }
    }
}

#[derive(Default)]
struct AbortStore {
    abort_count: i32,
}

impl From<&WasmValue> for Val {
    fn from(val: &WasmValue) -> Self {
        match *val {
            WasmValue::I32(v) => Val::I32(v),
            WasmValue::F32(v) => Val::F32(v),
            WasmValue::I64(v) => Val::I64(v),
            WasmValue::F64(v) => Val::F64(v),
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
        Self::as_wasmtime_values(&self.uninstrumented_assertion.arguments)
    }

    fn wasmtime_expected_uninstrumented_results(&self) -> Vec<Val> {
        Self::as_wasmtime_values(&self.uninstrumented_assertion.results)
    }

    fn assert_behavior(&self) {
        let WatModule(input_program) = self.into();
        let mut engine = Engine::new(Config::default().wasm_multi_memory(true)).unwrap();
        Self::assert_uninstrumented(self, &input_program, &mut engine);
        Self::assert_instrumented(self, &input_program, &mut engine);
    }

    fn assert_uninstrumented(&self, input_program_wasm: &[u8], engine: &mut Engine) {
        let mut store = Store::new(engine, ());
        let module = Module::from_binary(engine, input_program_wasm).unwrap();
        let instance = Instance::new(&mut store, &module, &[]).unwrap();

        let func: Func = instance
            .get_func(&mut store, &self.uninstrumented_assertion.input_entry_point)
            .unwrap_or_else(|| {
                panic!(
                    "Cannot retrieve func {} from input program {}",
                    &self.uninstrumented_assertion.input_entry_point,
                    &self.input_program.path.display()
                )
            });

        let params = self.wasmtime_args();
        let mut actual_results = self.wasmtime_expected_uninstrumented_results();

        func.call(store, &params, &mut actual_results).unwrap();
        WasmValue::assert_equals_wasmtime_values(
            &self.uninstrumented_assertion.results,
            &actual_results,
        );
    }

    fn assert_instrumented(&self, input_program_wasm: &[u8], engine: &mut Engine) {
        let input_program_wasm = Vec::from(input_program_wasm);
        for instrumented_assertion in &self.instrumented_assertions {
            let mut analysis_path = PathBuf::from_str(TEST_RELATIVE_PATH).unwrap();
            analysis_path.push(&instrumented_assertion.analysis);

            let input_analysis = read_to_string(&analysis_path)
                .unwrap_or_else(|_| panic!("Could not open {analysis_path:?}"));

            let instrumented_input =
                wastrumentation::wastrument(&input_program_wasm, &input_analysis)
                    .expect("Instrumentation pass failed");

            // 4. execute instrumented input program
            let mut store = Store::<AbortStore>::new(engine, AbortStore::default());
            let module = Module::from_binary(engine, &instrumented_input).unwrap();

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
                .get_func(&mut store, &self.uninstrumented_assertion.input_entry_point)
                .unwrap_or_else(|| {
                    panic!(
                        "Cannot retrieve func {} from input program {}",
                        &self.uninstrumented_assertion.input_entry_point,
                        self.input_program.path.display()
                    )
                });

            let params = self.wasmtime_args();
            let expected_results = match &instrumented_assertion.uninstrumented_assertion {
                test_conf::UninstrumentedInstrumentedAssertion::EqualToUninstrumentedAssertion => {
                    self.uninstrumented_assertion.results.clone()
                }
                test_conf::UninstrumentedInstrumentedAssertion::DifferentReturnValue(results) => {
                    results.clone()
                }
            };

            let mut actual_results = Self::as_wasmtime_values(&expected_results.clone());

            func.call(&mut store, &params, &mut actual_results).unwrap();

            // 5. check if output of instrumented input program matches
            WasmValue::assert_equals_wasmtime_values(&expected_results, &actual_results);

            assert_eq!(store.data().abort_count, 1);

            for instrumentation_configuration in &instrumented_assertion.post_execution_assertions {
                instrumentation_configuration.assert_outcome(&instance, &mut store);
            }
        }
    }

    fn compile_as_wat(&self, content: &[u8]) -> WatModule {
        let content: Vec<u8> = wat2wasm(content)
            .expect("wat2wasm of input program failed")
            .into();
        WatModule(content)
    }

    fn compile_as_assemblyscript(&self, content: &[u8], wasi_enabled: bool) -> WatModule {
        let source_code = String::from_utf8(content.to_vec()).unwrap();
        let compiler_options = AssemblScriptCompilerOptions {
            source_code,
            optimization_strategy: OptimizationStrategy::O3,
            enable_bulk_memory: false,
            enable_sign_extension: false,
            enable_nontrapping_f2i: false,
            enable_export_memory: wasi_enabled,
            enable_wasi_shim: wasi_enabled,
            runtime: RuntimeStrategy::Minimal,
        };
        let content = compiler_options.compile().module().unwrap();
        WatModule(content)
    }
}

impl PostExecutionAssertion {
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
            results,
        } = self;
        let call = instance.get_func(store.as_context_mut(), call).unwrap();
        let params = TestConfiguration::as_wasmtime_values(arguments);
        let mut actual_results = TestConfiguration::as_wasmtime_values(results);

        // Perform call
        call.call(store.as_context_mut(), &params, &mut actual_results)
            .unwrap();

        WasmValue::assert_equals_wasmtime_values(&self.results, &actual_results);
    }
}

impl GlobalValueEquals {
    fn assert_outcome(&self, instance: &Instance, store: &mut Store<AbortStore>) {
        let Self { identifier, result } = self;
        let global = instance
            .get_global(&mut store.as_context_mut(), identifier)
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
        results.first().unwrap().i32().unwrap(),
        10 * 9 * 8 * 7 * 6 * 5 * 4 * 3 * 2
    );

    let stdout_stream = stdout.try_into_inner().unwrap().into_inner();
    let stdout_content = String::from_utf8(stdout_stream).unwrap();
    assert_eq!(stdout_content, r#""#);
}
