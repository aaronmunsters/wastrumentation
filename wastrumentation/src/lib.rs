pub mod analysis;
mod instrument;
pub mod parse_nesting;
mod stack_library;

use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
};

use crate::instrument::InstrumentationResult;
use instrument::function_application::{
    INSTRUMENTATION_ANALYSIS_MODULE, INSTRUMENTATION_INSTRUMENTED_MODULE,
    INSTRUMENTATION_STACK_MODULE,
};
use wasm_merge::{InputModule, MergeError, MergeOptions};
use wastrumentation_instr_lib::{
    std_lib_compile::{Compiles, DefaultCompilerOptions, WasmModule},
    LibGeneratable, SourceCodeBound,
};

use anyhow::{anyhow, Result};

use analysis::ProcessedAnalysis;

pub use stack_library::Signature;

pub struct Wastrumenter<
    InstrumentationLanguage,
    InstrumentationLanguageCompiler,
    AnalysisLanguage,
    AnalysisLanguageCompiler,
> where
    InstrumentationLanguage: LibGeneratable + SourceCodeBound,
    InstrumentationLanguageCompiler: Compiles<InstrumentationLanguage>,
    AnalysisLanguage: SourceCodeBound,
    AnalysisLanguageCompiler: Compiles<AnalysisLanguage>,
{
    instrumentation_language_compiler: Box<InstrumentationLanguageCompiler>,
    instrumentation_language: PhantomData<InstrumentationLanguage>,
    analysis_language_compiler: Box<AnalysisLanguageCompiler>,
    analysis_language: PhantomData<AnalysisLanguage>,
}

impl<
        InstrumentationLanguage,
        InstrumentationLanguageCompiler,
        AnalysisLanguage,
        AnalysisLanguageCompiler,
    >
    Wastrumenter<
        InstrumentationLanguage,
        InstrumentationLanguageCompiler,
        AnalysisLanguage,
        AnalysisLanguageCompiler,
    >
where
    InstrumentationLanguage: LibGeneratable + SourceCodeBound,
    InstrumentationLanguageCompiler: Compiles<InstrumentationLanguage>,
    AnalysisLanguageCompiler: Compiles<AnalysisLanguage>,
    AnalysisLanguage: SourceCodeBound,
{
    pub fn new(
        instrumentation_language_compiler: Box<InstrumentationLanguageCompiler>,
        analysis_language_compiler: Box<AnalysisLanguageCompiler>,
    ) -> Self {
        Self {
            instrumentation_language_compiler,
            analysis_language_compiler,
            instrumentation_language: PhantomData,
            analysis_language: PhantomData,
        }
    }

    /// # Errors
    /// Errors upon failing to compile, instrument or merge.
    pub fn wastrument<Error>(
        &self,
        input_program: &WasmModule,
        analysis: impl TryInto<ProcessedAnalysis<AnalysisLanguage>, Error = Error>,
        target_indices: &Option<Vec<u32>>,
    ) -> Result<WasmModule>
    where
        Error: Display,
        Error: Debug,
    {
        // 1. Compile analysis
        let ProcessedAnalysis {
            analysis_library,
            analysis_interface,
        } = analysis.try_into().map_err(|err| anyhow!("{err:?}"))?;
        let analysis_compiler_options =
            AnalysisLanguageCompiler::CompilerOptions::default_for(analysis_library);
        let analysis_wasm = self
            .analysis_language_compiler
            .compile(&analysis_compiler_options)
            .map_err(|err| anyhow!(err.reason().to_string()))?;
        // 2. Instrument the input program
        let InstrumentationResult {
            module: instrumented_input,
            instrumentation_library,
        } = instrument::instrument::<InstrumentationLanguage>(
            input_program,
            &analysis_interface,
            target_indices,
        );
        // 3. Compile the instrumentation lib
        let compiled_instrumentation_lib = if let Some(library) = instrumentation_library {
            let instrumentation_compiler_options =
                InstrumentationLanguageCompiler::CompilerOptions::default_for(library.content);
            Some(
                self.instrumentation_language_compiler
                    .compile(&instrumentation_compiler_options)
                    .map_err(|err| anyhow!(err.reason().to_string()))?,
            )
        } else {
            None
        };

        // 4. Merge them all together
        let instrumented_input = Self::merge(
            instrumented_input,
            analysis_wasm,
            compiled_instrumentation_lib,
        )?;

        // 5. Yield expected result
        Ok(instrumented_input)
    }

    fn merge(
        instrumented_input: WasmModule,
        compiled_analysis: WasmModule,
        compiled_instrumentation_lib: Option<WasmModule>,
    ) -> Result<WasmModule> {
        let mut input_modules =
            if let Some(compiled_instrumentation_lib) = compiled_instrumentation_lib {
                vec![InputModule {
                    module: compiled_instrumentation_lib,
                    namespace: INSTRUMENTATION_STACK_MODULE.into(),
                }]
            } else {
                vec![]
            };
        input_modules.append(&mut vec![
            InputModule {
                module: compiled_analysis,
                namespace: INSTRUMENTATION_ANALYSIS_MODULE.into(),
            },
            InputModule {
                module: instrumented_input,
                namespace: INSTRUMENTATION_INSTRUMENTED_MODULE.into(),
            },
        ]);
        let merge_options = MergeOptions {
            no_validate: true,
            rename_export_conflicts: true,
            enable_multi_memory: true,
            input_modules,
        };
        wasm_merge::merge(&merge_options)
            .map_err(|MergeError(reason)| anyhow!("MergeError: {}", reason))
    }
}

