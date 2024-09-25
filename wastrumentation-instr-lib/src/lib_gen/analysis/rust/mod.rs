use std::collections::HashSet;

use anyhow::Result;
use serde::Deserialize;
use wastrumentation::analysis::{AnalysisInterface, ProcessedAnalysis};

use crate::lib_compile::rust::{options::RustSource, Rust};

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
    CallPre,
    CallPost,
    CallIndirectPre,
    CallIndirectPost,
}

pub fn interface_from(hooks: &HashSet<Hook>) -> Result<AnalysisInterface> {
    let mut interface = AnalysisInterface::default();
    for hook in hooks {
        match hook {
            Hook::GenericApply => {
                interface.generic_interface = Some(AnalysisInterface::interface_generic_apply())
            }
            Hook::CallPre => {
                interface.pre_trap_call = Some(AnalysisInterface::interface_call_pre())
            }
            Hook::CallPost => {
                interface.post_trap_call = Some(AnalysisInterface::interface_call_post())
            }
            Hook::CallIndirectPre => {
                interface.pre_trap_call_indirect =
                    Some(AnalysisInterface::interface_call_indirect_pre())
            }
            Hook::CallIndirectPost => {
                interface.post_trap_call_indirect =
                    Some(AnalysisInterface::interface_call_indirect_post())
            }
        }
    }
    Ok(interface)
}
