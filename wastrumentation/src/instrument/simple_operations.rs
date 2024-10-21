use crate::parse_nesting::{
    BodyInner, HighLevelBody, HighLevelInstr as Instr, TypedHighLevelInstr,
};
use wasabi_wasm::{BinaryOp, Function, Idx, Module, UnaryOp, Val};

use super::TransformationStrategy;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Target {
    Drop(Idx<Function>), // Optional: drop-generic / drop-specific

    Return(Idx<Function>),

    ConstI32(Idx<Function>),
    ConstF32(Idx<Function>),
    ConstI64(Idx<Function>),
    ConstF64(Idx<Function>),

    UnaryI32ToI32(Idx<Function>),
    UnaryI64ToI32(Idx<Function>),
    UnaryI64ToI64(Idx<Function>),
    UnaryF32ToF32(Idx<Function>),
    UnaryF64ToF64(Idx<Function>),
    UnaryF32ToI32(Idx<Function>),
    UnaryF64ToI32(Idx<Function>),
    UnaryI32ToI64(Idx<Function>),
    UnaryF32ToI64(Idx<Function>),
    UnaryF64ToI64(Idx<Function>),
    UnaryI32ToF32(Idx<Function>),
    UnaryI64ToF32(Idx<Function>),
    UnaryF64ToF32(Idx<Function>),
    UnaryI32ToF64(Idx<Function>),
    UnaryI64ToF64(Idx<Function>),
    UnaryF32ToF64(Idx<Function>),

    BinaryI32I32toI32(Idx<Function>),
    BinaryI64I64toI32(Idx<Function>),
    BinaryF32F32toI32(Idx<Function>),
    BinaryF64F64toI32(Idx<Function>),
    BinaryI64I64toI64(Idx<Function>),
    BinaryF32F32toF32(Idx<Function>),
    BinaryF64F64toF64(Idx<Function>),
}

impl TransformationStrategy for Target {
    fn transform(&self, high_level_body: &HighLevelBody, _: &mut Module) -> HighLevelBody {
        let HighLevelBody(body) = high_level_body;
        let transformed_body = transform(body, *self);
        HighLevelBody(transformed_body)
    }
}

trait WastrumentationSerializable {
    fn serialize(&self) -> i32;
}

impl WastrumentationSerializable for UnaryOp {
    fn serialize(&self) -> i32 {
        match self {
            UnaryOp::I32Eqz  /* . */   => 1,
            UnaryOp::I64Eqz            => 2,
            UnaryOp::I32Clz            => 3,
            UnaryOp::I32Ctz            => 4,
            UnaryOp::I32Popcnt         => 5,
            UnaryOp::I64Clz            => 6,
            UnaryOp::I64Ctz            => 7,
            UnaryOp::I64Popcnt         => 8,
            UnaryOp::F32Abs            => 9,
            UnaryOp::F32Neg            => 10,
            UnaryOp::F32Ceil           => 11,
            UnaryOp::F32Floor          => 12,
            UnaryOp::F32Trunc          => 13,
            UnaryOp::F32Nearest        => 14,
            UnaryOp::F32Sqrt           => 15,
            UnaryOp::F64Abs            => 16,
            UnaryOp::F64Neg            => 17,
            UnaryOp::F64Ceil           => 18,
            UnaryOp::F64Floor          => 19,
            UnaryOp::F64Trunc          => 20,
            UnaryOp::F64Nearest        => 21,
            UnaryOp::F64Sqrt           => 22,
            UnaryOp::I32WrapI64        => 23,
            UnaryOp::I32TruncF32S      => 24,
            UnaryOp::I32TruncF32U      => 25,
            UnaryOp::I32TruncF64S      => 26,
            UnaryOp::I32TruncF64U      => 27,
            UnaryOp::I32TruncSatF32S   => 28,
            UnaryOp::I32TruncSatF32U   => 29,
            UnaryOp::I32TruncSatF64S   => 30,
            UnaryOp::I32TruncSatF64U   => 31,
            UnaryOp::I64ExtendI32S     => 32,
            UnaryOp::I64ExtendI32U     => 33,
            UnaryOp::I64TruncF32S      => 34,
            UnaryOp::I64TruncF32U      => 35,
            UnaryOp::I64TruncF64S      => 36,
            UnaryOp::I64TruncF64U      => 37,
            UnaryOp::I64TruncSatF32S   => 38,
            UnaryOp::I64TruncSatF32U   => 39,
            UnaryOp::I64TruncSatF64S   => 40,
            UnaryOp::I64TruncSatF64U   => 41,
            UnaryOp::F32ConvertI32S    => 42,
            UnaryOp::F32ConvertI32U    => 43,
            UnaryOp::F32ConvertI64S    => 44,
            UnaryOp::F32ConvertI64U    => 45,
            UnaryOp::F32DemoteF64      => 46,
            UnaryOp::F64ConvertI32S    => 47,
            UnaryOp::F64ConvertI32U    => 48,
            UnaryOp::F64ConvertI64S    => 49,
            UnaryOp::F64ConvertI64U    => 50,
            UnaryOp::F64PromoteF32     => 51,
            UnaryOp::I32ReinterpretF32 => 52,
            UnaryOp::I64ReinterpretF64 => 53,
            UnaryOp::F32ReinterpretI32 => 54,
            UnaryOp::F64ReinterpretI64 => 55,
            UnaryOp::I32Extend8S       => 56,
            UnaryOp::I32Extend16S      => 57,
            UnaryOp::I64Extend8S       => 58,
            UnaryOp::I64Extend16S      => 59,
            UnaryOp::I64Extend32S      => 60,
        }
    }
}

