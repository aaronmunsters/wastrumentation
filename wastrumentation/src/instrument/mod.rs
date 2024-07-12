use std::collections::HashSet;

use wasabi_wasm::Code;
use wasabi_wasm::FunctionType;
use wasabi_wasm::ImportOrPresent;
use wasabi_wasm::Module;
use wasabi_wasm::ValType;
use wasp_compiler::ast::assemblyscript::AssemblyScriptProgram;

use wasabi_wasm::Function;
use wasabi_wasm::Idx;

use crate::analysis::{AnalysisInterface, WasmExport, WasmImport, WasmType};
use crate::parse_nesting::HighLevelBody;
use crate::parse_nesting::LowLevelBody;

use self::block_loop::Target::{BlockPost, BlockPre, LoopPost, LoopPre, Select};
use self::branch_if::Target::{BrIf, BrTable, IfThen, IfThenElse};
use self::function_application::INSTRUMENTATION_ANALYSIS_MODULE;
use self::function_call::TargetCall;
use self::function_call_indirect::Target::{CallIndirectPost, CallIndirectPre};

pub mod block_loop;
pub mod branch_if;
pub mod function_application;
pub mod function_call;
pub mod function_call_indirect;

pub struct InstrumentationResult {
    pub module: Vec<u8>,
    pub instrumentation_lib: AssemblyScriptProgram,
}

pub fn instrument(module: &[u8], analysis_interface: &AnalysisInterface) -> InstrumentationResult {
    let AnalysisInterface {
        generic_interface,
        if_then_else_trap,
        if_then_trap,
        br_if_trap,
        pre_trap_call,
        post_trap_call,
        pre_trap_call_indirect,
        post_trap_call_indirect,
        br_table_trap,
        pre_block,
        post_block,
        pre_loop,
        post_loop,
        select,
    } = analysis_interface;

    let mut instrumentation_lib = String::new();

    let (mut module, _offsets, _issue) = Module::from_bytes(module).unwrap();

    let target_indices: HashSet<Idx<Function>> = module
        .functions()
        .filter(|(_index, f)| f.code().is_some())
        .filter(|(_index, f)| !uses_reference_types(f))
        .map(|(idx, _)| idx)
        .collect();

    // Instrument call / call_indirect first, to prevent new call instructions to be instrumented too.
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
        .instrument(&mut module, &target_indices)
        .unwrap();

    pre_block
        .as_ref()
        .map(|export| module.install(export))
        .map(|index| module.instrument_function_bodies(&target_indices, &BlockPre(index)));

    post_block
        .as_ref()
        .map(|export| module.install(export))
        .map(|index| module.instrument_function_bodies(&target_indices, &BlockPost(index)));

    pre_loop
        .as_ref()
        .map(|export| module.install(export))
        .map(|index| module.instrument_function_bodies(&target_indices, &LoopPre(index)));

    post_loop
        .as_ref()
        .map(|export| module.install(export))
        .map(|index| module.instrument_function_bodies(&target_indices, &LoopPost(index)));

    select
        .as_ref()
        .map(|export| module.install(export))
        .map(|index| module.instrument_function_bodies(&target_indices, &Select(index)));

    pre_trap_call_indirect
        .as_ref()
        .map(|export| module.install(export))
        .map(|index| module.instrument_function_bodies(&target_indices, &CallIndirectPre(index)));

    post_trap_call_indirect
        .as_ref()
        .map(|export| module.install(export))
        .map(|index| module.instrument_function_bodies(&target_indices, &CallIndirectPost(index)));

    if_then_trap
        .as_ref()
        .map(|export| module.install(export))
        .map(|index| module.instrument_function_bodies(&target_indices, &IfThen(index)));

    if_then_else_trap
        .as_ref()
        .map(|export| module.install(export))
        .map(|index| module.instrument_function_bodies(&target_indices, &IfThenElse(index)));

    br_if_trap
        .as_ref()
        .map(|export| module.install(export))
        .map(|index| module.instrument_function_bodies(&target_indices, &BrIf(index)));

    br_table_trap
        .as_ref()
        .map(|export| module.install(export))
        .map(|index| module.instrument_function_bodies(&target_indices, &BrTable(index)));

    if let Some((generic_import, generic_export)) = generic_interface {
        let generic_function_instrumentation_lib = function_application::instrument(
            &mut module,
            &target_indices,
            generic_import,
            generic_export,
        );

        instrumentation_lib.push_str(&generic_function_instrumentation_lib.content);
    };

    InstrumentationResult {
        module: module.to_bytes().unwrap(),
        instrumentation_lib: AssemblyScriptProgram {
            content: instrumentation_lib,
        },
    }
}

