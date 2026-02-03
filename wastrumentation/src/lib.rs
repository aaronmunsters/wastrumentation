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

    // New merge
    fn merge(
        primary_selection: &Option<PrimaryTarget>,
        instrumented_input: &[u8],
        compiled_analysis: &[u8],
        compiled_instrumentation_lib: Option<&[u8]>,
    ) -> Result<WasmModule, Error<AnalysisLanguage, InstrumentationLanguage>> {
        use wasm_mergers::merge_options::MergeOptions;
        use wasm_mergers::*;

        let input_analysis = move || {
            Some(NamedModule::new(
                INSTRUMENTATION_ANALYSIS_MODULE,
                compiled_analysis,
            ))
        };
        let input_target = move || {
            Some(NamedModule::new(
                INSTRUMENTATION_INSTRUMENTED_MODULE,
                instrumented_input,
            ))
        };
        let input_instrumentation = move || {
            compiled_instrumentation_lib
                .map(|lib| NamedModule::new(INSTRUMENTATION_STACK_MODULE, lib))
        };

        let input_modules = match primary_selection {
            Some(PrimaryTarget::Analysis) => {
                vec![input_analysis(), input_target(), input_instrumentation()]
            }
            Some(PrimaryTarget::Target) => {
                vec![input_target(), input_analysis(), input_instrumentation()]
            }
            Some(PrimaryTarget::Instrumentation) => {
                vec![input_instrumentation(), input_target(), input_analysis()]
            }
            None => vec![input_target(), input_instrumentation(), input_analysis()],
        };

        let modules = input_modules.iter().flatten().collect::<Vec<_>>();

        use merge_options::{default_rename, RenameStrategy};
        let merge_options = MergeOptions {
            clashing_exports: merge_options::ClashingExports::Rename(RenameStrategy {
                first_occurrence: false,
                functions: default_rename,
                tables: default_rename,
                memories: default_rename,
                globals: default_rename,
                tags: default_rename,
            }),
            ..Default::default()
        };
        Ok(MergeConfiguration::new(&modules[..], merge_options)
            .merge()
            .unwrap())
    }
}
