pub mod analysis;
pub mod compiler;
mod instrument;
pub mod parse_nesting;
mod stack_library;
pub mod wasm_constructs;

use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
};

use crate::instrument::InstrumentationResult;
use compiler::{Compiles, DefaultCompilerOptions, LibGeneratable, SourceCodeBound, WasmModule};
use instrument::function_application::{
    INSTRUMENTATION_ANALYSIS_MODULE, INSTRUMENTATION_INSTRUMENTED_MODULE,
    INSTRUMENTATION_STACK_MODULE,
};
use wasm_merge::{InputModule, MergeError, MergeOptions};

use anyhow::{anyhow, Result};

use analysis::ProcessedAnalysis;

pub use stack_library::ModuleLinkedStackHooks;

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

#[derive(Debug, Clone, Default)]
pub struct Configuration {
    pub target_indices: Option<Vec<u32>>,
    pub primary_selection: Option<PrimaryTarget>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimaryTarget {
    Instrumentation,
    Target,
    Analysis,
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
        configuration: &Configuration,
    ) -> Result<WasmModule>
    where
        Error: Display,
        Error: Debug,
    {
        let Configuration {
            target_indices,
            primary_selection,
        } = configuration;
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
            primary_selection,
            instrumented_input,
            analysis_wasm,
            compiled_instrumentation_lib,
        )?;

        // 5. Yield expected result
        Ok(instrumented_input)
    }

    fn merge(
        primary_selection: &Option<PrimaryTarget>,
        instrumented_input: WasmModule,
        compiled_analysis: WasmModule,
        compiled_instrumentation_lib: Option<WasmModule>,
    ) -> Result<WasmModule> {
        let input_analysis = move || {
            Some(InputModule {
                module: compiled_analysis,
                namespace: INSTRUMENTATION_ANALYSIS_MODULE.into(),
            })
        };
        let input_target = move || {
            Some(InputModule {
                module: instrumented_input,
                namespace: INSTRUMENTATION_INSTRUMENTED_MODULE.into(),
            })
        };
        let input_instrumentation = move || {
            compiled_instrumentation_lib.map(|lib| InputModule {
                module: lib,
                namespace: INSTRUMENTATION_STACK_MODULE.into(),
            })
        };

        let (primary, input_modules) = match primary_selection {
            Some(PrimaryTarget::Analysis) => (
                input_analysis(),
                vec![input_target(), input_instrumentation()],
            ),
            Some(PrimaryTarget::Target) => (
                input_target(),
                vec![input_analysis(), input_instrumentation()],
            ),
            Some(PrimaryTarget::Instrumentation) => (
                input_instrumentation(),
                vec![input_target(), input_analysis()],
            ),
            None => (
                None,
                vec![input_instrumentation(), input_target(), input_analysis()],
            ),
        };

        let input_modules = input_modules.into_iter().flatten().collect();

        let merge_options = MergeOptions {
            no_validate: true,
            rename_export_conflicts: true,
            enable_multi_memory: true,
            primary,
            input_modules,
        };
        wasm_merge::merge(&merge_options)
            .map_err(|MergeError(reason)| anyhow!("MergeError: {}", reason))
    }
}