fn uses_reference_types(f: &Function) -> bool {
    for ty_ in f.type_.inputs() {
        match ty_ {
            ValType::I32 | ValType::I64 | ValType::F32 | ValType::F64 => continue,
            ValType::Ref(_) => return true,
        }
    }
    for ty_ in f.type_.results() {
        match ty_ {
            ValType::I32 | ValType::I64 | ValType::F32 | ValType::F64 => continue,
            ValType::Ref(_) => return true,
        }
    }
    false
}

trait Instrumentable {
    fn install(&mut self, export: &WasmExport) -> Idx<Function>;
    fn instrument_function_bodies(
        &mut self,
        target_functions: &HashSet<Idx<Function>>,
        instrumentation_strategy: &impl TransformationStrategy,
    ) -> Result<(), &'static str>;
}
impl Instrumentable for Module {
    fn install(&mut self, export: &WasmExport) -> Idx<Function> {
        self.add_function_import(
            export.as_function_type(),
            INSTRUMENTATION_ANALYSIS_MODULE.to_string(),
            export.name.to_string(),
        )
    }

    fn instrument_function_bodies(
        &mut self,
        target_functions: &HashSet<Idx<Function>>,
        instrumentation_strategy: &impl TransformationStrategy,
    ) -> Result<(), &'static str> {
        for target_function_idx in target_functions {
            let target_function = self.function_mut(*target_function_idx);
            let code = target_function.code_mut();
            match code {
                None => return Err("Attempt to instrument an `import` function"),
                Some(code) => {
                    let high_level_body: HighLevelBody =
                        LowLevelBody(code.body.clone()).try_into()?;
                    let high_level_body_transformed =
                        high_level_body.transform_for(instrumentation_strategy);
                    let LowLevelBody(transformed_low_level_body) =
                        high_level_body_transformed.into();

                    target_function.code = ImportOrPresent::Present(Code {
                        body: transformed_low_level_body,
                        locals: code.locals.clone(),
                    });
                }
            }
        }
        Ok(())
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

pub trait TransformationStrategy {
    fn transform(&self, high_level_body: &HighLevelBody) -> HighLevelBody;
}

impl HighLevelBody {
    #[must_use]
    pub fn transform_for(&self, transformation_strategy: &impl TransformationStrategy) -> Self {
        transformation_strategy.transform(self)
    }
}

#[cfg(test)]
mod tests {
    use wasabi_wasm::RefType::{ExternRef, FuncRef};
    use wasabi_wasm::ValType::{self, Ref, F32, F64, I32, I64};
    use wasabi_wasm::{Code, Function, FunctionType};

    use super::uses_reference_types;

    #[test]
    fn test_uses_reference_types() {
        let assertions: &[(&[ValType], &[ValType], bool)] = &[
            (&[], &[], false),
            (&[F32, F64], &[I32, I64], false),
            (&[I32, I32], &[], false),
            (&[], &[I32, I32], false),
            (&[Ref(FuncRef)], &[], true),
            (&[], &[Ref(FuncRef)], true),
            (&[Ref(ExternRef)], &[], true),
            (&[], &[Ref(ExternRef)], true),
            (&[F32, F32, F32, Ref(FuncRef)], &[], true),
            (&[], &[F64, F64, F64, Ref(FuncRef)], true),
            (&[I32, I32, I32, Ref(FuncRef)], &[], true),
            (&[], &[I64, I64, I64, Ref(FuncRef)], true),
            (
                &[I32, I32, I32, Ref(ExternRef)],
                &[I64, I64, I64, Ref(ExternRef)],
                true,
            ),
        ];
        for (inputs, results, uses_reference) in assertions {
            let ft = FunctionType::new(inputs, results);
            let fc = Function::new(ft, Code::new(), vec![]);
            assert_eq!(uses_reference_types(&fc), *uses_reference)
        }
    }
}
