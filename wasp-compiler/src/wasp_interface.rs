use crate::ast::wasp::WasmType;

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

// TODO: are `inputs` and `outputs` used?
#[derive(Debug, PartialEq, Eq, Default)]
pub struct WaspInterface {
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
