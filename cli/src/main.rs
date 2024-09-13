use std::io::{Read, Write};

use clap::Parser;
use clio::*;
use serde::Deserialize;
use wastrumentation::analysis::RustAnalysisSpec;
use wastrumentation::{analysis::Hook as AnalysisHook, Wastrumenter};

use wastrumentation_instr_lib::std_lib_compile::Compiles;

use wastrumentation_instr_lib::std_lib_compile::assemblyscript::compiler::Compiler as AssemblyScriptCompiler;
use wastrumentation_instr_lib::std_lib_compile::rust::{Compiler as RustCompiler, RustSource};

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
    CallBefore,
    CallAfter,
    CallIndirectBefore,
    CallIndirectAfter,
}

impl From<&Hook> for AnalysisHook {
    fn from(hook: &Hook) -> Self {
        match hook {
            Hook::GenericApply => AnalysisHook::GenericApply,
            Hook::CallBefore => AnalysisHook::CallBefore,
            Hook::CallAfter => AnalysisHook::CallAfter,
            Hook::CallIndirectBefore => AnalysisHook::CallIndirectBefore,
            Hook::CallIndirectAfter => AnalysisHook::CallIndirectAfter,
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

    let instrumented_wasm_module = Wastrumenter::new(
        Box::new(instrumentation_language_compiler),
        Box::new(analysis_language_compiler),
    )
    .wastrument(&wasm_module, analysis, &targets)
    .expect("Instrumenting failed");

    output_path.write_all(&instrumented_wasm_module)?;

    Ok(())
}
