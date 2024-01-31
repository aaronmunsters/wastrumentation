use std::collections::HashSet;

use wasabi_wasm::Module;
use wasp_compiler::ast::assemblyscript::AssemblyScriptProgram;
use wasp_compiler::wasp_interface::WaspInterface;

use wasabi_wasm::Function;
use wasabi_wasm::Idx;

pub mod branch_if;
pub mod function_application;

pub struct InstrumentationResult {
    pub module: Vec<u8>,
    pub instrumentation_lib: AssemblyScriptProgram,
}

pub fn instrument(module: &[u8], wasp_interface: WaspInterface) -> InstrumentationResult {
    let (mut module, _offsets, _issue) = Module::from_bytes(module).unwrap();
    let pre_instrumentation_function_indices: HashSet<Idx<Function>> = module
        .functions()
        .filter(|(_index, f)| f.code().is_some())
        .map(|(idx, _)| idx)
        .collect();

    if let Some((generic_import, generic_export)) = wasp_interface.generic_interface {
        let instrumentation_lib = function_application::instrument(
            &mut module,
            &pre_instrumentation_function_indices,
            generic_import,
            generic_export,
        );

        InstrumentationResult {
            instrumentation_lib,
            module: module.to_bytes().unwrap(),
        }
    } else {
        todo!()
    }

    // TODO: specific instrumentation
}
