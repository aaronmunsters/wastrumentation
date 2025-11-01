use std::collections::HashSet;

use wasabi_wasm::Code;
use wasabi_wasm::FunctionType;
use wasabi_wasm::ImportOrPresent;
use wasabi_wasm::Module;
use wasabi_wasm::ValType;

use crate::compiler::{LibGeneratable, Library};
use wasabi_wasm::Function;
use wasabi_wasm::Idx;

use crate::analysis::{AnalysisInterface, WasmExport, WasmImport, WasmType};
use crate::error::InstrumentationError;
use crate::parse_nesting::HighLevelBody;
use crate::parse_nesting::LowLevelBody;

use self::block_loop::Target::{BlockPost, BlockPre, LoopPost, LoopPre, Select};
use self::branch_if::Target::{Br, BrIf, BrTable, IfThen, IfThenElse, IfThenElsePost, IfThenPost};
use self::function_application::INSTRUMENTATION_ANALYSIS_MODULE;
use self::function_call_indirect::Target::{
    IndirectPost as CallIndirectPost, IndirectPre as CallIndirectPre, Post as CallPost,
    Pre as CallPre,
};
use self::memory::Target::*;
use self::simple_operations::Target::*;
use self::table::Target::*;

pub mod block_loop;
pub mod branch_if;
pub mod function_application;
pub mod function_call_indirect;
pub mod memory;
pub mod simple_operations;
pub mod table;

pub struct Instrumented<InstrumentationLanguage: LibGeneratable> {
    pub module: Vec<u8>,
    pub instrumentation_library: Option<Library<InstrumentationLanguage>>,
}

