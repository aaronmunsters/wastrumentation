use std::io::{Read, Write};
use std::path::absolute;

use clap::Parser;
use clio::*;
use serde::Deserialize;
use wastrumentation::{analysis, Wastrumenter};

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

    /// Output path for the instrumented module
    #[arg(short, long)]
    output_path: Output,
}

#[derive(clap::ValueEnum, Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Hook {
    CallBefore,
    CallAfter,
    CallIndirectBefore,
    CallIndirectAfter,
    IfThen,
}

// TODO: move away from strings, use enum & serde everywhere
impl Hook {
    fn to_interface_string(&self) -> String {
        match self {
            Hook::CallBefore => "advice-call-before".into(),
            Hook::CallAfter => "advice-call-after".into(),
            Hook::CallIndirectBefore => "advice-call-indirect-before".into(),
            Hook::CallIndirectAfter => "advice-call-indirect-after".into(),
            Hook::IfThen => "advice-if-then".into(),
        }
    }
}

fn main() -> std::io::Result<()> {
    let Args {
        mut input_program_path,
        rust_analysis_toml_path,
        mut output_path,
        hooks,
    } = Args::parse();

    let mut wasm_module = Vec::new();
    input_program_path.read_to_end(&mut wasm_module)?;
    let hooks: Vec<String> = hooks.iter().map(Hook::to_interface_string).collect();

    let manifest = absolute(rust_analysis_toml_path.path().path())?;
    let analysis = analysis::Analysis::Rust { manifest, hooks };
    let instrumented_wasm_module = Wastrumenter::new()
        .wastrument(&wasm_module, &analysis)
        .expect("Instrumenting failed");

    output_path.write_all(&instrumented_wasm_module)?;

    Ok(())
}
