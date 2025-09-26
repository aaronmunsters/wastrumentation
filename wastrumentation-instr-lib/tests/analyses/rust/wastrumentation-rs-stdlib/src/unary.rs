#[cfg(target_arch = "wasm32")]
use core::arch::asm;

use crate::WasmValue;
use cfg_if::cfg_if;

macro_rules! generate_unary_boilerplate {
    (
        $(
            $serialized:literal
            $operator:ident
            : $in_type:ident -> $wasm_asm_instruction:literal -> $wasm_res_out_type:ident
            REIFIED:
            $apply_body:expr
        ),*
        $(,)?

    ) => {
        #[derive(Debug, Clone, Copy)]
        pub enum UnaryOperator {
            $(
                $operator
            ),*
        }

        impl UnaryOperator {
            pub fn apply(&self, operand: WasmValue) -> WasmValue {
                match self {
                    $(
                        Self::$operator => {
                            let operand: $in_type = generate_unary_boilerplate!(@internally WasmValue-to-primitive operand $in_type);
                            cfg_if! {
                                if #[cfg(target_arch = "wasm32")] {
                                    let result: $wasm_res_out_type;
                                    unsafe {
                                        asm! {
                                            "local.get {operand}",
                                            $wasm_asm_instruction,
                                            "local.set {result}",
                                            operand = in(local) operand,
                                            result = out(local) result,
                                        };
                                    }
                                    generate_unary_boilerplate!(@internally primitive-to-WasmValue result $wasm_res_out_type)
                                } else {
                                    /* Using alternative attempt */
                                    let result: $wasm_res_out_type = $apply_body(operand);
                                    generate_unary_boilerplate!(@internally primitive-to-WasmValue result $wasm_res_out_type)

                                }
                            }
                        }
                    )*
                }
            }
        }

        impl From<i32> for UnaryOperator {
            fn from(value: i32) -> Self {
                match value {
                    $(
                        $serialized => UnaryOperator::$operator,
                    )*
                    v => panic!("Unknown serialized unary operator value: {v}"),
                }
            }
        }
    };

    (@internally WasmValue-to-primitive $wasm_value:ident i32) => { $wasm_value.as_i32() };
    (@internally WasmValue-to-primitive $wasm_value:ident f32) => { $wasm_value.as_f32() };
    (@internally WasmValue-to-primitive $wasm_value:ident i64) => { $wasm_value.as_i64() };
    (@internally WasmValue-to-primitive $wasm_value:ident f64) => { $wasm_value.as_f64() };

    (@internally primitive-to-WasmValue $value:ident i32) => { WasmValue::I32($value) };
    (@internally primitive-to-WasmValue $value:ident f32) => { WasmValue::F32($value) };
    (@internally primitive-to-WasmValue $value:ident i64) => { WasmValue::I64($value) };
    (@internally primitive-to-WasmValue $value:ident f64) => { WasmValue::F64($value) };
}

