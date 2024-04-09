use std::collections::HashSet;

use wasabi_wasm::FunctionType;
use wasabi_wasm::Module;
use wasabi_wasm::ValType;
use wasp_compiler::ast::assemblyscript::AssemblyScriptProgram;
use wasp_compiler::ast::wasp::WasmType;
use wasp_compiler::wasp_interface::WasmExport;
use wasp_compiler::wasp_interface::WasmImport;
use wasp_compiler::wasp_interface::WaspInterface;

use wasabi_wasm::Function;
use wasabi_wasm::Idx;

use self::function_call::TargetCall;

pub mod branch_if;
pub mod function_application;
pub mod function_call;
pub mod function_call_indirect;

pub struct InstrumentationResult {
    pub module: Vec<u8>,
    pub instrumentation_lib: AssemblyScriptProgram,
}

pub fn instrument(module: &[u8], wasp_interface: WaspInterface) -> InstrumentationResult {
    let WaspInterface {
        generic_interface,
        if_then_else_trap,
        if_then_trap,
        br_if_trap,
        pre_trap_call,
        pre_trap_call_indirect,
        post_trap_call,
        post_trap_call_indirect,
        br_table_trap,
        .. // TODO: remove?
    } = wasp_interface;
    let mut instrumentation_lib = String::new();
    let (mut module, _offsets, _issue) = Module::from_bytes(module).unwrap();
    let pre_instrumentation_function_indices: HashSet<Idx<Function>> = module
        .functions()
        .filter(|(_index, f)| f.code().is_some())
        .map(|(idx, _)| idx)
        .collect();

    // Instrument call / call_indirect first, to prevent new calls to be instrumented too.
    let target_call = match (pre_trap_call, post_trap_call) {
        (None, None) => TargetCall::None,
        (Some(pre_call_trap), None) => TargetCall::Pre(pre_call_trap),
        (None, Some(post_call_trap)) => TargetCall::Post(post_call_trap),
        (Some(pre_call_trap), Some(post_call_trap)) => TargetCall::Both {
            pre_call_trap,
            post_call_trap,
        },
    };

    target_call
        .instrument(&mut module, &pre_instrumentation_function_indices)
        .unwrap();

    if let Some(trap_export) = pre_trap_call_indirect {
        function_call_indirect::instrument(
            &mut module,
            &pre_instrumentation_function_indices,
            trap_export,
            function_call_indirect::Target::CallIndirectPre,
        )
        .unwrap() // TODO: handle
    }

    if let Some(trap_export) = post_trap_call_indirect {
        function_call_indirect::instrument(
            &mut module,
            &pre_instrumentation_function_indices,
            trap_export,
            function_call_indirect::Target::CallIndirectPost,
        )
        .unwrap() // TODO: handle
    }

    if let Some(trap_export) = if_then_trap {
        branch_if::instrument(
            &mut module,
            &pre_instrumentation_function_indices,
            trap_export,
            branch_if::Target::IfThen,
        )
        .unwrap() // TODO: handle
    };

    if let Some(trap_export) = if_then_else_trap {
        branch_if::instrument(
            &mut module,
            &pre_instrumentation_function_indices,
            trap_export,
            branch_if::Target::IfThenElse,
        )
        .unwrap() // TODO: handle
    };

    if let Some((generic_import, generic_export)) = generic_interface {
        let generic_function_instrumentation_lib = function_application::instrument(
            &mut module,
            &pre_instrumentation_function_indices,
            generic_import,
            generic_export,
        );

        instrumentation_lib.push_str(&generic_function_instrumentation_lib.content);
    };

    if let Some(trap_export) = br_if_trap {
        branch_if::instrument(
            &mut module,
            &pre_instrumentation_function_indices,
            trap_export,
            branch_if::Target::BrIf,
        )
        .unwrap() // TODO: handle
    }

    if let Some(trap_export) = br_table_trap {
        branch_if::instrument(
            &mut module,
            &pre_instrumentation_function_indices,
            trap_export,
            branch_if::Target::BrTable,
        )
        .unwrap() // TODO: handle
    }

    InstrumentationResult {
        instrumentation_lib: AssemblyScriptProgram {
            content: instrumentation_lib,
        },
        module: module.to_bytes().unwrap(),
    }
}

struct WasabiValType(ValType);
impl From<WasmType> for WasabiValType {
    fn from(value: WasmType) -> Self {
        match value {
            WasmType::I32 => WasabiValType(ValType::I32),
            WasmType::F32 => WasabiValType(ValType::F32),
            WasmType::I64 => WasabiValType(ValType::I64),
            WasmType::F64 => WasabiValType(ValType::F64),
        }
    }
}

struct ValTypeVec(Vec<ValType>);
impl From<Vec<WasmType>> for ValTypeVec {
    fn from(value: Vec<WasmType>) -> Self {
        ValTypeVec(
            value
                .into_iter()
                .map(|wasm_type| WasabiValType::from(wasm_type).0)
                .collect(),
        )
    }
}

trait FunctionTypeConvertible {
    fn as_function_type(&self) -> FunctionType;
}

impl FunctionTypeConvertible for WasmExport {
    fn as_function_type(&self) -> FunctionType {
        let WasmExport { args, results, .. } = self;
        let args: &[ValType] = &ValTypeVec::from(args.clone()).0;
        let results: &[ValType] = &ValTypeVec::from(results.clone()).0;
        FunctionType::new(args, results)
    }
}

impl FunctionTypeConvertible for WasmImport {
    fn as_function_type(&self) -> FunctionType {
        let WasmImport { args, results, .. } = self;
        let args: &[ValType] = &ValTypeVec::from(args.clone()).0;
        let results: &[ValType] = &ValTypeVec::from(results.clone()).0;
        FunctionType::new(args, results)
    }
}
