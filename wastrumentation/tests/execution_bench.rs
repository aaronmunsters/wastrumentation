use rayon::prelude::*;
use test_conf::{
    Analysis, AnalysisRust, InputProgram, InstrumentedAssertion, PostExecutionAssertion,
    UninstrumentedAssertion, WasmValue,
};
use wasmtime::*;
use wastrumentation::{
    compiler::{Compiles, DefaultCompilerOptions},
    Wastrumenter,
};
use wastrumentation_instr_lib::std_lib_compile::assemblyscript::compiler::WaspAnalysisSpec;
use wastrumentation_instr_lib::std_lib_compile::rust::RustAnalysisSpec;

use crate::test_conf::{CallYields, GlobalValueEquals, InputProgramAssertion, TestConfiguration};
use std::fs::{read, read_to_string};
use std::path::absolute;
use std::path::PathBuf;

mod test_conf;

const TEST_RELATIVE_PATH: &str = "./tests/";

use wastrumentation_instr_lib::std_lib_compile::assemblyscript::compiler::Compiler as ASCompiler;
use wastrumentation_instr_lib::std_lib_compile::assemblyscript::compiler_options::CompilerOptions as ASCompilerCompilerOptions;
use wastrumentation_instr_lib::std_lib_compile::rust::{Compiler as RSCompiler, RustSource};

#[test]
fn test_integration_configurations() {
    let test_configurations_json = include_str!("test-configurations.json");
    let test_configurations: Vec<TestConfiguration> =
        serde_json::from_str(test_configurations_json).unwrap();

    let as_compiler_ref = &ASCompiler::setup_compiler().unwrap();
    test_configurations
        .par_iter()
        .for_each(|test_configuration| test_configuration.assert_behavior(as_compiler_ref));
}

struct WatModule(pub Vec<u8>);

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

struct EngineSetup {
    store: Store<()>,
    engine: Engine,
}

impl EngineSetup {
    fn new() -> Self {
        let engine: Engine = Engine::new(Config::default().wasm_multi_memory(true)).unwrap();
        let store = Store::new(&engine, ());

        Self { store, engine }
    }
}

impl TestConfiguration {
    fn to_wat_module(&self, as_compiler: &ASCompiler) -> std::io::Result<WatModule> {
        let path = PathBuf::from(TEST_RELATIVE_PATH).join(&self.input_program.path);
        let content = read(&path)?;
        let wat_module = match &self.input_program.r#type {
            test_conf::ProgramType::Wat => self.compile_as_wat(&content),
            test_conf::ProgramType::AssemblyScript => {
                self.compile_as_assemblyscript(&content, as_compiler)
            }
        };
        Ok(wat_module)
    }

    fn as_wasmtime_values(values: &[WasmValue]) -> Vec<Val> {
        values.iter().map(Into::into).collect()
    }

    fn wasmtime_args(&self) -> Vec<Val> {
        Self::as_wasmtime_values(&self.uninstrumented_assertion.arguments)
    }

    fn wasmtime_expected_uninstrumented_results(&self) -> Vec<Val> {
        Self::as_wasmtime_values(&self.uninstrumented_assertion.results)
    }

    fn assert_behavior(&self, as_compiler: &ASCompiler) {
        let WatModule(input_program) = self.to_wat_module(as_compiler).unwrap();
        Self::assert_uninstrumented(self, &input_program);
        Self::assert_instrumented(self, &input_program);
    }

    fn get_entry_point<T>(&self, instance: Instance, store: &mut Store<T>) -> Func {
        let InputProgram { path, .. } = &self.input_program;
        let UninstrumentedAssertion {
            input_entry_point, ..
        } = &self.uninstrumented_assertion;
        instance
            .get_func(store, input_entry_point)
            .unwrap_or_else(|| {
                panic!("Cannot retrieve func {input_entry_point:#?} from input program {path:#?}")
            })
    }

    fn assert_uninstrumented(&self, input_program_wasm: &[u8]) {
        let EngineSetup { mut store, engine } = EngineSetup::new();
        let module = Module::from_binary(&engine, input_program_wasm).unwrap();
        let instance = Instance::new(&mut store, &module, &[]).unwrap();

        // Check uninstrumented
        let params = self.wasmtime_args();
        let mut actual_results = self.wasmtime_expected_uninstrumented_results();

        self.get_entry_point(instance, &mut store)
            .call(store, &params, &mut actual_results)
            .unwrap();
        WasmValue::assert_equals_wasmtime_values(
            &self.uninstrumented_assertion.results,
            &actual_results,
        );
    }

    fn assert_instrumented(&self, input_program_wasm: &[u8]) {
        let input_program_wasm = Vec::from(input_program_wasm);
        for instrumented_assertion @ InstrumentedAssertion { analysis, .. } in
            &self.instrumented_assertions
        {
            let instrumented_input = analysis.instrument_for(&input_program_wasm);

            let EngineSetup {
                mut store, engine, ..
            } = EngineSetup::new();
            let InstrumentedAssertion {
                post_execution_assertions,
                input_program_assertion,
                ..
            } = instrumented_assertion;
            // 4. execute instrumented input program
            let module = Module::from_binary(&engine, &instrumented_input).unwrap();
            let instance = Instance::new(&mut store, &module, &[]).unwrap();

            // Check instrumentation result
            let params = self.wasmtime_args();
            let expected_results = match input_program_assertion {
                InputProgramAssertion::DifferentReturnValue(results) => results.clone(),
                InputProgramAssertion::EqualToUninstrumented => {
                    self.uninstrumented_assertion.results.clone()
                }
            };

            let mut actual_results = Self::as_wasmtime_values(&expected_results.clone());

            // Call input program
            self.get_entry_point(instance, &mut store)
                .call(&mut store, &params, &mut actual_results)
                .unwrap();

            // 5. check if output of instrumented input program matches
            WasmValue::assert_equals_wasmtime_values(&expected_results, &actual_results);

            for instrumentation_configuration in post_execution_assertions {
                instrumentation_configuration.assert_outcome(&instance, &mut store);
            }
        }
    }

    fn compile_as_wat(&self, content: &[u8]) -> WatModule {
        let content: Vec<u8> = wat::parse_bytes(content)
            .expect("wat2wasm of input program failed")
            .into();
        WatModule(content)
    }

    fn compile_as_assemblyscript(&self, content: &[u8], as_compiler: &ASCompiler) -> WatModule {
        let source_code = String::from_utf8(content.to_vec()).unwrap();
        let compiler_options = ASCompilerCompilerOptions::default_for(source_code);
        let content = as_compiler.compile(&compiler_options).unwrap();

        WatModule(content)
    }
}

