use crate::WasmValue;

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
     4 I32Ctz            |o: WasmValue| WasmValue::I32(i32::count_zeros(o.as_i32()) as i32),
     5 I32Popcnt         |o: WasmValue| WasmValue::I32(i32::count_ones(o.as_i32()) as i32),
     6 I64Clz            |o: WasmValue| WasmValue::I32(i64::leading_zeros(o.as_i64()) as i32),
     7 I64Ctz            |o: WasmValue| WasmValue::I32(i64::count_zeros(o.as_i64()) as i32),
     8 I64Popcnt         |o: WasmValue| WasmValue::I32(i64::count_ones(o.as_i64()) as i32),
     9 F32Abs            |o: WasmValue| WasmValue::F32(libm::fabsf(o.as_f32())),
    10 F32Neg            |o: WasmValue| WasmValue::F32(-o.as_f32()),
    11 F32Ceil           |o: WasmValue| WasmValue::F32(libm::ceilf(o.as_f32())),
    12 F32Floor          |o: WasmValue| WasmValue::F32(libm::floorf(o.as_f32())),
    13 F32Trunc          |o: WasmValue| WasmValue::F32(libm::truncf(o.as_f32())),
    14 F32Nearest        |o: WasmValue| todo!("Provide a correct implementation for {o:?}.nearest"),
    15 F32Sqrt           |o: WasmValue| WasmValue::F32(libm::sqrtf(o.as_f32())),
    16 F64Abs            |o: WasmValue| WasmValue::F64(libm::fabs(o.as_f64())),
    17 F64Neg            |o: WasmValue| WasmValue::F64(-o.as_f64()),
    18 F64Ceil           |o: WasmValue| WasmValue::F64(libm::ceil(o.as_f64())),
    19 F64Floor          |o: WasmValue| WasmValue::F64(libm::floor(o.as_f64())),
    20 F64Trunc          |o: WasmValue| WasmValue::F64(libm::trunc(o.as_f64())),
    21 F64Nearest        |o: WasmValue| todo!("Provide a correct implementation for {o:?}.nearest"),
    22 F64Sqrt           |o: WasmValue| WasmValue::F64(libm::sqrt(o.as_f64())),
    23 I32WrapI64        |o: WasmValue| WasmValue::I32(o.as_i64() as i32),
    24 I32TruncF32S      |o: WasmValue| WasmValue::I32(libm::truncf(o.as_f32()) as i32),
    25 I32TruncF32U      |o: WasmValue| WasmValue::I32(libm::fabsf(libm::truncf(o.as_f32())) as i32),
    26 I32TruncF64S      |o: WasmValue| WasmValue::I32(libm::trunc(o.as_f64()) as i32),
    27 I32TruncF64U      |o: WasmValue| WasmValue::I32(libm::fabs(libm::trunc(o.as_f64())) as i32),
    28 I32TruncSatF32S   |o: WasmValue| WasmValue::F32(libm::truncf(o.as_f32()).clamp(f32::MIN, f32::MAX)),
    29 I32TruncSatF32U   |o: WasmValue| WasmValue::I32(libm::fabsf(libm::truncf(o.as_f32())).clamp(0.0, f32::MAX) as i32),
    30 I32TruncSatF64S   |o: WasmValue| WasmValue::I32(libm::trunc(o.as_f64()).clamp(f64::MIN, f64::MAX) as i32),
    31 I32TruncSatF64U   |o: WasmValue| WasmValue::I32(libm::fabs(libm::trunc(o.as_f64())).clamp(0.0, f64::MAX) as i32),
    32 I64ExtendI32S     |o: WasmValue| WasmValue::I64(o.as_i32() as i64),
    33 I64ExtendI32U     |o: WasmValue| WasmValue::I64(o.as_i32() as u64 as i64),
    34 I64TruncF32S      |o: WasmValue| WasmValue::I64(libm::truncf(o.as_f32()) as i64),
    35 I64TruncF32U      |o: WasmValue| WasmValue::I64(libm::fabsf(libm::truncf(o.as_f32())) as i64),
    36 I64TruncF64S      |o: WasmValue| WasmValue::I64(libm::trunc(o.as_f64()) as i64),
    37 I64TruncF64U      |o: WasmValue| WasmValue::I64(libm::fabs(libm::trunc(o.as_f64())) as i64),
    38 I64TruncSatF32S   |o: WasmValue| WasmValue::I64(libm::truncf(o.as_f32()).clamp(f32::MIN, f32::MAX) as i64),
    39 I64TruncSatF32U   |o: WasmValue| WasmValue::I64(libm::fabsf(libm::truncf(o.as_f32())).clamp(0.0, f32::MAX) as i64),
    40 I64TruncSatF64S   |o: WasmValue| WasmValue::I64(libm::trunc(o.as_f64()).clamp(f64::MIN, f64::MAX) as i64),
    41 I64TruncSatF64U   |o: WasmValue| WasmValue::I64(libm::fabs(libm::trunc(o.as_f64())).clamp(0.0, f64::MAX) as i64),
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
    52 I32ReinterpretF32 |o: WasmValue| WasmValue::I32(o.as_f32().to_bits().try_into().unwrap()),
    53 I64ReinterpretF64 |o: WasmValue| WasmValue::I64(o.as_f64().to_bits().try_into().unwrap()),
    54 F32ReinterpretI32 |o: WasmValue| WasmValue::F32(f32::from_le_bytes(o.as_i32().to_le_bytes())),
    55 F64ReinterpretI64 |o: WasmValue| WasmValue::F64(f64::from_le_bytes(o.as_i64().to_le_bytes())),
    56 I32Extend8S       |o: WasmValue| WasmValue::I32((o.as_i32() as i8 as i32) << 24),
    57 I32Extend16S      |o: WasmValue| WasmValue::I32((o.as_i32() as i16 as i32) << 16),
    58 I64Extend8S       |o: WasmValue| WasmValue::I64((o.as_i64() as i8 as i64) << 56),
    59 I64Extend16S      |o: WasmValue| WasmValue::I64((o.as_i64() as i16 as i64) << 48),
    60 I64Extend32S      |o: WasmValue| WasmValue::I64((o.as_i64() as i32 as i64) << 32)
);