generate_unary_boilerplate!(
// IDX VARIANT           CONVERSION IMPLEMENTATION
     1 I32Eqz            : i32 -> "i32.eqz"             -> i32 REIFIED: |o: i32|                    (o == 0)                                             as i32,
     2 I64Eqz            : i64 -> "i64.eqz"             -> i32 REIFIED: |o: i64|                    (o == 0)                                             as i32,
     3 I32Clz            : i32 -> "i32.clz"             -> i32 REIFIED: |o: i32| i32:: leading_zeros(o)                                                  as i32,
     4 I32Ctz            : i32 -> "i32.ctz"             -> i32 REIFIED: |o: i32| i32::trailing_zeros(o)                                                  as i32,
     5 I32Popcnt         : i32 -> "i32.popcnt"          -> i32 REIFIED: |o: i32| i32::    count_ones(o)                                                  as i32,
     6 I64Clz            : i64 -> "i64.clz"             -> i64 REIFIED: |o: i64| i64:: leading_zeros(o)                                                  as i64,
     7 I64Ctz            : i64 -> "i64.ctz"             -> i64 REIFIED: |o: i64| i64::trailing_zeros(o)                                                  as i64,
     8 I64Popcnt         : i64 -> "i64.popcnt"          -> i64 REIFIED: |o: i64| i64::    count_ones(o)                                                  as i64,
     9 F32Abs            : f32 -> "f32.abs"             -> f32 REIFIED: |o: f32| libm::        fabsf(o)                                                        ,
    10 F32Neg            : f32 -> "f32.neg"             -> f32 REIFIED: |o: f32|                    -o                                                         ,
    11 F32Ceil           : f32 -> "f32.ceil"            -> f32 REIFIED: |o: f32|        libm:: ceilf(o)                                                        ,
    12 F32Floor          : f32 -> "f32.floor"           -> f32 REIFIED: |o: f32|        libm::floorf(o)                                                        ,
    13 F32Trunc          : f32 -> "f32.trunc"           -> f32 REIFIED: |o: f32|        libm::truncf(o)                                                        ,
    14 F32Nearest        : f32 -> "f32.nearest"         -> f32 REIFIED: |o: f32|        libm::roundf(o)                                                        ,
    15 F32Sqrt           : f32 -> "f32.sqrt"            -> f32 REIFIED: |o: f32|        libm:: sqrtf(o)                                                        ,
    16 F64Abs            : f64 -> "f64.abs"             -> f64 REIFIED: |o: f64|        libm::  fabs(o)                                                        ,
    17 F64Neg            : f64 -> "f64.neg"             -> f64 REIFIED: |o: f64|                    -o                                                         ,
    18 F64Ceil           : f64 -> "f64.ceil"            -> f64 REIFIED: |o: f64|         libm:: ceil(o)                                                        ,
    19 F64Floor          : f64 -> "f64.floor"           -> f64 REIFIED: |o: f64|         libm::floor(o)                                                        ,
    20 F64Trunc          : f64 -> "f64.trunc"           -> f64 REIFIED: |o: f64|         libm::trunc(o)                                                        ,
    21 F64Nearest        : f64 -> "f64.nearest"         -> f64 REIFIED: |o: f64|         libm::round(o)                                                        ,
    22 F64Sqrt           : f64 -> "f64.sqrt"            -> f64 REIFIED: |o: f64|         libm:: sqrt(o)                                                        ,
    23 I32WrapI64        : i64 -> "i32.wrap_i64"        -> i32 REIFIED: |o: i64|                     o                                                   as i32,
    24 I32TruncF32S      : f32 -> "i32.trunc_f32_s"     -> i32 REIFIED: |o: f32|        libm::truncf(o)                                                  as i32,
    25 I32TruncF32U      : f32 -> "i32.trunc_f32_u"     -> i32 REIFIED: |o: f32|       (libm::truncf(o)                                          as u32) as i32,
    26 I32TruncF64S      : f64 -> "i32.trunc_f64_s"     -> i32 REIFIED: |o: f64|        libm:: trunc(o)                                                  as i32,
    27 I32TruncF64U      : f64 -> "i32.trunc_f64_u"     -> i32 REIFIED: |o: f64|       (libm:: trunc(o)                                          as u32) as i32,
    28 I32TruncSatF32S   : f32 -> "i32.trunc_sat_f32_s" -> i32 REIFIED: |o: f32|        libm::truncf(o) .clamp(i32::MIN as f32, i32::MAX as f32)         as i32,
    29 I32TruncSatF32U   : f32 -> "i32.trunc_sat_f32_u" -> i32 REIFIED: |o: f32|        libm::truncf(o) .clamp(            0.0, u32::MAX as f32) as u32  as i32,
    30 I32TruncSatF64S   : f64 -> "i32.trunc_sat_f64_s" -> i32 REIFIED: |o: f64|        libm:: trunc(o) .clamp(i32::MIN as f64, i32::MAX as f64)         as i32,
    31 I32TruncSatF64U   : f64 -> "i32.trunc_sat_f64_u" -> i32 REIFIED: |o: f64|       (libm:: trunc(o) .clamp(            0.0, u32::MAX as f64) as u32) as i32,
    32 I64ExtendI32S     : i32 -> "i64.extend_i32_s"    -> i64 REIFIED: |o: i32|                    (o /*as i32*/)                                       as i64,
    33 I64ExtendI32U     : i32 -> "i64.extend_i32_u"    -> i64 REIFIED: |o: i32|                    (o   as u32  )                                       as i64,
    34 I64TruncF32S      : f32 -> "i64.trunc_f32_s"     -> i64 REIFIED: |o: f32|        libm::truncf(o)                                                  as i64,
    35 I64TruncF32U      : f32 -> "i64.trunc_f32_u"     -> i64 REIFIED: |o: f32|       (libm::truncf(o) as u64)                                          as i64,
    36 I64TruncF64S      : f64 -> "i64.trunc_f64_s"     -> i64 REIFIED: |o: f64|        libm:: trunc(o)                                                  as i64,
    37 I64TruncF64U      : f64 -> "i64.trunc_f64_u"     -> i64 REIFIED: |o: f64|       (libm:: trunc(o) as u64)                                          as i64,
    38 I64TruncSatF32S   : f32 -> "i64.trunc_sat_f32_s" -> i64 REIFIED: |o: f32|        libm::truncf(o).clamp(f32::MIN, f32::MAX)                        as i64,
    39 I64TruncSatF32U   : f32 -> "i64.trunc_sat_f32_u" -> i64 REIFIED: |o: f32|       (libm::truncf(o).clamp(0.0, u64::MAX as f32) as u64)              as i64,
    40 I64TruncSatF64S   : f64 -> "i64.trunc_sat_f64_s" -> i64 REIFIED: |o: f64|        libm:: trunc(o).clamp(f64::MIN, f64::MAX)                        as i64,
    41 I64TruncSatF64U   : f64 -> "i64.trunc_sat_f64_u" -> i64 REIFIED: |o: f64|       (libm:: trunc(o).clamp(0.0, u64::MAX as f64) as u64)              as i64,
    42 F32ConvertI32S    : i32 -> "f32.convert_i32_s"   -> f32 REIFIED: |o: i32|                     o                                                   as f32,
    43 F32ConvertI32U    : i32 -> "f32.convert_i32_u"   -> f32 REIFIED: |o: i32|                    (o as u32)                                           as f32,
    44 F32ConvertI64S    : i64 -> "f32.convert_i64_s"   -> f32 REIFIED: |o: i64|                     o                                                   as f32,
    45 F32ConvertI64U    : i64 -> "f32.convert_i64_u"   -> f32 REIFIED: |o: i64|                    (o as u64)                                           as f32,
    46 F32DemoteF64      : f64 -> "f32.demote_f64"      -> f32 REIFIED: |o: f64|                     o                                                   as f32,
    47 F64ConvertI32S    : i32 -> "f64.convert_i32_s"   -> f64 REIFIED: |o: i32|                     o                                                   as f64,
    48 F64ConvertI32U    : i32 -> "f64.convert_i32_u"   -> f64 REIFIED: |o: i32|                    (o as u32)                                           as f64,
    49 F64ConvertI64S    : i64 -> "f64.convert_i64_s"   -> f64 REIFIED: |o: i64|                     o                                                   as f64,
    50 F64ConvertI64U    : i64 -> "f64.convert_i64_u"   -> f64 REIFIED: |o: i64|                    (o as u64)                                           as f64,
    51 F64PromoteF32     : f32 -> "f64.promote_f32"     -> f64 REIFIED: |o: f32|                     o                                                   as f64,
    52 I32ReinterpretF32 : f32 -> "i32.reinterpret_f32" -> i32 REIFIED: |o: f32|  i32::from_ne_bytes(o.to_ne_bytes())                                          ,
    53 I64ReinterpretF64 : f64 -> "i64.reinterpret_f64" -> i64 REIFIED: |o: f64|  i64::from_ne_bytes(o.to_ne_bytes())                                          ,
    54 F32ReinterpretI32 : i32 -> "f32.reinterpret_i32" -> f32 REIFIED: |o: i32|  f32::from_le_bytes(o.to_le_bytes())                                          ,
    55 F64ReinterpretI64 : i64 -> "f64.reinterpret_i64" -> f64 REIFIED: |o: i64|  f64::from_le_bytes(o.to_le_bytes())                                          ,
    56 I32Extend8S       : i32 -> "i32.extend8_s"       -> i32 REIFIED: |o: i32|                    (o as  i8)                                           as i32,
    57 I32Extend16S      : i32 -> "i32.extend16_s"      -> i32 REIFIED: |o: i32|                    (o as i16)                                           as i32,
    58 I64Extend8S       : i64 -> "i64.extend8_s"       -> i64 REIFIED: |o: i64|                    (o as  i8)                                           as i64,
    59 I64Extend16S      : i64 -> "i64.extend16_s"      -> i64 REIFIED: |o: i64|                    (o as i16)                                           as i64,
    60 I64Extend32S      : i64 -> "i64.extend32_s"      -> i64 REIFIED: |o: i64|                    (o as i32)                                           as i64,
);
