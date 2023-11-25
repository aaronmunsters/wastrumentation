use crate::ast::wasp::{
    AdviceDefinition, ApplyHookSignature, ApplySpe, TrapApply, TrapSignature, WasmType, WaspRoot,
};

pub const TRANSFORMED_INPUT_NS: &str = "transformed_input";
pub const GENERIC_APPLY_FUNCTION_NAME: &str = "generic_apply";
pub const CALL_BASE: &str = "generic_apply";

#[derive(Debug, PartialEq, Eq, Default)]
pub struct WaspInterface {
    pub inputs: Vec<WasmImport>,
    pub outputs: Vec<WasmExport>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct WasmImport {
    pub namespace: String,
    pub name: String,
    pub args: Vec<WasmType>,
    pub results: Vec<WasmType>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct WasmExport {
    pub name: String,
    pub args: Vec<WasmType>,
    pub results: Vec<WasmType>,
}

impl From<&WaspRoot> for WaspInterface {
    fn from(wasp_root: &WaspRoot) -> Self {
        let mut wasm_imports: Vec<WasmImport> = Vec::new();
        let mut wasm_exports: Vec<WasmExport> = Vec::new();
        let WaspRoot(advice_definitions) = wasp_root;
        for advice_definition in advice_definitions {
            if let AdviceDefinition::AdviceTrap(trap_signature) = advice_definition {
                match trap_signature {
                    TrapSignature::TrapApply(TrapApply {
                        apply_hook_signature: ApplyHookSignature::Gen(_),
                        ..
                    }) => {
                        wasm_exports.push(WasmExport {
                            name: GENERIC_APPLY_FUNCTION_NAME.into(),
                            args: vec![
                                WasmType::I32, // f_apply
                                WasmType::I32, // argc
                                WasmType::I32, // resc
                                WasmType::I32, // sigv
                                WasmType::I32, // sigtypv
                            ],
                            results: vec![],
                        });
                        wasm_imports.push(WasmImport {
                            namespace: GENERIC_APPLY_FUNCTION_NAME.into(),
                            name: CALL_BASE.into(),
                            args: vec![
                                WasmType::I32, // f_apply
                                WasmType::I32, // sigv
                            ],
                            results: vec![],
                        });
                    }
                    TrapSignature::TrapApply(TrapApply {
                        apply_hook_signature:
                            ApplyHookSignature::Spe(ApplySpe {
                                mutable_signature,
                                parameters_arguments,
                                parameters_results,
                                ..
                            }),
                        ..
                    }) => {
                        wasm_imports.push(WasmImport::for_extern_call_base(
                            *mutable_signature,
                            parameters_arguments,
                            parameters_results,
                        ));
                        wasm_exports.push(WasmExport::for_exported_apply_trap(
                            *mutable_signature,
                            parameters_arguments,
                            parameters_results,
                        ));
                    }
                }
            };
        }
        Self {
            inputs: wasm_imports,
            outputs: wasm_exports,
        }
    }
}

// TODO: tests
