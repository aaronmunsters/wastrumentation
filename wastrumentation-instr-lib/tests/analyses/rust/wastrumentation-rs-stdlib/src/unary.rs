#[cfg(target_arch = "wasm32")]
use core::arch::asm;

use crate::WasmValue;
use cfg_if::cfg_if;

macro_rules! generate_unary_boilerplate {
    (
        $(
            $serialized:literal $operator:ident $apply_body:expr
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
                        Self::$operator => $apply_body(operand),
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
}

generate_unary_boilerplate!(
// IDX VARIANT           CONVERSION IMPLEMENTATION
     1 I32Eqz            |o: WasmValue| WasmValue::i32_from_bool(o.as_i32() == 0),
     2 I64Eqz            |o: WasmValue| WasmValue::i32_from_bool(o.as_i64() == 0),
     3 I32Clz            |o: WasmValue| WasmValue::I32(i32::leading_zeros(o.as_i32()) as i32),
     4 I32Ctz            |o: WasmValue| WasmValue::I32(i32::trailing_zeros(o.as_i32()) as i32),
     5 I32Popcnt         |o: WasmValue| WasmValue::I32(i32::count_ones(o.as_i32()) as i32),
     6 I64Clz            |o: WasmValue| WasmValue::I64(i64::leading_zeros(o.as_i64()) as i64),
     7 I64Ctz            |o: WasmValue| WasmValue::I64(i64::trailing_zeros(o.as_i64()) as i64),
     8 I64Popcnt         |o: WasmValue| WasmValue::I64(i64::count_ones(o.as_i64()) as i64),
     9 F32Abs            |o: WasmValue| WasmValue::F32(libm::fabsf(o.as_f32())),
    10 F32Neg            |o: WasmValue| WasmValue::F32(-o.as_f32()),
    11 F32Ceil           |o: WasmValue| { let mut operand = o.as_f32(); cfg_if! { if #[cfg(target_arch = "wasm32")] { unsafe { asm! { "local.get {operand}", "f32.ceil"   , "local.set {operand}", operand = inout(local) operand }; }; } else { /* Return the rounded value using libm */ operand = libm::ceilf(operand) ; } }; WasmValue::F32(operand) },
    12 F32Floor          |o: WasmValue| { let mut operand = o.as_f32(); cfg_if! { if #[cfg(target_arch = "wasm32")] { unsafe { asm! { "local.get {operand}", "f32.floor"  , "local.set {operand}", operand = inout(local) operand }; }; } else { /* Return the rounded value using libm */ operand = libm::floorf(operand); } }; WasmValue::F32(operand) },
    13 F32Trunc          |o: WasmValue| { let mut operand = o.as_f32(); cfg_if! { if #[cfg(target_arch = "wasm32")] { unsafe { asm! { "local.get {operand}", "f32.trunc"  , "local.set {operand}", operand = inout(local) operand }; }; } else { /* Return the rounded value using libm */ operand = libm::truncf(operand); } }; WasmValue::F32(operand) },
    14 F32Nearest        |o: WasmValue| { let mut operand = o.as_f32(); cfg_if! { if #[cfg(target_arch = "wasm32")] { unsafe { asm! { "local.get {operand}", "f32.nearest", "local.set {operand}", operand = inout(local) operand }; }; } else { /* Return the rounded value using libm */ operand = libm::roundf(operand); } }; WasmValue::F32(operand) },
    15 F32Sqrt           |o: WasmValue| WasmValue::F32(libm::sqrtf(o.as_f32())),
    16 F64Abs            |o: WasmValue| WasmValue::F64(libm::fabs(o.as_f64())),
    17 F64Neg            |o: WasmValue| WasmValue::F64(-o.as_f64()),
    18 F64Ceil           |o: WasmValue| { let mut operand = o.as_f64(); cfg_if! { if #[cfg(target_arch = "wasm32")] { unsafe { asm! { "local.get {operand}", "f64.ceil"   , "local.set {operand}", operand = inout(local) operand }; }; } else { /* Return the rounded value using libm */ operand = libm::ceil(operand)  ; } }; WasmValue::F64(operand) },
    19 F64Floor          |o: WasmValue| { let mut operand = o.as_f64(); cfg_if! { if #[cfg(target_arch = "wasm32")] { unsafe { asm! { "local.get {operand}", "f64.floor"  , "local.set {operand}", operand = inout(local) operand }; }; } else { /* Return the rounded value using libm */ operand = libm::floor(operand) ; } }; WasmValue::F64(operand) },
    20 F64Trunc          |o: WasmValue| { let mut operand = o.as_f64(); cfg_if! { if #[cfg(target_arch = "wasm32")] { unsafe { asm! { "local.get {operand}", "f64.trunc"  , "local.set {operand}", operand = inout(local) operand }; }; } else { /* Return the rounded value using libm */ operand = libm::trunc(operand) ; } }; WasmValue::F64(operand) },
    21 F64Nearest        |o: WasmValue| { let mut operand = o.as_f64(); cfg_if! { if #[cfg(target_arch = "wasm32")] { unsafe { asm! { "local.get {operand}", "f64.nearest", "local.set {operand}", operand = inout(local) operand }; }; } else { /* Return the rounded value using libm */ operand = libm::round(operand) ; } }; WasmValue::F64(operand) },
    22 F64Sqrt           |o: WasmValue| WasmValue::F64(libm::sqrt(o.as_f64())),
    23 I32WrapI64        |o: WasmValue| WasmValue::I32(o.as_i64() as i32),
    24 I32TruncF32S      |o: WasmValue| WasmValue::I32(libm::truncf(o.as_f32()) as i32),
    25 I32TruncF32U      |o: WasmValue| WasmValue::I32((libm::truncf(o.as_f32()) as u32) as i32),
    26 I32TruncF64S      |o: WasmValue| WasmValue::I32(libm::trunc(o.as_f64()) as i32),
    27 I32TruncF64U      |o: WasmValue| WasmValue::I32((libm::trunc(o.as_f64()) as u32) as i32),
    28 I32TruncSatF32S   |o: WasmValue| { let value = o.as_f32(); if value.is_nan() { WasmValue::I32(0) } else { WasmValue::I32(libm::truncf(value).clamp(i32::MIN as f32, i32::MAX as f32) as i32) } },
    29 I32TruncSatF32U   |o: WasmValue| { let value = o.as_f32(); if value.is_nan() { WasmValue::I32(0) } else { let truncated = libm::truncf(value); let clamped = truncated.clamp(0.0, u32::MAX as f32); /* Clamp between 0 and u32::MAX */ WasmValue::I32(clamped as u32 as i32) /* Cast to u32 and then to i32 */ } },
    30 I32TruncSatF64S   |o: WasmValue| { let value = o.as_f64(); if value.is_nan() { WasmValue::I32(0) } else { let truncated = libm::trunc(value); let clamped = truncated.clamp(i32::MIN as f64, i32::MAX as f64); /* Clamp between i32::MIN and i32::MAX */ WasmValue::I32(clamped as i32) /* Cast to i32 */ } },
    31 I32TruncSatF64U   |o: WasmValue| { let value = o.as_f64(); if value.is_nan() { WasmValue::I32(0) } else { let truncated = libm::trunc(value); let clamped = truncated.clamp(0.0, u32::MAX as f64); /* Clamp between 0 and u32::MAX */ WasmValue::I32(clamped as u32 as i32) /* Cast to u32 then to i32 */ } },
    32 I64ExtendI32S     |o: WasmValue| WasmValue::I64((o.as_i32() as i32) as i64),
    33 I64ExtendI32U     |o: WasmValue| WasmValue::I64((o.as_i32() as u32) as i64),
    34 I64TruncF32S      |o: WasmValue| WasmValue::I64(libm::truncf(o.as_f32()) as i64),
    35 I64TruncF32U      |o: WasmValue| WasmValue::I64((libm::truncf(o.as_f32()) as u64) as i64),
    36 I64TruncF64S      |o: WasmValue| WasmValue::I64(libm::trunc(o.as_f64()) as i64),
    37 I64TruncF64U      |o: WasmValue| WasmValue::I64(libm::trunc(o.as_f64()) as u64 as i64),
    38 I64TruncSatF32S   |o: WasmValue| WasmValue::I64(libm::truncf(o.as_f32()).clamp(f32::MIN, f32::MAX) as i64),
    39 I64TruncSatF32U   |o: WasmValue| { let value = o.as_f32(); if value.is_nan() { WasmValue::I64(0) } else { let truncated = libm::truncf(value); let clamped = truncated.clamp(0.0, u64::MAX as f32); /* Clamp between 0 and u64::MAX as f32 */ let result = if clamped > u64::MAX as f32 { u64::MAX as i64 /* Return the maximum value for i64 */ } else { clamped as u64 as i64 /* Convert to u64 then to i64 */ }; WasmValue::I64(result) } },
    40 I64TruncSatF64S   |o: WasmValue| WasmValue::I64(libm::trunc(o.as_f64()).clamp(f64::MIN, f64::MAX) as i64),
    41 I64TruncSatF64U   |o: WasmValue| { let value = o.as_f64(); if value.is_nan() { WasmValue::I64(0) } else { let truncated = libm::trunc(value); let clamped = truncated.clamp(0.0, u64::MAX as f64); /* Clamp between 0 and u64::MAX */ /* If clamped exceeds u64::MAX, return the maximum value for i64 */ let result = if clamped > u64::MAX as f64 { u64::MAX as i64 /* Return the maximum value for i64 */ } else { clamped as u64 as i64 /* Convert to u64 then to i64 */ }; WasmValue::I64(result) } },
    42 F32ConvertI32S    |o: WasmValue| WasmValue::F32(o.as_i32() as f32),
    43 F32ConvertI32U    |o: WasmValue| WasmValue::F32(o.as_i32() as u32 as f32),
    44 F32ConvertI64S    |o: WasmValue| WasmValue::F32(o.as_i64() as f32),
    45 F32ConvertI64U    |o: WasmValue| WasmValue::F32(o.as_i64() as u64 as f32),
    46 F32DemoteF64      |o: WasmValue| WasmValue::F32(o.as_f64() as f32),
    47 F64ConvertI32S    |o: WasmValue| WasmValue::F64(o.as_i32() as f64),
    48 F64ConvertI32U    |o: WasmValue| WasmValue::F64(o.as_i32() as u32 as f64),
    49 F64ConvertI64S    |o: WasmValue| WasmValue::F64(o.as_i64() as f64),
    50 F64ConvertI64U    |o: WasmValue| WasmValue::F64(o.as_i64() as u64 as f64),
    51 F64PromoteF32     |o: WasmValue| WasmValue::F64(o.as_f32() as f64),
    52 I32ReinterpretF32 |o: WasmValue| WasmValue::I32(o.as_f32().to_bits() as i32),
    53 I64ReinterpretF64 |o: WasmValue| WasmValue::I64(o.as_f64().to_bits() as i64),
    54 F32ReinterpretI32 |o: WasmValue| WasmValue::F32(f32::from_le_bytes(o.as_i32().to_le_bytes())),
    55 F64ReinterpretI64 |o: WasmValue| WasmValue::F64(f64::from_le_bytes(o.as_i64().to_le_bytes())),
    56 I32Extend8S       |o: WasmValue| WasmValue::I32((o.as_i32() as  i8) as i32),
    57 I32Extend16S      |o: WasmValue| WasmValue::I32((o.as_i32() as i16) as i32),
    58 I64Extend8S       |o: WasmValue| WasmValue::I64((o.as_i64() as  i8)  as i64),
    59 I64Extend16S      |o: WasmValue| WasmValue::I64((o.as_i64() as i16)  as i64),
    60 I64Extend32S      |o: WasmValue| WasmValue::I64((o.as_i64() as i32)  as i64),
);
