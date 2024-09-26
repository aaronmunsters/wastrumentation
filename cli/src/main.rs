use std::io::{Read, Write};

use clap::Parser;
use clio::*;
use serde::Deserialize;
use wastrumentation::compiler::Compiles;
use wastrumentation::{Configuration, Wastrumenter};
use wastrumentation_instr_lib::lib_gen::analysis::rust::{Hook as AnalysisHook, RustAnalysisSpec};

use assemblyscript_compiler::compiler::Compiler as AssemblyScriptCompiler;
use wastrumentation_instr_lib::lib_compile::rust::compiler::Compiler as RustCompiler;
use wastrumentation_instr_lib::lib_compile::rust::options::RustSource;

/// Command-line interface to the wastrumentation utility
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to input wasm module
    #[arg(short, long)]
    input_program_path: Input,

    /// Path to rust analysis TOML file
    #[arg(short, long)]
    rust_analysis_toml_path: Input,

    /// Hooks to instrument
    #[arg(long, required = true, num_args = 1..)]
    hooks: Vec<Hook>,

    // Target functions of interest
    #[arg(long, required = false, num_args = 1..)]
    targets: Option<Vec<u32>>,

    /// Output path for the instrumented module
    #[arg(short, long)]
    output_path: Output,
}

#[derive(clap::ValueEnum, Debug, Clone, Deserialize, PartialEq, Eq, Copy, Hash)]
enum Hook {
    GenericApply,
    CallPre,
    CallPost,
    CallIndirectPre,
    CallIndirectPost,
}

impl From<&Hook> for AnalysisHook {
    fn from(hook: &Hook) -> Self {
        match hook {
            Hook::GenericApply => AnalysisHook::GenericApply,
            Hook::CallPre => AnalysisHook::CallPre,
            Hook::CallPost => AnalysisHook::CallPost,
            Hook::CallIndirectPre => AnalysisHook::CallIndirectPre,
            Hook::CallIndirectPost => AnalysisHook::CallIndirectPost,
        }
    }
}

fn main() -> anyhow::Result<()> {
    let Args {
        mut input_program_path,
        rust_analysis_toml_path,
        mut output_path,
        hooks,
        targets,
    } = Args::parse();

    let mut wasm_module = Vec::new();
    input_program_path.read_to_end(&mut wasm_module)?;

    let analysis = RustAnalysisSpec {
        hooks: hooks.iter().map(From::from).collect(),
        source: RustSource::Manifest(rust_analysis_toml_path.path().to_path_buf()),
    };

    let instrumentation_language_compiler = AssemblyScriptCompiler::setup_compiler()?;
    let analysis_language_compiler = RustCompiler::setup_compiler()?;
    let configuration = Configuration {
        target_indices: targets,
        primary_selection: None,
    };

    let instrumented_wasm_module = Wastrumenter::new(
        Box::new(instrumentation_language_compiler),
        Box::new(analysis_language_compiler),
    )
    .wastrument(&wasm_module, analysis, &configuration)
    .expect("Instrumenting failed");

    output_path.write_all(&instrumented_wasm_module)?;

    Ok(())
}
