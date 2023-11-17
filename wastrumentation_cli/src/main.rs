mod cli;

use anyhow::Error;
use clap::Parser;
use cli::Cli;

use wasabi_wasm::Module;

pub const INSTRUMENTATION_STACK_MODULE: &str = "wastrumentation_stack";
pub const INSTRUMENTATION_ANALYSIS_MODULE: &str = "analysis";

fn main() -> Result<(), Error> {
    let cli = Cli::parse();
    let (mut module, _offsets, _warnings) = Module::from_file(cli.input)?;
    wastrumentation::instrument(&mut module);
    module.to_file(cli.output)?;
    Ok(())
}