pub fn instrument<InstrumentationLanguage: LibGeneratable>(
    module: &[u8],
    analysis_interface: &AnalysisInterface,
    target_indices: &Option<Vec<u32>>,
) -> Result<Instrumented<InstrumentationLanguage>, InstrumentationError> {
    let AnalysisInterface {
        generic_interface,
        if_then_else_trap,
        if_then_else_post_trap,
        if_then_trap,
        if_then_post_trap,
        br_trap,
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
        drop_trap,
        return_trap,
        const_i32_trap,
        const_f32_trap,
        const_i64_trap,
        const_f64_trap,
        unary_i32_to_i32,
        unary_i64_to_i32,
        unary_i64_to_i64,
        unary_f32_to_f32,
        unary_f64_to_f64,
        unary_f32_to_i32,
        unary_f64_to_i32,
        unary_i32_to_i64,
        unary_f32_to_i64,
        unary_f64_to_i64,
        unary_i32_to_f32,
        unary_i64_to_f32,
        unary_f64_to_f32,
        unary_i32_to_f64,
        unary_i64_to_f64,
        unary_f32_to_f64,
        binary_i32_i32_to_i32,
        binary_i64_i64_to_i32,
        binary_f32_f32_to_i32,
        binary_f64_f64_to_i32,
        binary_i64_i64_to_i64,
        binary_f32_f32_to_f32,
        binary_f64_f64_to_f64,
        memory_size,
        memory_grow,
        local_get_i32,
        local_set_i32,
        local_tee_i32,
        global_get_i32,
        global_set_i32,
        local_get_f32,
        local_set_f32,
        local_tee_f32,
        global_get_f32,
        global_set_f32,
        local_get_i64,
        local_set_i64,
        local_tee_i64,
        global_get_i64,
        global_set_i64,
        local_get_f64,
        local_set_f64,
        local_tee_f64,
        global_get_f64,
        global_set_f64,
        f32_store,
        f64_store,
        i32_store,
        i64_store,
        f32_load,
        f64_load,
        i32_load,
        i64_load,
        ref_func,
        ref_null,
        ref_is_null,
        table_get,
        table_set,
        table_size,
        table_grow,
        table_fill,
        table_copy,
        table_copy_get_source,
        table_copy_get_destination,
        table_copy_get_size,
        table_init,
        table_init_get_element_source,
        table_init_get_table_destination,
        table_init_get_size,
        elem_drop,
    } = analysis_interface;

    let (mut module, _offsets, _issue) =
        Module::from_bytes(module).map_err(InstrumentationError::ParseModuleError)?;

    let target_indices_including_imports: HashSet<Idx<Function>> = module
        .functions()
        .filter(|(_index, f)| !uses_reference_types(f))
        .map(|(idx, _)| idx)
        .filter(|index| {
            target_indices
                .as_ref()
                .is_none_or(|ts| ts.contains(&index.to_u32()))
        })
        .collect();

    let target_indices: HashSet<Idx<Function>> = module
        .functions()
        .filter(|(_index, f)| f.code().is_some())
        .filter(|(_index, f)| !uses_reference_types(f))
        .map(|(idx, _)| idx)
        .filter(|index| {
            target_indices
                .as_ref()
                .is_none_or(|ts| ts.contains(&index.to_u32()))
        })
        .collect();

    // For each function, generate high-level typed AST
    let target_high_level_functions: Vec<HighLevelBody> = target_indices
        .iter()
        .map(|target_function_idx| {
            let target_function = module.function(*target_function_idx);
            let code = target_function
                .code()
                .ok_or(InstrumentationError::AttemptInnerInstrumentImport)?;
            ((&module), target_function, code, target_function_idx)
                .try_into()
                .map_err(|e| InstrumentationError::LowToHighError { low_to_high_err: e })
        })
        .collect::<Result<Vec<HighLevelBody>, InstrumentationError>>()?;

    //  Install all tarps
    type TFn = fn(Idx<Function>) -> Box<dyn TransformationStrategy>;
    let traps_target_generators = [
        (pre_block, (|i| Box::new(BlockPre(i)))),
        (post_block, (|i| Box::new(BlockPost(i)))),
        (pre_loop, (|i| Box::new(LoopPre(i)))),
        (post_loop, (|i| Box::new(LoopPost(i)))),
        (select, (|i| Box::new(Select(i)))),
        (pre_trap_call, (|i| Box::new(CallPre(i)))),
        (post_trap_call, (|i| Box::new(CallPost(i)))),
        (pre_trap_call_indirect, (|i| Box::new(CallIndirectPre(i)))),
        (post_trap_call_indirect, (|i| Box::new(CallIndirectPost(i)))),
        (if_then_trap, (|i| Box::new(IfThen(i)))),
        (if_then_post_trap, (|i| Box::new(IfThenPost(i)))),
        (if_then_else_trap, (|i| Box::new(IfThenElse(i)))),
        (if_then_else_post_trap, (|i| Box::new(IfThenElsePost(i)))),
        (br_trap, (|i| Box::new(Br(i)))),
        (br_if_trap, (|i| Box::new(BrIf(i)))),
        (br_table_trap, (|i| Box::new(BrTable(i)))),
        (drop_trap, (|i| Box::new(Drop(i)))),
        (return_trap, (|i| Box::new(Return(i)))),
        (const_i32_trap, (|i| Box::new(ConstI32(i)))),
        (const_f32_trap, (|i| Box::new(ConstF32(i)))),
        (const_i64_trap, (|i| Box::new(ConstI64(i)))),
        (const_f64_trap, (|i| Box::new(ConstF64(i)))),
        (unary_i32_to_i32, (|i| Box::new(UnaryI32ToI32(i)))),
        (unary_i64_to_i32, (|i| Box::new(UnaryI64ToI32(i)))),
        (unary_i64_to_i64, (|i| Box::new(UnaryI64ToI64(i)))),
        (unary_f32_to_f32, (|i| Box::new(UnaryF32ToF32(i)))),
        (unary_f64_to_f64, (|i| Box::new(UnaryF64ToF64(i)))),
        (unary_f32_to_i32, (|i| Box::new(UnaryF32ToI32(i)))),
        (unary_f64_to_i32, (|i| Box::new(UnaryF64ToI32(i)))),
        (unary_i32_to_i64, (|i| Box::new(UnaryI32ToI64(i)))),
        (unary_f32_to_i64, (|i| Box::new(UnaryF32ToI64(i)))),
        (unary_f64_to_i64, (|i| Box::new(UnaryF64ToI64(i)))),
        (unary_i32_to_f32, (|i| Box::new(UnaryI32ToF32(i)))),
        (unary_i64_to_f32, (|i| Box::new(UnaryI64ToF32(i)))),
        (unary_f64_to_f32, (|i| Box::new(UnaryF64ToF32(i)))),
        (unary_i32_to_f64, (|i| Box::new(UnaryI32ToF64(i)))),
        (unary_i64_to_f64, (|i| Box::new(UnaryI64ToF64(i)))),
        (unary_f32_to_f64, (|i| Box::new(UnaryF32ToF64(i)))),
        (binary_i32_i32_to_i32, (|i| Box::new(BinaryI32I32toI32(i)))),
        (binary_i64_i64_to_i32, (|i| Box::new(BinaryI64I64toI32(i)))),
        (binary_f32_f32_to_i32, (|i| Box::new(BinaryF32F32toI32(i)))),
        (binary_f64_f64_to_i32, (|i| Box::new(BinaryF64F64toI32(i)))),
        (binary_i64_i64_to_i64, (|i| Box::new(BinaryI64I64toI64(i)))),
        (binary_f32_f32_to_f32, (|i| Box::new(BinaryF32F32toF32(i)))),
        (binary_f64_f64_to_f64, (|i| Box::new(BinaryF64F64toF64(i)))),
        (memory_size, (|i| Box::new(MemorySize(i)))),
        (memory_grow, (|i| Box::new(MemoryGrow(i)))),
        (local_get_i32, (|i| Box::new(LocalGetI32(i)))),
        (local_set_i32, (|i| Box::new(LocalSetI32(i)))),
        (local_tee_i32, (|i| Box::new(LocalTeeI32(i)))),
        (global_get_i32, (|i| Box::new(GlobalGetI32(i)))),
        (global_set_i32, (|i| Box::new(GlobalSetI32(i)))),
        (local_get_f32, (|i| Box::new(LocalGetF32(i)))),
        (local_set_f32, (|i| Box::new(LocalSetF32(i)))),
        (local_tee_f32, (|i| Box::new(LocalTeeF32(i)))),
        (global_get_f32, (|i| Box::new(GlobalGetF32(i)))),
        (global_set_f32, (|i| Box::new(GlobalSetF32(i)))),
        (local_get_i64, (|i| Box::new(LocalGetI64(i)))),
        (local_set_i64, (|i| Box::new(LocalSetI64(i)))),
        (local_tee_i64, (|i| Box::new(LocalTeeI64(i)))),
        (global_get_i64, (|i| Box::new(GlobalGetI64(i)))),
        (global_set_i64, (|i| Box::new(GlobalSetI64(i)))),
        (local_get_f64, (|i| Box::new(LocalGetF64(i)))),
        (local_set_f64, (|i| Box::new(LocalSetF64(i)))),
        (local_tee_f64, (|i| Box::new(LocalTeeF64(i)))),
        (global_get_f64, (|i| Box::new(GlobalGetF64(i)))),
        (global_set_f64, (|i| Box::new(GlobalSetF64(i)))),
        (f32_store, (|i| Box::new(F32Store(i)))),
        (f64_store, (|i| Box::new(F64Store(i)))),
        (i32_store, (|i| Box::new(I32Store(i)))),
        (i64_store, (|i| Box::new(I64Store(i)))),
        (f32_load, (|i| Box::new(F32Load(i)))),
        (f64_load, (|i| Box::new(F64Load(i)))),
        (i32_load, (|i| Box::new(I32Load(i)))),
        (i64_load, (|i| Box::new(I64Load(i)))),
        (ref_func, (|i| Box::new(RefFunc(i)))),
        (ref_null, (|i| Box::new(RefNull(i)))),
        (ref_is_null, (|i| Box::new(RefIsNull(i)))),
        (table_get, (|i| Box::new(TableGet(i)))),
        (table_set, (|i| Box::new(TableSet(i)))),
        (table_size, (|i| Box::new(TableSize(i)))),
        (table_grow, (|i| Box::new(TableGrow(i)))),
        (table_fill, (|i| Box::new(TableFill(i)))),
        // table copy is currently handled separately
        // table init is currently handled separately
        (elem_drop, (|i| Box::new(ElemDrop(i)))),
    ] as [(&Option<WasmExport>, TFn); 84];

    let mut targets: Vec<Box<dyn TransformationStrategy>> = traps_target_generators
        .into_iter()
        .filter_map(|(export, target_gen)| {
            export
                .as_ref()
                .map(|export| module.install(export))
                .map(target_gen)
        })
        .collect();

    // table copy target
    if let Some(table_copy_trap) = table_copy {
        let table_copy_trap_idx = module.install(table_copy_trap);
        let table_copy_get_src_idx = table_copy_get_source
            .as_ref()
            .map(|e| module.install(e))
            .expect("table_copy_get_source required when table_copy is present");
        let table_copy_get_dst_idx = table_copy_get_destination
            .as_ref()
            .map(|e| module.install(e))
            .expect("table_copy_get_destination required when table_copy is present");
        let table_copy_get_size_idx = table_copy_get_size
            .as_ref()
            .map(|e| module.install(e))
            .expect("table_copy_get_size required when table_copy is present");

        let table_copy_target = Box::new(TableCopy {
            trap_idx: table_copy_trap_idx,
            get_src_idx: table_copy_get_src_idx,
            get_dst_idx: table_copy_get_dst_idx,
            get_size_idx: table_copy_get_size_idx,
        }) as Box<dyn TransformationStrategy>;

        targets.push(table_copy_target);
    }

    if let Some(table_init_trap) = table_init {
        let table_init_trap_idx = module.install(table_init_trap);
        let table_init_get_elem_src_idx = table_init_get_element_source
            .as_ref()
            .map(|e| module.install(e))
            .expect("table_init_get_element_source required when table_init is present");
        let table_init_get_table_dst_idx = table_init_get_table_destination
            .as_ref()
            .map(|e| module.install(e))
            .expect("table_init_get_table_destination required when table_init is present");
        let table_init_get_size_idx = table_init_get_size
            .as_ref()
            .map(|e| module.install(e))
            .expect("table_init_get_size required when table_init is present");

        let table_init_target = Box::new(TableInit {
            trap_idx: table_init_trap_idx,
            get_elem_source_idx: table_init_get_elem_src_idx,
            get_table_destination_idx: table_init_get_table_dst_idx,
            get_size_idx: table_init_get_size_idx,
        }) as Box<dyn TransformationStrategy>;

        targets.push(table_init_target);
    }

    let transformed_bodies: Vec<HighLevelBody> = target_high_level_functions
        .into_iter()
        .map(|high_level_body| {
            targets.iter().fold(high_level_body, |transformed, target| {
                target.transform(&transformed, &mut module)
            })
        })
        .collect();

    for (target_function_idx, transformed_body) in target_indices.iter().zip(transformed_bodies) {
        let LowLevelBody(transformed_low_level_body) = transformed_body.into();
        let locals = module
            .function(*target_function_idx)
            .code()
            .ok_or(InstrumentationError::AttemptInnerInstrumentImport)?
            .locals
            .clone();
        module.function_mut(*target_function_idx).code = ImportOrPresent::Present(Code {
            body: transformed_low_level_body,
            locals,
        });
    }

    let instrumentation_library =
        generic_interface
            .as_ref()
            .map(|(generic_import, generic_export)| {
                function_application::instrument::<InstrumentationLanguage>(
                    &mut module,
                    &target_indices_including_imports,
                    generic_import,
                    generic_export,
                )
            });

    memory::inject_memory_loads(&mut module);
    memory::inject_memory_stores(&mut module);
    memory::inject_memory_grow(&mut module);
    memory::inject_memory_size(&mut module);

    Ok(Instrumented {
        module: module
            .to_bytes()
            .map_err(InstrumentationError::EncodeError)?,
        instrumentation_library,
    })
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
}

impl Instrumentable for Module {
    fn install(&mut self, export: &WasmExport) -> Idx<Function> {
        self.add_function_import(
            export.as_function_type(),
            INSTRUMENTATION_ANALYSIS_MODULE.to_string(),
            export.name.to_string(),
        )
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
    fn transform(&self, high_level_body: &HighLevelBody, module: &mut Module) -> HighLevelBody;
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
