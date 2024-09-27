use core::panic;

use super::TransformationStrategy;
use crate::parse_nesting::{
    BodyInner, HighLevelBody, HighLevelInstr as Instr, TypedHighLevelInstr,
};

use wasabi_wasm::types::InferredInstructionType;
use wasabi_wasm::{
    Function, FunctionType, GlobalOp, Idx, LoadOp, LocalOp, Memarg, Module, StoreOp, Val, ValType,
};

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Target {
    MemorySize(Idx<Function>),
    MemoryGrow(Idx<Function>),

    // Local: Get / Set / Tee
    // - I32
    LocalGetI32(Idx<Function>),
    LocalSetI32(Idx<Function>),
    LocalTeeI32(Idx<Function>),
    GlobalGetI32(Idx<Function>),
    GlobalSetI32(Idx<Function>),
    // - F32
    LocalGetF32(Idx<Function>),
    LocalSetF32(Idx<Function>),
    LocalTeeF32(Idx<Function>),
    GlobalGetF32(Idx<Function>),
    GlobalSetF32(Idx<Function>),
    // - I64
    LocalGetI64(Idx<Function>),
    LocalSetI64(Idx<Function>),
    LocalTeeI64(Idx<Function>),
    GlobalGetI64(Idx<Function>),
    GlobalSetI64(Idx<Function>),
    // - F64
    LocalGetF64(Idx<Function>),
    LocalSetF64(Idx<Function>),
    LocalTeeF64(Idx<Function>),
    GlobalGetF64(Idx<Function>),
    GlobalSetF64(Idx<Function>),

    // Memory
    // - store
    F32Store(Idx<Function>),
    F64Store(Idx<Function>),
    I32Store(Idx<Function>),
    I64Store(Idx<Function>),
    // - load
    F32Load(Idx<Function>),
    F64Load(Idx<Function>),
    I32Load(Idx<Function>),
    I64Load(Idx<Function>),
}

impl TransformationStrategy for Target {
    fn transform(&self, high_level_body: &HighLevelBody) -> HighLevelBody {
        let HighLevelBody(body) = high_level_body;
        let transformed_body = transform(body, *self);
        HighLevelBody(transformed_body)
    }
}