impl PostExecutionAssertion {
    fn assert_outcome<T>(&self, instance: &Instance, store: &mut Store<T>) {
        match self {
            Self::CallYields(call_yields) => call_yields.assert_outcome(instance, store),
            Self::GlobalValueEquals(global_value_equals) => {
                global_value_equals.assert_outcome(instance, store)
            }
        }
    }
}

impl CallYields {
    fn assert_outcome<T>(&self, instance: &Instance, store: &mut Store<T>) {
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
    fn assert_outcome<T>(&self, instance: &Instance, store: &mut Store<T>) {
        let Self { identifier, result } = self;
        let global = instance
            .get_global(&mut store.as_context_mut(), identifier)
            .unwrap()
            .get(store.as_context_mut());
        result.assert_equals_wasmtime(&global);
    }
}

impl Analysis {
    fn instrument_for(&self, input_program: &Vec<u8>) -> Vec<u8> {
        // Language for Instrumentation: Currently alwasy Rust
        // Language for Analysis: Depends on input analysis

        let rs_compiler = Box::new(RSCompiler::setup_compiler().unwrap());

        match self {
            Analysis::Wasp(analysis_path) => {
                let full_analysis_path = PathBuf::from(TEST_RELATIVE_PATH).join(analysis_path);
                let wasp_source = read_to_string(full_analysis_path)
                    .unwrap_or_else(|_| panic!("Could not open {analysis_path:?}"));
                let wasp_analysis_spec = WaspAnalysisSpec { wasp_source };

                let as_compiler = Box::new(ASCompiler::setup_compiler().unwrap());

                Wastrumenter::new(rs_compiler, as_compiler)
                    .wastrument(input_program, &wasp_analysis_spec, &None)
                    .unwrap()
            }
            Analysis::Rust(AnalysisRust {
                manifest: manifest_path,
                hooks,
            }) => {
                let analysis = RustAnalysisSpec {
                    source: RustSource::Manifest(absolute(manifest_path).unwrap()),
                    hooks: hooks.clone().into_iter().collect(),
                };

                let second_rs_compiler = Box::new(RSCompiler::setup_compiler().unwrap());

                Wastrumenter::new(rs_compiler, second_rs_compiler)
                    .wastrument(input_program, analysis, &None)
                    .unwrap()
            }
        }
    }
}