impl WastrumentationSerializable for BinaryOp {
    fn serialize(&self) -> i32 {
        match self {
            BinaryOp::I32Eq /* . */ => 1,
            BinaryOp::I32Ne         => 2,
            BinaryOp::I32LtS        => 3,
            BinaryOp::I32LtU        => 4,
            BinaryOp::I32GtS        => 5,
            BinaryOp::I32GtU        => 6,
            BinaryOp::I32LeS        => 7,
            BinaryOp::I32LeU        => 8,
            BinaryOp::I32GeS        => 9,
            BinaryOp::I32GeU        => 10,
            BinaryOp::I64Eq         => 11,
            BinaryOp::I64Ne         => 12,
            BinaryOp::I64LtS        => 13,
            BinaryOp::I64LtU        => 14,
            BinaryOp::I64GtS        => 15,
            BinaryOp::I64GtU        => 16,
            BinaryOp::I64LeS        => 17,
            BinaryOp::I64LeU        => 18,
            BinaryOp::I64GeS        => 19,
            BinaryOp::I64GeU        => 20,
            BinaryOp::F32Eq         => 21,
            BinaryOp::F32Ne         => 22,
            BinaryOp::F32Lt         => 23,
            BinaryOp::F32Gt         => 24,
            BinaryOp::F32Le         => 25,
            BinaryOp::F32Ge         => 26,
            BinaryOp::F64Eq         => 27,
            BinaryOp::F64Ne         => 28,
            BinaryOp::F64Lt         => 29,
            BinaryOp::F64Gt         => 30,
            BinaryOp::F64Le         => 31,
            BinaryOp::F64Ge         => 32,
            BinaryOp::I32Add        => 33,
            BinaryOp::I32Sub        => 34,
            BinaryOp::I32Mul        => 35,
            BinaryOp::I32DivS       => 36,
            BinaryOp::I32DivU       => 37,
            BinaryOp::I32RemS       => 38,
            BinaryOp::I32RemU       => 39,
            BinaryOp::I32And        => 40,
            BinaryOp::I32Or         => 41,
            BinaryOp::I32Xor        => 42,
            BinaryOp::I32Shl        => 43,
            BinaryOp::I32ShrS       => 44,
            BinaryOp::I32ShrU       => 45,
            BinaryOp::I32Rotl       => 46,
            BinaryOp::I32Rotr       => 47,
            BinaryOp::I64Add        => 48,
            BinaryOp::I64Sub        => 49,
            BinaryOp::I64Mul        => 50,
            BinaryOp::I64DivS       => 51,
            BinaryOp::I64DivU       => 52,
            BinaryOp::I64RemS       => 53,
            BinaryOp::I64RemU       => 54,
            BinaryOp::I64And        => 55,
            BinaryOp::I64Or         => 56,
            BinaryOp::I64Xor        => 57,
            BinaryOp::I64Shl        => 58,
            BinaryOp::I64ShrS       => 59,
            BinaryOp::I64ShrU       => 60,
            BinaryOp::I64Rotl       => 61,
            BinaryOp::I64Rotr       => 62,
            BinaryOp::F32Add        => 63,
            BinaryOp::F32Sub        => 64,
            BinaryOp::F32Mul        => 65,
            BinaryOp::F32Div        => 66,
            BinaryOp::F32Min        => 67,
            BinaryOp::F32Max        => 68,
            BinaryOp::F32Copysign   => 69,
            BinaryOp::F64Add        => 70,
            BinaryOp::F64Sub        => 71,
            BinaryOp::F64Mul        => 72,
            BinaryOp::F64Div        => 73,
            BinaryOp::F64Min        => 74,
            BinaryOp::F64Max        => 75,
            BinaryOp::F64Copysign   => 76,
        }
    }
}

