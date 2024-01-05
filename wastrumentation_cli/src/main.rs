mod cli;

use std::fs::File;
use std::io::Write;

use anyhow::Error;
use clap::Parser;
use cli::Cli;

pub const INSTRUMENTATION_STACK_MODULE: &str = "wastrumentation_stack";
pub const INSTRUMENTATION_ANALYSIS_MODULE: &str = "analysis";

// TODO: implement tests

fn main() -> Result<(), Error> {
    let Cli {
        wasp,
        input,
        output,
    } = Cli::parse();
    // 1. Load WASP
    let wasp_source = std::fs::read_to_string(&wasp)
        .expect(&format!("Failed to read {}", wasp.to_string_lossy()));

    // 2. Load input program
    let input_program =
        std::fs::read(&input).expect(&format!("Failed to read {}", input.to_string_lossy()));

    // 3. Instrument the module
    let instrumented_input =
        wastrumentation::wastrument(&input_program, &wasp_source).expect("Failed to instrument");

    // 4. Save the output
    File::create(output)
        .unwrap()
        .write_all(&instrumented_input)
        .unwrap();

    Ok(())
}
