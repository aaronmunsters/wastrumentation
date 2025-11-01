pub mod analysis;
pub mod compiler;
pub mod error;
mod instrument;
pub mod parse_nesting;
mod stack_library;
pub mod wasm_constructs;

use std::fmt::Debug;
use std::marker::PhantomData;

use crate::instrument::Instrumented;
use analysis::ProcessedAnalysis;
use compiler::{Compiles, DefaultCompilerOptions, LibGeneratable, SourceCodeBound, WasmModule};
use instrument::function_application::INSTRUMENTATION_ANALYSIS_MODULE;
use instrument::function_application::INSTRUMENTATION_INSTRUMENTED_MODULE;
use instrument::function_application::INSTRUMENTATION_STACK_MODULE;
pub use stack_library::ModuleLinkedStackHooks;
use wasm_merge::options::BulkMemoryOpt;
use wasm_merge::options::{
    BulkMemory, Multimemory, NoValidate, ReferenceTypes, RenameExportConflicts,
};
use wasm_merge::{InputModule, MergeOptions};

use crate::error::Error;

#[derive(Clone)]
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
    pub fn wastrument(
        &self,
        input_program: &[u8],
        analysis: ProcessedAnalysis<AnalysisLanguage>,
        configuration: &Configuration,
    ) -> Result<WasmModule, Error<AnalysisLanguage, InstrumentationLanguage>> {
        let Configuration {
            target_indices,
            primary_selection,
        } = configuration;
        // 1. Compile analysis
        let ProcessedAnalysis {
            analysis_library,
            analysis_interface,
        } = analysis;
        let analysis_compiler_options =
            AnalysisLanguageCompiler::CompilerOptions::default_for(analysis_library);
        let analysis_wasm = self
            .analysis_language_compiler
            .compile(&analysis_compiler_options)
            .map_err(Error::CompilationErrorAnalysis)?;
        // 2. Instrument the input program
        let Instrumented {
            module: instrumented_input,
            instrumentation_library,
        } = instrument::instrument::<InstrumentationLanguage>(
            input_program,
            &analysis_interface,
            target_indices,
        )
        .map_err(Error::InstrumentationError)?;
        // 3. Compile the instrumentation lib
        let compiled_instrumentation_lib = if let Some(library) = instrumentation_library {
            let instrumentation_compiler_options =
                InstrumentationLanguageCompiler::CompilerOptions::default_for(library.content);
            Some(
                self.instrumentation_language_compiler
                    .compile(&instrumentation_compiler_options)
                    .map_err(Error::CompilationErrorInstrumentation)?,
            )
        } else {
            None
        };

        // 4. Merge them all together
        let instrumented_input = Self::merge(
            primary_selection,
            &instrumented_input,
            &analysis_wasm,
            compiled_instrumentation_lib.as_deref(),
        )?;

        // 5. Yield expected result
        Ok(instrumented_input)
    }

    fn merge(
        primary_selection: &Option<PrimaryTarget>,
        instrumented_input: &[u8],
        compiled_analysis: &[u8],
        compiled_instrumentation_lib: Option<&[u8]>,
    ) -> Result<WasmModule, Error<AnalysisLanguage, InstrumentationLanguage>> {
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
                vec![input_target(), input_instrumentation(), input_analysis()],
            ),
        };

        let input_modules = input_modules.into_iter().flatten().collect();

        let merge_options = MergeOptions {
            primary,
            input_modules,
            no_validation: NoValidate::Enable,
            rename_export_conflicts: RenameExportConflicts::Enable,
            multimemory: Multimemory::Enable,
            multivalue: wasm_merge::options::Multivalue::Enable,
            bulk_memory: BulkMemory::Enable,
            bulk_memory_opt: BulkMemoryOpt::Enable,
            reference_types: ReferenceTypes::Enable,
            nontrapping_float_to_int: wasm_merge::options::NontrappingFloatToInt::Enable,
            ..Default::default()
        };
        merge_options.merge().map_err(Error::MergeError)
    }
}
