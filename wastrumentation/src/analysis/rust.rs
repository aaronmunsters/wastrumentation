use super::bail;
use super::AnalysisInterface;
use super::Result;

pub fn interface_from(hooks: &[String]) -> Result<AnalysisInterface> {
    let mut interface = AnalysisInterface::default();
    for hook in hooks {
        match hook.as_str() {
            "advice-call-before" => {
                interface.pre_trap_call = Some(AnalysisInterface::interface_call_pre())
            }
            "advice-call-after" => {
                interface.post_trap_call = Some(AnalysisInterface::interface_call_post())
            }
            "advice-call-indirect-before" => {
                interface.pre_trap_call_indirect =
                    Some(AnalysisInterface::interface_call_indirect_pre())
            }
            "advice-call-indirect-after" => {
                interface.post_trap_call_indirect =
                    Some(AnalysisInterface::interface_call_indirect_post())
            }
            "advice-apply" => {
                interface.generic_interface = Some(AnalysisInterface::interface_generic_apply())
            }
            unknown_hook => bail!("Unknown hook target: {unknown_hook}"),
        }
    }
    Ok(interface)
}
