mod cli;

use std::fs::File;
use std::io::Write;

use anyhow::Error;
use clap::Parser;
use cli::Cli;

use wasabi_wasm::Module;
use wasp_compiler::{compile, CompilationResult as WaspCompilationResult};

pub const INSTRUMENTATION_STACK_MODULE: &str = "wastrumentation_stack";
pub const INSTRUMENTATION_ANALYSIS_MODULE: &str = "analysis";

fn main() -> Result<(), Error> {
    let Cli {
        wasp,
        input,
        output,
    } = Cli::parse();
    // 1. Compile WASP
    let wasp = std::fs::read_to_string(&wasp)
        .expect(&format!("Failed to read {}", wasp.to_string_lossy()));
    let WaspCompilationResult {
        assemblyscript_program,
        join_points,
        wasp_interface,
    } = compile(&wasp).expect(&format!("Failed to compile {wasp}"));

    let mut output_analysis =
        File::create("./wastrumentation_instr_lib/src_generated/analysis.ts")?;
    write!(output_analysis, "{}", assemblyscript_program.content)?;

    // 2. Load module
    let (mut module, _offsets, _warnings) =
        Module::from_file(&input).expect(&format!("Failed to read {}", input.to_string_lossy()));

    // 3. Instrument the module
    wastrumentation::instrument(&mut module, join_points, wasp_interface);

    // 4. Save the output
    module
        .to_file(&output)
        .expect(&format!("Failed to write to {}", output.to_string_lossy()));
    Ok(())
}
