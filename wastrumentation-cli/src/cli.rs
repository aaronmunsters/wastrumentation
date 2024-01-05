use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "wastrumentation",
    about = "A Wasm instrumentation platform",
    long_about = None
)]
pub struct Cli {
    /// The input wasp program, in Wasm
    #[arg(short = 'w', long)]
    pub wasp: PathBuf,
    /// The input program to transform, in Wasm
    #[arg(short = 'i', long)]
    pub input: PathBuf,
    /// The output program path to write to, in Wasm
    #[arg(short = 'o', long)]
    pub output: PathBuf,
}
