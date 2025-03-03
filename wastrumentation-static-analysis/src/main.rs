use std::io::{Read, Write};

use clap::Parser;
use clio::*;
use wasabi_wasm::Idx;
use wastrumentation_static_analysis::immutable_functions;

/// Command-line interface to the wastrumentation utility
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to input wasm module
    #[arg(short, long)]
    input_program_path: Input,

    /// Output path for the analysis results
    #[arg(short, long)]
    output_path: Output,

    /// Minimum size of body
    #[arg(short, long)]
    minimum_body: Option<u32>,
}

fn main() -> anyhow::Result<()> {
    let Args {
        mut input_program_path,
        mut output_path,
        minimum_body,
    } = Args::parse();

    let mut input_program = vec![];
    input_program_path.read_to_end(&mut input_program)?;

    let (module, _, _) = wasabi_wasm::Module::from_bytes(&input_program)?;
    let set = immutable_functions(&module);
    let mut results: Vec<u32> = set.iter().copied().collect();
    if let Some(minimum_body_count) = minimum_body {
        results.retain(|index| {
            module
                .function(Idx::from(*index))
                .code()
                .expect("Function has no body ... ?")
                .body
                .len()
                >= minimum_body_count
                    .try_into()
                    .expect("Cannot convert to usize")
        });
    }
    let _ = output_path.write(&serde_json::ser::to_vec(&results)?)?;
    Ok(())
}