fn transform(body: &BodyInner, target: Target) -> BodyInner {
    let mut result = Vec::new();

    for typed_instr @ TypedHighLevelInstr { instr, .. } in body {
        match (target, instr) {
            (Target::MemorySize(trap_idx), Instr::MemorySize(idx)) => {
                result.extend_from_slice(&[
                    // []                   // Perform operation
                    typed_instr.place_original(instr.clone()),
                    // [size:I32]           // Push memory index on stack
                    typed_instr.instrument_with(Instr::Const(Val::I64(idx.to_u32().into()))),
                    // [size:I32,index:I64] // Call trap
                    typed_instr.instrument_with(Instr::Call(trap_idx)),
                    // [size:I32]
                ]);
                continue;
            }
            (Target::MemoryGrow(trap_idx), Instr::MemoryGrow(idx)) => {
                result.extend_from_slice(&[
                    // [amount:I32]                   // Push memory index on stack
                    typed_instr.instrument_with(Instr::Const(Val::I64(idx.to_u32().into()))),
                    // [amount:I32,index:I64]         // Call to instrumentation
                    typed_instr.instrument_with(Instr::Call(trap_idx)),
                    // [previous-size-or-neg-one:I32]
                ]);
                continue;
            }
            _ => (),
        }

        {
            use GlobalOp::{Get as GGet, Set as GSet};
            use Instr::{Global, Local};
            use LocalOp::{Get as LGet, Set as LSet, Tee};
            use Target::*;
            use ValType::{F32, F64, I32, I64};

            match typed_instr.type_ {
                InferredInstructionType::Reachable(type_) => {
                    let (params, results) = (type_.inputs(), type_.results());
                    match (target, &typed_instr.instr, params, results) {
                        (LocalGetI32(trap_idx), Local(LGet, get_idx), &[], &[I32])
                        | (LocalGetF32(trap_idx), Local(LGet, get_idx), &[], &[F32])
                        | (LocalGetI64(trap_idx), Local(LGet, get_idx), &[], &[I64])
                        | (LocalGetF64(trap_idx), Local(LGet, get_idx), &[], &[F64]) => {
                            result.extend_from_slice(&[
                                // Perform operation
                                typed_instr.place_original(instr.clone()),
                                // Push get-index
                                typed_instr.instrument_with(
                                    Instr::Const(Val::I64(i64::from((get_idx).to_u32()))).clone(),
                                ),
                                // [value-to-write] // Call trap (in value, get-index -> out value)
                                typed_instr.instrument_with(Instr::Call(trap_idx)),
                            ]);
                            continue;
                        }
                        (GlobalGetI32(trap_idx), Global(GGet, get_idx), &[], &[I32])
                        | (GlobalGetF32(trap_idx), Global(GGet, get_idx), &[], &[F32])
                        | (GlobalGetI64(trap_idx), Global(GGet, get_idx), &[], &[I64])
                        | (GlobalGetF64(trap_idx), Global(GGet, get_idx), &[], &[F64]) => {
                            result.extend_from_slice(&[
                                // Perform operation
                                typed_instr.place_original(instr.clone()),
                                // Push get-index
                                typed_instr.instrument_with(
                                    Instr::Const(Val::I64(i64::from((get_idx).to_u32()))).clone(),
                                ),
                                // [value-to-write] // Call trap (in value, get-index -> out value)
                                typed_instr.instrument_with(Instr::Call(trap_idx)),
                            ]);
                            continue;
                        }
                        (LocalSetI32(trap_idx), Local(LSet, set_idx), &[I32], &[])
                        | (LocalSetF32(trap_idx), Local(LSet, set_idx), &[F32], &[])
                        | (LocalSetI64(trap_idx), Local(LSet, set_idx), &[I64], &[])
                        | (LocalSetF64(trap_idx), Local(LSet, set_idx), &[F64], &[]) => {
                            result.extend_from_slice(&[
                                // [value-to-write]
                                typed_instr.instrument_with(
                                    // Push set-index
                                    Instr::Const(Val::I64(i64::from((set_idx).to_u32()))).clone(),
                                ),
                                // [value-to-write, set-index] // Call trap (in value, set-index -> out value)
                                typed_instr.instrument_with(Instr::Call(trap_idx)),
                                // Perform operation
                                typed_instr.place_original(instr.clone()),
                            ]);
                            continue;
                        }
                        (GlobalSetI32(trap_idx), Global(GSet, set_idx), &[I32], &[])
                        | (GlobalSetF32(trap_idx), Global(GSet, set_idx), &[F32], &[])
                        | (GlobalSetI64(trap_idx), Global(GSet, set_idx), &[I64], &[])
                        | (GlobalSetF64(trap_idx), Global(GSet, set_idx), &[F64], &[]) => {
                            result.extend_from_slice(&[
                                // [value-to-write]
                                typed_instr.instrument_with(
                                    // Push set-index
                                    Instr::Const(Val::I64(i64::from((set_idx).to_u32()))).clone(),
                                ),
                                // [value-to-write, set-index] // Call trap (in value, set-index -> out value)
                                typed_instr.instrument_with(Instr::Call(trap_idx)),
                                // Perform operation
                                typed_instr.place_original(instr.clone()),
                            ]);
                            continue;
                        }
                        (LocalTeeI32(trap_idx), Local(Tee, tee_idx), &[I32], &[I32])
                        | (LocalTeeF32(trap_idx), Local(Tee, tee_idx), &[F32], &[F32])
                        | (LocalTeeI64(trap_idx), Local(Tee, tee_idx), &[I64], &[I64])
                        | (LocalTeeF64(trap_idx), Local(Tee, tee_idx), &[F64], &[F64]) => {
                            result.extend_from_slice(&[
                                // [value-to-write]
                                typed_instr.instrument_with(
                                    // Push tee-index
                                    Instr::Const(Val::I64(i64::from((tee_idx).to_u32()))).clone(),
                                ),
                                // [value-to-write, tee-index] // Call trap (in value, tee-index -> out value)
                                typed_instr.instrument_with(Instr::Call(trap_idx)),
                                // Perform operation
                                typed_instr.place_original(instr.clone()),
                            ]);
                            continue;
                        }

                        _ => (), // Skip
                    };
                }
                InferredInstructionType::Unreachable => (), // Skip
            };
        }

        macro_rules! instrument_memory_op {
            (
                store:
                $(
                    ($target:ident, $store_op:ident)
                ),*
            ) => {
                match (target, instr) {
                    $(
                        (Target::$target(trap_idx), Instr::Store($store_op, Memarg { offset, .. })) => {
                            result.extend_from_slice(&[
                                // [i32: index to write to, F32: value to write to] // FIXME: not sure if TOS index or value
                                typed_instr.instrument_with(Instr::Const(Val::I64((*offset).into()))),
                                // [i32: index to write to, F32: value to write to, U32 as I64: Offset]
                                typed_instr.instrument_with(Instr::Const(Val::I32($store_op.serialize()))),
                                // [i32: index to write to, F32: value to write to, U32 as I64: Offset, i32: serialized operation]
                                typed_instr.instrument_with(Instr::Call(trap_idx)),
                            ]);
                            continue;
                        }
                    ),*
                    _ => (),
                }
            };
            (
                load:
                $(
                    ($target:ident, $load_op:ident)
                ),*
            ) => {
                match (target, instr) {
                    $(
                        (Target::$target(trap_idx), Instr::Load($load_op, Memarg { offset, .. })) => {
                            result.extend_from_slice(&[
                                // [i32: index to load from]
                                typed_instr.instrument_with(Instr::Const(Val::I64((*offset).into()))),
                                // [i32: index to load from,  U32as I64: Offset]
                                typed_instr.instrument_with(Instr::Const(Val::I32($load_op.serialize()))),
                                // [i32: index to load from,  U32as I64: Offset, i32: serialized operation]
                                typed_instr.instrument_with(Instr::Call(trap_idx)),
                            ]);
                            continue;
                        }
                    ),*
                    _ => (),
                }
            };

        }
        {
            use StoreOp::{
                F32Store, F64Store, I32Store, I32Store16, I32Store8, I64Store, I64Store16,
                I64Store32, I64Store8,
            };
            instrument_memory_op!(
                store:
                (F32Store, F32Store),
                (F64Store, F64Store),
                (I32Store, I32Store),
                (I32Store, I32Store16),
                (I32Store, I32Store8),
                (I64Store, I64Store),
                (I64Store, I64Store16),
                (I64Store, I64Store32),
                (I64Store, I64Store8)
            );

            use LoadOp::{
                F32Load, F64Load, I32Load, I32Load16S, I32Load16U, I32Load8S, I32Load8U, I64Load,
                I64Load16S, I64Load16U, I64Load32S, I64Load32U, I64Load8S, I64Load8U,
            };
            instrument_memory_op!(
                load:
                (F32Load, F32Load),
                (F64Load, F64Load),
                (I32Load, I32Load),
                (I32Load, I32Load16S),
                (I32Load, I32Load16U),
                (I32Load, I32Load8S),
                (I32Load, I32Load8U),
                (I64Load, I64Load),
                (I64Load, I64Load16S),
                (I64Load, I64Load16U),
                (I64Load, I64Load32S),
                (I64Load, I64Load32U),
                (I64Load, I64Load8S),
                (I64Load, I64Load8U)
            );
        }

        match (target, instr) {
            // DEFAULT TRAVERSAL
            (target, Instr::If(type_, then, None)) => {
                result.push(typed_instr.place_untouched(Instr::If(
                    *type_,
                    transform(then, target),
                    None,
                )));
            }
            (target, Instr::If(type_, then, Some(else_))) => {
                result.push(typed_instr.place_untouched(Instr::If(
                    *type_,
                    transform(then, target),
                    Some(transform(else_, target)),
                )))
            }
            (target, Instr::Loop(type_, body)) => {
                result.push(
                    typed_instr.place_untouched(Instr::Loop(*type_, transform(body, target))),
                );
            }
            (target, Instr::Block(type_, body)) => {
                result.push(
                    typed_instr.place_untouched(Instr::Block(*type_, transform(body, target))),
                );
            }
            (_, instr) => result.push(typed_instr.place_untouched(instr.clone())),
        }
    }
    result
}

