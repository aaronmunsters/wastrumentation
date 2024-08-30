use std::io::{Read, Write};
use std::path::absolute;

use clap::Parser;
use clio::*;
use serde::Deserialize;
use wastrumentation::{analysis, analysis::Hook as AnalysisHook, Wastrumenter};

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

fn main() -> std::io::Result<()> {
    let Args {
        mut input_program_path,
        rust_analysis_toml_path,
        mut output_path,
        hooks,
        targets,
    } = Args::parse();

    let mut wasm_module = Vec::new();
    input_program_path.read_to_end(&mut wasm_module)?;
    let hooks = hooks.iter().map(From::from).collect();

    let manifest = absolute(rust_analysis_toml_path.path().path())?;
    let analysis = analysis::Analysis::Rust { manifest, hooks };
    let instrumented_wasm_module = Wastrumenter::new()
        .wastrument(&wasm_module, &analysis, &targets)
        .expect("Instrumenting failed");

    output_path.write_all(&instrumented_wasm_module)?;

    Ok(())
}
