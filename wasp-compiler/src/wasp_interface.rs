use crate::ast::wasp::WasmType;

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
