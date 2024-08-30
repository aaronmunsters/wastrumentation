use std::collections::HashSet;

use serde::Deserialize;

use super::AnalysisInterface;
use super::Result;

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