macro_rules! transformation_strategy {
    (
        $typed_instr: ident, $loop_target:ident, $loop_instr:ident, $total_result:ident,
        $( $target:ident for $instr:ident instr $enum:ident::{ $( $variant:ident ) | * $(|)? } )*
    ) => {
        match ($loop_target, $loop_instr) {
            // GENERATED TRAVERSAL
            $(
                (Target::$target(trap_idx), Instr::$instr(op)) if matches!(op, $($enum::$variant) | *) => {
                    $total_result.extend_from_slice(&[
                        // Push serialized operator
                        $typed_instr.instrument_with(Instr::Const(Val::I32(op.serialize()))),
                    ]);
                    $total_result.extend_from_slice(&$typed_instr.to_trap_call(&trap_idx));
                    continue;
                }
            )*,
            _ => ()
        }
    };

    (
        $typed_instr: ident, $loop_target:ident, $loop_instr:ident, $total_result:ident,
        $( $target:ident for $instr:pat_param )*
    ) => {
        match ($loop_target, $loop_instr) {
            // GENERATED TRAVERSAL
            $(
                (Target::$target(trap_idx), $instr) => {
                    $total_result.extend_from_slice(&[
                        // Inject original instruction to push constant
                        $typed_instr.place_original($loop_instr.clone()),
                    ]);
                    $total_result.extend_from_slice(&$typed_instr.to_trap_call(&trap_idx));
                    continue;
                }
            )*,
            _ => ()
        }
    };
}

