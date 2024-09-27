use crate::WasmValue;
use core::fmt::Display;

macro_rules! generate_unary_boilerplate {
    (
        $(
            $serialized:literal $operator:ident $apply_body:expr
        ),*
        $(,)?

    ) => {
        #[derive(Debug, Clone, Copy)]
        pub enum BinaryOperator {
            $(
                $operator
            ),*
        }

        impl Display for BinaryOperator {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                let s = match self {
                    $(
                        Self::$operator => stringify!($operator),
                    )*

                };
                write!(f, "{s}")
            }
        }

        impl BinaryOperator {
            pub fn apply(&self, l: WasmValue, r: WasmValue) -> WasmValue {
                match self {
                    $(
                        Self::$operator => $apply_body(l, r),
                    )*
                }
            }
        }

        impl From<i32> for BinaryOperator {
            fn from(value: i32) -> Self {
                match value {
                    $(
                        $serialized => BinaryOperator::$operator,
                    )*
                    v => panic!("Unknown serialized unary operator value: {v}"),
                }
            }
        }
    };
}

generate_unary_boilerplate!(
// IDX VARIANT           CONVERSION IMPLEMENTATION
     1 I32Eq             |l: WasmValue, r: WasmValue| WasmValue::i32_from_bool(l.as_i32() == r.as_i32()),
     2 I32Ne             |l: WasmValue, r: WasmValue| WasmValue::i32_from_bool(l.as_i32() != r.as_i32()),
     3 I32LtS            |l: WasmValue, r: WasmValue| WasmValue::i32_from_bool(l.as_i32() < r.as_i32()),
     4 I32LtU            |l: WasmValue, r: WasmValue| WasmValue::i32_from_bool(l.as_i32() < r.as_i32()),
     5 I32GtS            |l: WasmValue, r: WasmValue| WasmValue::i32_from_bool(l.as_i32() > r.as_i32()),
     6 I32GtU            |l: WasmValue, r: WasmValue| WasmValue::i32_from_bool(l.as_i32() > r.as_i32()),
     7 I32LeS            |l: WasmValue, r: WasmValue| WasmValue::i32_from_bool(l.as_i32() <= r.as_i32()),
     8 I32LeU            |l: WasmValue, r: WasmValue| WasmValue::i32_from_bool(l.as_i32() <= r.as_i32()),
     9 I32GeS            |l: WasmValue, r: WasmValue| WasmValue::i32_from_bool(l.as_i32() >= r.as_i32()),
    10 I32GeU            |l: WasmValue, r: WasmValue| WasmValue::i32_from_bool(l.as_i32() >= r.as_i32()),
    11 I64Eq             |l: WasmValue, r: WasmValue| WasmValue::i32_from_bool(l.as_i64() == r.as_i64()),
    12 I64Ne             |l: WasmValue, r: WasmValue| WasmValue::i32_from_bool(l.as_i64() != r.as_i64()),
    13 I64LtS            |l: WasmValue, r: WasmValue| WasmValue::i32_from_bool(l.as_i64() < r.as_i64()),
    14 I64LtU            |l: WasmValue, r: WasmValue| WasmValue::i32_from_bool(l.as_i64() < r.as_i64()),
    15 I64GtS            |l: WasmValue, r: WasmValue| WasmValue::i32_from_bool(l.as_i64() > r.as_i64()),
    16 I64GtU            |l: WasmValue, r: WasmValue| WasmValue::i32_from_bool(l.as_i64() > r.as_i64()),
    17 I64LeS            |l: WasmValue, r: WasmValue| WasmValue::i32_from_bool(l.as_i64() <= r.as_i64()),
    18 I64LeU            |l: WasmValue, r: WasmValue| WasmValue::i32_from_bool(l.as_i64() <= r.as_i64()),
    19 I64GeS            |l: WasmValue, r: WasmValue| WasmValue::i32_from_bool(l.as_i64() >= r.as_i64()),
    20 I64GeU            |l: WasmValue, r: WasmValue| WasmValue::i32_from_bool(l.as_i64() >= r.as_i64()),
    21 F32Eq             |l: WasmValue, r: WasmValue| WasmValue::i32_from_bool(l.as_f32() == r.as_f32()),
    22 F32Ne             |l: WasmValue, r: WasmValue| WasmValue::i32_from_bool(l.as_f32() != r.as_f32()),
    23 F32Lt             |l: WasmValue, r: WasmValue| WasmValue::i32_from_bool(l.as_f32() < r.as_f32()),
    24 F32Gt             |l: WasmValue, r: WasmValue| WasmValue::i32_from_bool(l.as_f32() > r.as_f32()),
    25 F32Le             |l: WasmValue, r: WasmValue| WasmValue::i32_from_bool(l.as_f32() <= r.as_f32()),
    26 F32Ge             |l: WasmValue, r: WasmValue| WasmValue::i32_from_bool(l.as_f32() >= r.as_f32()),
    27 F64Eq             |l: WasmValue, r: WasmValue| WasmValue::i32_from_bool(l.as_f64() == r.as_f64()),
    28 F64Ne             |l: WasmValue, r: WasmValue| WasmValue::i32_from_bool(l.as_f64() != r.as_f64()),
    29 F64Lt             |l: WasmValue, r: WasmValue| WasmValue::i32_from_bool(l.as_f64() < r.as_f64()),
    30 F64Gt             |l: WasmValue, r: WasmValue| WasmValue::i32_from_bool(l.as_f64() > r.as_f64()),
    31 F64Le             |l: WasmValue, r: WasmValue| WasmValue::i32_from_bool(l.as_f64() <= r.as_f64()),
    32 F64Ge             |l: WasmValue, r: WasmValue| WasmValue::i32_from_bool(l.as_f64() >= r.as_f64()),
    33 I32Add            |l: WasmValue, r: WasmValue| WasmValue::I32(l.as_i32() + r.as_i32()),
    34 I32Sub            |l: WasmValue, r: WasmValue| WasmValue::I32(l.as_i32() - r.as_i32()),
    35 I32Mul            |l: WasmValue, r: WasmValue| WasmValue::I32(l.as_i32() * r.as_i32()),
    36 I32DivS           |l: WasmValue, r: WasmValue| WasmValue::I32(l.as_i32() / r.as_i32()),
    37 I32DivU           |l: WasmValue, r: WasmValue| WasmValue::I32(l.as_i32() / r.as_i32()),
    38 I32RemS           |l: WasmValue, r: WasmValue| WasmValue::I32(l.as_i32() % r.as_i32()),
    39 I32RemU           |l: WasmValue, r: WasmValue| WasmValue::I32(l.as_i32() % r.as_i32()),
    40 I32And            |l: WasmValue, r: WasmValue| WasmValue::I32(l.as_i32() & r.as_i32()),
    41 I32Or             |l: WasmValue, r: WasmValue| WasmValue::I32(l.as_i32() | r.as_i32()),
    42 I32Xor            |l: WasmValue, r: WasmValue| WasmValue::I32(l.as_i32() ^ r.as_i32()),
    43 I32Shl            |l: WasmValue, r: WasmValue| WasmValue::I32(l.as_i32() << r.as_i32()),
    44 I32ShrS           |l: WasmValue, r: WasmValue| WasmValue::I32(l.as_i32() >> r.as_i32()),
    45 I32ShrU           |l: WasmValue, r: WasmValue| WasmValue::I32(l.as_i32() >> r.as_i32()),
    46 I32Rotl           |l: WasmValue, r: WasmValue| WasmValue::I32(l.as_i32().rotate_left(r.as_i32() as u32)),
    47 I32Rotr           |l: WasmValue, r: WasmValue| WasmValue::I32(l.as_i32().rotate_right(r.as_i32() as u32)),
    48 I64Add            |l: WasmValue, r: WasmValue| WasmValue::I64(l.as_i64() + r.as_i64()),
    49 I64Sub            |l: WasmValue, r: WasmValue| WasmValue::I64(l.as_i64() - r.as_i64()),
    50 I64Mul            |l: WasmValue, r: WasmValue| WasmValue::I64(l.as_i64() * r.as_i64()),
    51 I64DivS           |l: WasmValue, r: WasmValue| WasmValue::I64(l.as_i64() / r.as_i64()),
    52 I64DivU           |l: WasmValue, r: WasmValue| WasmValue::I64(l.as_i64() / r.as_i64()),
    53 I64RemS           |l: WasmValue, r: WasmValue| WasmValue::I64(l.as_i64() % r.as_i64()),
    54 I64RemU           |l: WasmValue, r: WasmValue| WasmValue::I64(l.as_i64() % r.as_i64()),
    55 I64And            |l: WasmValue, r: WasmValue| WasmValue::I64(l.as_i64() & r.as_i64()),
    56 I64Or             |l: WasmValue, r: WasmValue| WasmValue::I64(l.as_i64() | r.as_i64()),
    57 I64Xor            |l: WasmValue, r: WasmValue| WasmValue::I64(l.as_i64() ^ r.as_i64()),
    58 I64Shl            |l: WasmValue, r: WasmValue| WasmValue::I64(l.as_i64() << r.as_i32()),
    59 I64ShrS           |l: WasmValue, r: WasmValue| WasmValue::I64(l.as_i64() >> r.as_i32()),
    60 I64ShrU           |l: WasmValue, r: WasmValue| WasmValue::I64(l.as_i64() >> r.as_i32()),
    61 I64Rotl           |l: WasmValue, r: WasmValue| WasmValue::I64(l.as_i64().rotate_left(r.as_i32() as u32)),
    62 I64Rotr           |l: WasmValue, r: WasmValue| WasmValue::I64(l.as_i64().rotate_right(r.as_i32() as u32)),
    63 F32Add            |l: WasmValue, r: WasmValue| WasmValue::F32(l.as_f32() + r.as_f32()),
    64 F32Sub            |l: WasmValue, r: WasmValue| WasmValue::F32(l.as_f32() - r.as_f32()),
    65 F32Mul            |l: WasmValue, r: WasmValue| WasmValue::F32(l.as_f32() * r.as_f32()),
    66 F32Div            |l: WasmValue, r: WasmValue| WasmValue::F32(l.as_f32() / r.as_f32()),
    67 F32Min            |l: WasmValue, r: WasmValue| WasmValue::F32(l.as_f32().min(r.as_f32())),
    68 F32Max            |l: WasmValue, r: WasmValue| WasmValue::F32(l.as_f32().max(r.as_f32())),
    69 F32Copysign       |l: WasmValue, r: WasmValue| WasmValue::F32(libm::copysignf(l.as_f32(), r.as_f32())),
    70 F64Add            |l: WasmValue, r: WasmValue| WasmValue::F64(l.as_f64() + r.as_f64()),
    71 F64Sub            |l: WasmValue, r: WasmValue| WasmValue::F64(l.as_f64() - r.as_f64()),
    72 F64Mul            |l: WasmValue, r: WasmValue| WasmValue::F64(l.as_f64() * r.as_f64()),
    73 F64Div            |l: WasmValue, r: WasmValue| WasmValue::F64(l.as_f64() / r.as_f64()),
    74 F64Min            |l: WasmValue, r: WasmValue| WasmValue::F64(l.as_f64().min(r.as_f64())),
    75 F64Max            |l: WasmValue, r: WasmValue| WasmValue::F64(l.as_f64().max(r.as_f64())),
    76 F64Copysign       |l: WasmValue, r: WasmValue| WasmValue::F64(libm::copysign(l.as_f64(), r.as_f64())),
);
