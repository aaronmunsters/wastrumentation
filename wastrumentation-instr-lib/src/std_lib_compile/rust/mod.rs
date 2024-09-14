use std::{collections::HashSet, path::PathBuf};

use crate::Rust;
use anyhow::Result;
use rust_to_wasm_compiler::{Profile, RustToWasmCompiler};
use serde::Deserialize;
use wastrumentation::{
    analysis::{AnalysisInterface, ProcessedAnalysis},
    compiler::{CompilationError, CompilationResult, Compiles, DefaultCompilerOptions},
};

pub struct Compiler {
    compiler: RustToWasmCompiler,
}

#[derive(Debug, Clone)]
pub struct ManifestSource(pub String);
#[derive(Debug, Clone)]
pub struct RustSourceCode(pub String);

#[derive(Debug, Clone)]
pub enum RustSource {
    SourceCode(ManifestSource, RustSourceCode),
    Manifest(PathBuf),
}

pub struct CompilerOptions {
    pub source: RustSource,
    pub profile: Profile,
}

impl DefaultCompilerOptions<Rust> for CompilerOptions {
    fn default_for(source: RustSource) -> Self {
        Self {
            profile: Profile::Release,
            source,
        }
    }
}

impl Compiles<Rust> for Compiler {
    type CompilerOptions = CompilerOptions;

    fn setup_compiler() -> anyhow::Result<Self> {
        Ok(Self {
            compiler: RustToWasmCompiler::new()?,
        })
    }

    fn compile(&self, compiler_options: &Self::CompilerOptions) -> CompilationResult<Rust> {
        match &compiler_options.source {
            RustSource::SourceCode(
                ManifestSource(manifest_source_code),
                RustSourceCode(rust_source_code),
            ) => self.compiler.compile_source(
                manifest_source_code,
                rust_source_code,
                compiler_options.profile,
            ),
            RustSource::Manifest(manifest_path) => self
                .compiler
                .compile(manifest_path, compiler_options.profile),
        }
        .map_err(|err| CompilationError::because(err.to_string()))
    }
}

#[derive(Clone)]
pub struct RustAnalysisSpec {
    pub source: RustSource,
    pub hooks: HashSet<Hook>,
}

impl TryInto<ProcessedAnalysis<Rust>> for RustAnalysisSpec {
    type Error = anyhow::Error;

    fn try_into(self) -> std::result::Result<ProcessedAnalysis<Rust>, Self::Error> {
        let RustAnalysisSpec { ref hooks, source } = self;

        let analysis_interface: AnalysisInterface = interface_from(hooks)?;

        Ok(ProcessedAnalysis {
            analysis_interface,
            analysis_library: source,
        })
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash, Deserialize)]
pub enum Hook {
    GenericApply,
    CallBefore,
    CallAfter,
    CallIndirectBefore,
    CallIndirectAfter,
}

pub fn interface_from(hooks: &HashSet<Hook>) -> Result<AnalysisInterface> {
    let mut interface = AnalysisInterface::default();
    for hook in hooks {
        match hook {
            Hook::GenericApply => {
                interface.generic_interface = Some(AnalysisInterface::interface_generic_apply())
            }
            Hook::CallBefore => {
                interface.pre_trap_call = Some(AnalysisInterface::interface_call_pre())
            }
            Hook::CallAfter => {
                interface.post_trap_call = Some(AnalysisInterface::interface_call_post())
            }
            Hook::CallIndirectBefore => {
                interface.pre_trap_call_indirect =
                    Some(AnalysisInterface::interface_call_indirect_pre())
            }
            Hook::CallIndirectAfter => {
                interface.post_trap_call_indirect =
                    Some(AnalysisInterface::interface_call_indirect_post())
            }
        }
    }
    Ok(interface)
}
