pub mod analysis;
mod instrument;
pub mod parse_nesting;
mod stack_library;

use crate::instrument::InstrumentationResult;
use analysis::assemblyscript::AssemblyScriptProgram;
use instrument::function_application::{
    INSTRUMENTATION_ANALYSIS_MODULE, INSTRUMENTATION_INSTRUMENTED_MODULE,
    INSTRUMENTATION_STACK_MODULE,
};
use wasm_merge::{InputModule, MergeError, MergeOptions};
use wastrumentation_instr_lib::std_lib_compile::{
    assemblyscript::compiler_options::CompilerOptions as AssemblyScriptCompilerOptions, WasmModule,
};

use anyhow::{anyhow, Result};

use wastrumentation_instr_lib::std_lib_compile::assemblyscript::compiler::Compiler as AssemblyScriptCompiler;

pub use analysis::Analysis;
use analysis::AnalysisCompilationResult;

pub struct Wastrumenter {
    assemblyscript_compiler: AssemblyScriptCompiler,
}

impl Default for Wastrumenter {
    fn default() -> Self {
        Self::new()
    }
}

impl Wastrumenter {
    pub fn new() -> Self {
        let assemblyscript_compiler = AssemblyScriptCompiler::new();
        Self {
            assemblyscript_compiler,
        }
    }

    pub fn assemblyscript_compiler(&self) -> &AssemblyScriptCompiler {
        &self.assemblyscript_compiler
    }

    /// # Errors
    /// Errors upon failing to compile, instrument or merge.
    pub fn wastrument(
        &self,
        input_program: &WasmModule,
        analysis: &Analysis,
    ) -> Result<WasmModule> {
        // 1. Compile wasp_source
        let AnalysisCompilationResult {
            analysis_wasm,
            analysis_interface,
        } = analysis.compile(self)?;
        // 2. Instrument the input program
        let InstrumentationResult {
            module: instrumented_input,
            instrumentation_lib,
        } = instrument::instrument(input_program, &analysis_interface);
        // 3. Compile the analysis & instrumentation lib
        let compiled_instrumentation_lib = self.compile(instrumentation_lib)?;

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
        compiled_instrumentation_lib: WasmModule,
    ) -> Result<WasmModule> {
        let merge_options = MergeOptions {
            no_validate: true,
            rename_export_conflicts: true,
            enable_multi_memory: true,
            input_modules: vec![
                InputModule {
                    module: compiled_instrumentation_lib,
                    namespace: INSTRUMENTATION_STACK_MODULE.into(),
                },
                InputModule {
                    module: compiled_analysis,
                    namespace: INSTRUMENTATION_ANALYSIS_MODULE.into(),
                },
                InputModule {
                    module: instrumented_input,
                    namespace: INSTRUMENTATION_INSTRUMENTED_MODULE.into(),
                },
            ],
        };
        wasm_merge::merge(&merge_options)
            .map_err(|MergeError(reason)| anyhow!("MergeError: {}", reason))
    }

    fn compile(&self, assemblyscript_program: AssemblyScriptProgram) -> Result<WasmModule> {
        let compiler = &self.assemblyscript_compiler;
        let AssemblyScriptProgram { content } = assemblyscript_program;
        let compiler_options = AssemblyScriptCompilerOptions::new(content);
        compiler
            .compile(&compiler_options)
            .map_err(|e| anyhow!(e.reason().to_string()))
    }
}

#[cfg(test)]
mod tests {
    use wastrumentation_instr_lib::std_lib_compile::assemblyscript::compiler_options::CompilerOptions as AssemblyscriptCompilerOptions;

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
    fn example_instrumentation() {
        let compiler_options = AssemblyscriptCompilerOptions::new(SOURCE_CODE_INPUT.into());
        let input_program = AssemblyScriptCompiler::new()
            .compile(&compiler_options)
            .unwrap();

        let wastrumenter = Wastrumenter::new();

        // Instrument the application
        let instrumented_input: Vec<u8> = wastrumenter
            .wastrument(
                &input_program,
                &Analysis::AssemblyScript {
                    wasp_source: SOURCE_CODE_WASP.into(),
                },
            )
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
