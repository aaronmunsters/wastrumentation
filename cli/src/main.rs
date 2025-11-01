use std::io::{Read, Write};

use clap::Parser;
use clio::*;
use rust_to_wasm_compiler::WasiSupport;
use serde::Deserialize;
use wastrumentation::compiler::Compiles;
use wastrumentation::{Configuration, Wastrumenter};
use wastrumentation_lang_rust::compile::compiler::Compiler as RustCompiler;
use wastrumentation_lang_rust::compile::options::RustSource;
use wastrumentation_lang_rust::generate::analysis::{Hook as AnalysisHook, RustAnalysisSpec};

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
    #[arg(long, num_args = 1..)]
    hooks: Option<Vec<Hook>>,

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
    IfThen,
    IfThenPost,
    IfThenElse,
    IfThenElsePost,
    Branch,
    BranchIf,
    BranchTable,
    Select,
    Unary,
    Binary,
    Drop,
    Return,
    Const,
    Local,
    Global,
    Store,
    Load,
    MemorySize,
    MemoryGrow,
    BlockPre,
    BlockPost,
    LoopPre,
    LoopPost,
    RefFunc,
    RefNull,
    RefIsNull,
    TableGet,
    TableSet,
    TableSize,
    TableGrow,
    TableFill,
    TableCopy,
    TableInit,
    ElemDrop,
}

impl From<&Hook> for AnalysisHook {
    fn from(hook: &Hook) -> Self {
        match hook {
            Hook::GenericApply => AnalysisHook::GenericApply,
            Hook::CallPre => AnalysisHook::CallPre,
            Hook::CallPost => AnalysisHook::CallPost,
            Hook::CallIndirectPre => AnalysisHook::CallIndirectPre,
            Hook::CallIndirectPost => AnalysisHook::CallIndirectPost,
            Hook::IfThen => AnalysisHook::IfThen,
            Hook::IfThenPost => AnalysisHook::IfThenPost,
            Hook::IfThenElse => AnalysisHook::IfThenElse,
            Hook::IfThenElsePost => AnalysisHook::IfThenElsePost,
            Hook::Branch => AnalysisHook::Branch,
            Hook::BranchIf => AnalysisHook::BranchIf,
            Hook::BranchTable => AnalysisHook::BranchTable,
            Hook::Select => AnalysisHook::Select,
            Hook::Unary => AnalysisHook::Unary,
            Hook::Binary => AnalysisHook::Binary,
            Hook::Drop => AnalysisHook::Drop,
            Hook::Return => AnalysisHook::Return,
            Hook::Const => AnalysisHook::Const,
            Hook::Local => AnalysisHook::Local,
            Hook::Global => AnalysisHook::Global,
            Hook::Store => AnalysisHook::Store,
            Hook::Load => AnalysisHook::Load,
            Hook::MemorySize => AnalysisHook::MemorySize,
            Hook::MemoryGrow => AnalysisHook::MemoryGrow,
            Hook::BlockPre => AnalysisHook::BlockPre,
            Hook::BlockPost => AnalysisHook::BlockPost,
            Hook::LoopPre => AnalysisHook::LoopPre,
            Hook::LoopPost => AnalysisHook::LoopPost,
            Hook::RefFunc => AnalysisHook::RefFunc,
            Hook::RefNull => AnalysisHook::RefNull,
            Hook::RefIsNull => AnalysisHook::RefIsNull,
            Hook::TableGet => AnalysisHook::TableGet,
            Hook::TableSet => AnalysisHook::TableSet,
            Hook::TableSize => AnalysisHook::TableSize,
            Hook::TableGrow => AnalysisHook::TableGrow,
            Hook::TableFill => AnalysisHook::TableFill,
            Hook::TableCopy => AnalysisHook::TableCopy,
            Hook::TableInit => AnalysisHook::TableInit,
            Hook::ElemDrop => AnalysisHook::ElemDrop,
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

    let hooks = match hooks {
        None => AnalysisHook::all_hooks(),
        Some(hooks) => hooks.iter().map(From::from).collect(),
    };

    let analysis = RustAnalysisSpec {
        hooks,
        source: RustSource::Manifest(
            WasiSupport::Disabled,
            rust_analysis_toml_path.path().to_path_buf(),
        ),
    }
    .into();

    let instrumentation_language_compiler = RustCompiler::setup_compiler()?;
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