#[cfg(test)]
mod tests {
    use analysis::WaspAnalysisSpec;
    use wastrumentation_instr_lib::std_lib_compile::assemblyscript::compiler::Compiler as AssemblyscriptCompiler;
    use wastrumentation_instr_lib::std_lib_compile::assemblyscript::compiler_options::CompilerOptions as AssemblyscriptCompilerOptions;
    use wastrumentation_instr_lib::std_lib_compile::rust::Compiler as RustCompiler;

    use super::*;
    use wasmtime::*;

    const SOURCE_CODE_WASP: &str = r#"
    (aspect
        (global >>>GUEST>>>
            export let call_count: i32 = 0;
            export let call_depth_max: i32 = 0;
            let call_stack: i32 = 0;

            function max<T>(a: T, b: T): T {
                return a > b ? a : b;
            }
        <<<GUEST<<<)

        (advice apply (func    WasmFunction)
                      (args    MutDynArgs)
                      (results MutDynResults) >>>GUEST>>>
            // Before call:
            // [1] Increment call stack size
            // [2] Ensure highest call stack size is recorded
            // [3] Ensure call count is incremented
            // After call:
            // [4] Ensure call count is decremented


            /* [1] */
            call_stack += 1;
            /* [2] */
            call_depth_max = max(call_depth_max, call_stack);
            /* [3] */
            call_count += 1;
            func.apply();
            /* [4] */
            call_stack -= 1;

        <<<GUEST<<<))"#;

    const SOURCE_CODE_INPUT: &str = r#"
    export function fib(n: i32): i32 {
        return n <= 2 ? 1 : fib(n - 1) + fib(n - 2);
    }"#;

    #[test]
    fn example_instrumentation_rust() {
        let as_compiler = AssemblyscriptCompiler::setup_compiler().unwrap();
        let as_compiler_options =
            AssemblyscriptCompilerOptions::default_for(SOURCE_CODE_INPUT.into());

        let input_program = as_compiler.compile(&as_compiler_options).unwrap();

        let wasp_analysis_spec: WaspAnalysisSpec = WaspAnalysisSpec {
            wasp_source: SOURCE_CODE_WASP.into(),
        };

        let rust_compiler = RustCompiler::setup_compiler().unwrap();
        let instrumented_input = Wastrumenter::new(Box::new(rust_compiler), Box::new(as_compiler))
            .wastrument(&input_program, &wasp_analysis_spec, &None)
            .unwrap();

        // Execute & check instrumentation
        let mut store = Store::<()>::default();
        let module = Module::from_binary(store.engine(), &instrumented_input).unwrap();
        let instance = Instance::new(&mut store, &module, &[]).unwrap();

        // Fetch `fib` export
        let fib = instance
            .get_typed_func::<i32, i32>(&mut store, "fib")
            .unwrap();

        let call_to_fib_result = fib.call(&mut store, 10).unwrap();

        assert_eq!(call_to_fib_result, 55);

        let expected_globals = [("call_count", 109), ("call_depth_max", 9)];
        for (global_name, expected_value) in expected_globals {
            let global = instance.get_global(&mut store, global_name).unwrap();
            let actual_value = global.get(&mut store).i32().unwrap();
            assert_eq!(expected_value, actual_value);
        }
    }

    #[test]
    fn example_instrumentation_wasp() {
        let assemblyscript_compiler_options =
            AssemblyscriptCompilerOptions::default_for(SOURCE_CODE_INPUT.to_string());
        let assemblyscript_compiler1 = AssemblyscriptCompiler::setup_compiler().unwrap();
        let assemblyscript_compiler2 = AssemblyscriptCompiler::setup_compiler().unwrap();

        // TODO: better to create a second one? Perhaps use Rc? Or maybe Arc?

        let input_program = assemblyscript_compiler1
            .compile(&assemblyscript_compiler_options)
            .unwrap();

        let wasp_analysis_spec: WaspAnalysisSpec = WaspAnalysisSpec {
            wasp_source: SOURCE_CODE_WASP.into(),
        };

        let instrumented_input = Wastrumenter::new(
            Box::new(assemblyscript_compiler1),
            Box::new(assemblyscript_compiler2),
        )
        .wastrument(&input_program, &wasp_analysis_spec, &None)
        .unwrap();

        // Execute & check instrumentation
        let mut store = Store::<()>::default();
        let module = Module::from_binary(store.engine(), &instrumented_input).unwrap();
        let instance = Instance::new(&mut store, &module, &[]).unwrap();

        // Fetch `fib` export
        let fib = instance
            .get_typed_func::<i32, i32>(&mut store, "fib")
            .unwrap();

        let call_to_fib_result = fib.call(&mut store, 10).unwrap();

        assert_eq!(call_to_fib_result, 55);

        let expected_globals = [("call_count", 109), ("call_depth_max", 9)];
        for (global_name, expected_value) in expected_globals {
            let global = instance.get_global(&mut store, global_name).unwrap();
            let actual_value = global.get(&mut store).i32().unwrap();
            assert_eq!(expected_value, actual_value);
        }
    }
}