fn transform(body: &BodyInner, target: Target) -> BodyInner {
    let mut result = Vec::new();

    for typed_instr @ TypedHighLevelInstr { instr, .. } in body {
        if typed_instr.is_uninstrumented() {
            if let (Target::Return(trap_idx), Instr::Return) = (target, instr) {
                // Inject call
                result.extend_from_slice(&typed_instr.to_trap_call(&trap_idx));
                result.extend_from_slice(&[
                    // Inject original instruction after
                    typed_instr.place_original(instr.clone()),
                ]);
                continue;
            }

            if let (Target::Drop(trap_idx), Instr::Drop) = (target, instr) {
                // Inject call
                result.extend_from_slice(&typed_instr.to_trap_call(&trap_idx));
                result.extend_from_slice(&[
                    // Inject original instruction after
                    typed_instr.place_original(instr.clone()),
                ]);
                continue;
            }

            transformation_strategy!(
                typed_instr, target, instr, result,
                ConstI32 for Instr::Const(Val::I32(_))
                ConstF32 for Instr::Const(Val::F32(_))
                ConstI64 for Instr::Const(Val::I64(_))
                ConstF64 for Instr::Const(Val::F64(_))
            );

            transformation_strategy! {
                typed_instr, target, instr, result,
                // Unary
                UnaryI32ToI32 for Unary instr UnaryOp::{I32Eqz}
                UnaryI64ToI32 for Unary instr UnaryOp::{I64Eqz}
                UnaryI32ToI32 for Unary instr UnaryOp::{I32Clz | I32Ctz | I32Popcnt}
                UnaryI64ToI64 for Unary instr UnaryOp::{I64Clz | I64Ctz | I64Popcnt}

                UnaryF32ToF32 for Unary instr UnaryOp::{F32Abs | F32Neg | F32Ceil | F32Floor | F32Trunc | F32Nearest | F32Sqrt}
                UnaryF64ToF64 for Unary instr UnaryOp::{F64Abs | F64Neg | F64Ceil | F64Floor | F64Trunc | F64Nearest | F64Sqrt}

                UnaryI64ToI32 for Unary instr UnaryOp::{I32WrapI64}
                UnaryF32ToI32 for Unary instr UnaryOp::{I32TruncF32S | I32TruncF32U | I32TruncSatF32S | I32TruncSatF32U}
                UnaryF64ToI32 for Unary instr UnaryOp::{I32TruncF64S | I32TruncF64U | I32TruncSatF64S | I32TruncSatF64U}
                UnaryI32ToI64 for Unary instr UnaryOp::{I64ExtendI32S | I64ExtendI32U}
                UnaryF32ToI64 for Unary instr UnaryOp::{I64TruncF32S | I64TruncF32U | I64TruncSatF32S | I64TruncSatF32U}
                UnaryF64ToI64 for Unary instr UnaryOp::{I64TruncF64S | I64TruncF64U | I64TruncSatF64S | I64TruncSatF64U}
                UnaryI32ToF32 for Unary instr UnaryOp::{F32ConvertI32S | F32ConvertI32U}
                UnaryI64ToF32 for Unary instr UnaryOp::{F32ConvertI64S | F32ConvertI64U}
                UnaryF64ToF32 for Unary instr UnaryOp::{F32DemoteF64}
                UnaryI32ToF64 for Unary instr UnaryOp::{F64ConvertI32S | F64ConvertI32U}
                UnaryI64ToF64 for Unary instr UnaryOp::{F64ConvertI64S | F64ConvertI64U}
                UnaryF32ToF64 for Unary instr UnaryOp::{F64PromoteF32}
                UnaryF32ToI32 for Unary instr UnaryOp::{I32ReinterpretF32}
                UnaryF64ToI64 for Unary instr UnaryOp::{I64ReinterpretF64}
                UnaryI32ToF32 for Unary instr UnaryOp::{F32ReinterpretI32}
                UnaryI64ToF64 for Unary instr UnaryOp::{F64ReinterpretI64}
                UnaryI32ToI32 for Unary instr UnaryOp::{I32Extend8S | I32Extend16S }
                UnaryI64ToI64 for Unary instr UnaryOp::{I64Extend8S | I64Extend16S | I64Extend32S }

                // Binary
                BinaryI32I32toI32 for Binary instr BinaryOp::{I32Eq | I32Ne | I32LtS | I32LtU | I32GtS | I32GtU | I32LeS | I32LeU | I32GeS | I32GeU}
                BinaryI64I64toI32 for Binary instr BinaryOp::{I64Eq | I64Ne | I64LtS | I64LtU | I64GtS | I64GtU | I64LeS | I64LeU | I64GeS | I64GeU}

                BinaryF32F32toI32 for Binary instr BinaryOp::{F32Eq | F32Ne | F32Lt | F32Gt | F32Le | F32Ge}
                BinaryF64F64toI32 for Binary instr BinaryOp::{F64Eq | F64Ne | F64Lt | F64Gt | F64Le | F64Ge}

                BinaryI32I32toI32 for Binary instr BinaryOp::{I32Add | I32Sub | I32Mul | I32DivS | I32DivU | I32RemS | I32RemU | I32And | I32Or | I32Xor | I32Shl | I32ShrS | I32ShrU | I32Rotl | I32Rotr}
                BinaryI64I64toI64 for Binary instr BinaryOp::{I64Add | I64Sub | I64Mul | I64DivS | I64DivU | I64RemS | I64RemU | I64And | I64Or | I64Xor | I64Shl | I64ShrS | I64ShrU | I64Rotl | I64Rotr}
                BinaryF32F32toF32 for Binary instr BinaryOp::{F32Add | F32Sub | F32Mul | F32Div | F32Min | F32Max | F32Copysign}
                BinaryF64F64toF64 for Binary instr BinaryOp::{F64Add | F64Sub | F64Mul | F64Div | F64Min | F64Max | F64Copysign}
            }
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
