use std::path::PathBuf;

pub const FUNCTION_NAME_BLOCK_PRE: &str = "block_pre";
pub const FUNCTION_NAME_BLOCK_POST: &str = "block_post";
pub const FUNCTION_NAME_CALL_BASE: &str = "call_base";
pub const FUNCTION_NAME_GENERIC_APPLY: &str = "generic_apply";
pub const FUNCTION_NAME_LOOP_PRE: &str = "loop_pre";
pub const FUNCTION_NAME_LOOP_POST: &str = "loop_post";
pub const FUNCTION_NAME_SELECT: &str = "specialized_select";
pub const FUNCTION_NAME_SPECIALIZED_BR_IF: &str = "specialized_br_if";
pub const FUNCTION_NAME_SPECIALIZED_BR_TABLE: &str = "specialized_br_table";
pub const FUNCTION_NAME_SPECIALIZED_CALL_POST: &str = "specialized_call_post";
pub const FUNCTION_NAME_SPECIALIZED_CALL_PRE: &str = "specialized_call_pre";
pub const FUNCTION_NAME_SPECIALIZED_CALL_INDIRECT_POST: &str = "specialized_call_indirect_post";
pub const FUNCTION_NAME_SPECIALIZED_CALL_INDIRECT_PRE: &str = "specialized_call_indirect_pre";
pub const FUNCTION_NAME_SPECIALIZED_IF_THEN: &str = "specialized_if_then_k";
pub const FUNCTION_NAME_SPECIALIZED_IF_THEN_ELSE: &str = "specialized_if_then_else_k";
pub const NAMESPACE_TRANSFORMED_INPUT: &str = "transformed_input";

use anyhow::{bail, Result};
use assemblyscript::ASRoot;
use wasp_compiler::CompilationResult;

pub mod assemblyscript;
mod rust;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum WasmType {
    I32,
    F32,
    I64,
    F64,
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

#[derive(Debug, PartialEq, Eq, Default)]
pub struct AnalysisInterface {
    pub generic_interface: Option<(WasmExport, WasmImport)>,
    pub if_then_trap: Option<WasmExport>,
    pub if_then_else_trap: Option<WasmExport>,
    pub br_if_trap: Option<WasmExport>,
    pub br_table_trap: Option<WasmExport>,
    pub pre_trap_call: Option<WasmExport>,
    pub pre_trap_call_indirect: Option<WasmExport>,
    pub post_trap_call: Option<WasmExport>,
    pub post_trap_call_indirect: Option<WasmExport>,
    pub pre_block: Option<WasmExport>,
    pub post_block: Option<WasmExport>,
    pub pre_loop: Option<WasmExport>,
    pub post_loop: Option<WasmExport>,
    pub select: Option<WasmExport>,
}

pub struct AnalysisCompilationResult {
    pub analysis_wasm: Vec<u8>,
    pub analysis_interface: AnalysisInterface,
}

#[derive(Clone)]
pub enum Analysis {
    Rust {
        manifest: PathBuf,
        hooks: Vec<String>,
    },
    AssemblyScript {
        wasp_source: String,
    },
}

impl Analysis {
    pub fn compile(&self, wastrumenter: &Wastrumenter) -> Result<AnalysisCompilationResult> {
        match self {
            Analysis::Rust { manifest, hooks } => {
                let analysis_wasm =
                    rust_to_wasm_compiler::RustToWasmCompiler::new()?.compile(manifest)?;
                let analysis_interface = rust::interface_from(hooks)?;
                Ok(AnalysisCompilationResult {
                    analysis_wasm,
                    analysis_interface,
                })
            }
            Analysis::AssemblyScript { wasp_source } => {
                let CompilationResult {
                    wasp_root,
                    join_points: _,
                } = wasp_compiler::compile(wasp_source)?;

                let analysis_interface = AnalysisInterface::from(&wasp_root);
                let as_root = ASRoot(wasp_root);
                let wasp_assemblyscript = as_root.into();
                let analysis_wasm = wastrumenter.compile(wasp_assemblyscript)?;

                Ok(AnalysisCompilationResult {
                    analysis_wasm,
                    analysis_interface,
                })
            }
        }
    }
}

type ApplyInterface = (WasmExport, WasmImport);

use crate::{analysis::WasmType::I32, Wastrumenter};

impl AnalysisInterface {
    fn interface_generic_apply() -> ApplyInterface {
        (
            WasmExport {
                name: FUNCTION_NAME_GENERIC_APPLY.into(),
                // f_apply, argc, resc, sigv, sigtypv
                args: vec![I32, I32, I32, I32, I32],
                results: vec![],
            },
            WasmImport {
                namespace: NAMESPACE_TRANSFORMED_INPUT.into(),
                name: FUNCTION_NAME_CALL_BASE.into(),
                // f_apply, sigv
                args: vec![I32, I32],
                results: vec![],
            },
        )
    }

    fn interface_if_then() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_SPECIALIZED_IF_THEN.into(),
            // path_kontinuation
            args: vec![I32],
            // path_kontinuation
            results: vec![I32],
        }
    }

    fn interface_if_then_else() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_SPECIALIZED_IF_THEN_ELSE.into(),
            // path_kontinuation
            args: vec![I32],
            // path_kontinuation
            results: vec![I32],
        }
    }

    fn interface_br_if() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_SPECIALIZED_BR_IF.into(),
            // path_kontinuation, label
            // TODO: is `label` interesting? This value does not change at runtime
            args: vec![I32, I32],
            // path_kontinuation
            results: vec![I32],
        }
    }

    fn interface_br_table() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_SPECIALIZED_BR_TABLE.into(),
            // table_target_index, default
            args: vec![I32, I32],
            // table_target_index
            results: vec![I32],
        }
    }

    fn interface_call_pre() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_SPECIALIZED_CALL_PRE.into(),
            // function_target
            args: vec![I32],
            // void
            results: vec![],
        }
    }

    fn interface_call_post() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_SPECIALIZED_CALL_POST.into(),
            // function_target
            args: vec![I32],
            // void
            results: vec![],
        }
    }

    fn interface_call_indirect_pre() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_SPECIALIZED_CALL_INDIRECT_PRE.into(),
            // function_table_index, function_table
            args: vec![I32, I32],
            // void
            results: vec![I32],
        }
    }

    fn interface_call_indirect_post() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_SPECIALIZED_CALL_INDIRECT_POST.into(),
            // function_table
            args: vec![I32],
            // void
            results: vec![],
        }
    }

    fn interface_block_pre() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_BLOCK_PRE.into(),
            args: vec![],
            results: vec![],
        }
    }
    fn interface_block_post() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_BLOCK_POST.into(),
            args: vec![],
            results: vec![],
        }
    }
    fn interface_loop_pre() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_LOOP_PRE.into(),
            args: vec![],
            results: vec![],
        }
    }
    fn interface_loop_post() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_LOOP_POST.into(),
            args: vec![],
            results: vec![],
        }
    }
    fn interface_select() -> WasmExport {
        WasmExport {
            name: FUNCTION_NAME_SELECT.into(),
            // condition
            args: vec![I32],
            // kontinuation
            results: vec![I32],
        }
    }
}