trait Serialize {
    fn serialize(&self) -> i32;
}

impl Serialize for StoreOp {
    fn serialize(&self) -> i32 {
        match self {
            StoreOp::I32Store => 1,
            StoreOp::I64Store => 2,
            StoreOp::F32Store => 3,
            StoreOp::F64Store => 4,
            StoreOp::I32Store8 => 5,
            StoreOp::I32Store16 => 6,
            StoreOp::I64Store8 => 7,
            StoreOp::I64Store16 => 8,
            StoreOp::I64Store32 => 9,
        }
    }
}

impl Serialize for LoadOp {
    fn serialize(&self) -> i32 {
        match self {
            LoadOp::I32Load => 1,
            LoadOp::I64Load => 2,
            LoadOp::F32Load => 3,
            LoadOp::F64Load => 4,
            LoadOp::I32Load8S => 5,
            LoadOp::I32Load8U => 6,
            LoadOp::I32Load16S => 7,
            LoadOp::I32Load16U => 8,
            LoadOp::I64Load8S => 9,
            LoadOp::I64Load8U => 10,
            LoadOp::I64Load16S => 11,
            LoadOp::I64Load16U => 12,
            LoadOp::I64Load32S => 13,
            LoadOp::I64Load32U => 14,
        }
    }
}

pub fn inject_memory_loads(module: &mut Module) {
    use wasabi_wasm::Instr::{Binary, End, Load, Local};

    for (name, load_type) in [
        ("instrumented_base_load_i32", ValType::I32),
        ("instrumented_base_load_f32", ValType::F32),
        ("instrumented_base_load_i64", ValType::I64),
        ("instrumented_base_load_f64", ValType::F64),
    ] {
        let load_op = match load_type {
            ValType::I32 => LoadOp::I32Load,
            ValType::I64 => LoadOp::I64Load,
            ValType::F32 => LoadOp::F32Load,
            ValType::F64 => LoadOp::F64Load,
            ValType::Ref(_) => panic!("Unreachable statement"),
        };
        let function_type = FunctionType::new(&[ValType::I32, ValType::I32], &[load_type]);
        let body = vec![
            // []
            Local(LocalOp::Get, 0_u32.into()),
            // [ptr]
            Local(LocalOp::Get, 1_u32.into()),
            // [ptr, offset]
            Binary(wasabi_wasm::BinaryOp::I32Add),
            // [ptr + offset]
            Load(load_op, Memarg::default(load_op)),
            // [value]
            End,
        ];

        let memory_function_idx = module.add_function(function_type, vec![], body);
        module
            .function_mut(memory_function_idx)
            .export
            .push(name.to_string());
    }
}

pub fn inject_memory_stores(module: &mut Module) {
    use wasabi_wasm::Instr::{Binary, End, Local, Store};
    for (name, load_type) in [
        ("instrumented_base_store_i32", ValType::I32),
        ("instrumented_base_store_f32", ValType::F32),
        ("instrumented_base_store_i64", ValType::I64),
        ("instrumented_base_store_f64", ValType::F64),
    ] {
        let store_op = match load_type {
            ValType::I32 => StoreOp::I32Store,
            ValType::I64 => StoreOp::I64Store,
            ValType::F32 => StoreOp::F32Store,
            ValType::F64 => StoreOp::F64Store,
            ValType::Ref(_) => panic!("Unreachable statement"),
        };
        let function_type = FunctionType::new(&[ValType::I32, load_type, ValType::I32], &[]);
        let body = vec![
            // []
            Local(LocalOp::Get, 0_u32.into()),
            // [ptr]
            Local(LocalOp::Get, 2_u32.into()),
            // [ptr, offset]
            Binary(wasabi_wasm::BinaryOp::I32Add),
            // [ptr + offset]
            Local(LocalOp::Get, 1_u32.into()),
            // [ptr + offset, value]
            Store(store_op, Memarg::default(store_op)),
            // []
            End,
        ];

        let memory_function_idx: Idx<Function> = module.add_function(function_type, vec![], body);
        module
            .function_mut(memory_function_idx)
            .export
            .push(name.to_string());
    }
}

pub fn inject_memory_grow(module: &mut Module) {
    use wasabi_wasm::Instr::{End, Local, MemoryGrow};

    let function_type = FunctionType::new(&[ValType::I32, ValType::I32], &[ValType::I32]);
    let body = vec![
        // []
        Local(LocalOp::Get, 0_u32.into()),
        // [amount:i32]
        MemoryGrow(0_u32.into()),
        // [delta_or_neg_1:i32]
        End,
    ];

    let memory_function_idx: Idx<Function> = module.add_function(function_type, vec![], body);
    module
        .function_mut(memory_function_idx)
        .export
        .push("instrumented_memory_grow".to_string());
}
