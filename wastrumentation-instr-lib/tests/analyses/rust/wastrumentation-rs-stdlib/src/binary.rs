#[cfg(target_arch = "wasm32")]
use core::arch::asm;

use crate::WasmValue;
use cfg_if::cfg_if;

macro_rules! generate_binary_boilerplate {
    (
        $(
            $serialized:literal
            $operator:ident
            : $l_in_type:ident -> $r_in_type:ident -> $wasm_asm_instruction:literal -> $wasm_res_out_type:ident
            REIFIED:
            $apply_body:expr
        ),*
        $(,)?
    ) => {
        #[derive(Debug, Clone, Copy)]
        pub enum BinaryOperator {
            $(
                $operator
            ),*
        }

        impl BinaryOperator {
            pub fn apply(&self, l: WasmValue, r: WasmValue) -> WasmValue {
                match self {
                    $(
                        Self::$operator => {
                            let l_operand: $l_in_type = generate_binary_boilerplate!(@internally WasmValue-to-primitive l $l_in_type);
                            let r_operand: $r_in_type = generate_binary_boilerplate!(@internally WasmValue-to-primitive r $r_in_type);

                            cfg_if! {
                                if #[cfg(target_arch = "wasm32")] {
                                    unsafe {
                                        let result: $wasm_res_out_type;
                                        asm! {
                                            "local.get {l_operand}",
                                            "local.get {r_operand}",
                                            $wasm_asm_instruction,
                                            "local.set {result}",
                                            l_operand = in(local) l_operand,
                                            r_operand = in(local) r_operand,
                                            result = out(local) result
                                        };
                                        generate_binary_boilerplate!(@internally primitive-to-WasmValue result $wasm_res_out_type)
                                    }
                                } else {
                                    /* Using alternative attempt */
                                    let result: $wasm_res_out_type = $apply_body(l_operand, r_operand);
                                    generate_binary_boilerplate!(@internally primitive-to-WasmValue result $wasm_res_out_type)
                                }
                            }
                        }
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

    (@internally WasmValue-to-primitive $wasm_value:ident i32) => { $wasm_value.as_i32() };
    (@internally WasmValue-to-primitive $wasm_value:ident f32) => { $wasm_value.as_f32() };
    (@internally WasmValue-to-primitive $wasm_value:ident i64) => { $wasm_value.as_i64() };
    (@internally WasmValue-to-primitive $wasm_value:ident f64) => { $wasm_value.as_f64() };

    (@internally primitive-to-WasmValue $value:ident i32) => { WasmValue::I32($value) };
    (@internally primitive-to-WasmValue $value:ident f32) => { WasmValue::F32($value) };
    (@internally primitive-to-WasmValue $value:ident i64) => { WasmValue::I64($value) };
    (@internally primitive-to-WasmValue $value:ident f64) => { WasmValue::F64($value) };
}

generate_binary_boilerplate!(
// IDX VARIANT     <-LIN----RIN----WASM_OPERATOR----OUT->        <--------------------------RUST IMPL------------------------->
     1 I32Eq       : i32 -> i32 -> "i32.eq"      -> i32 REIFIED: |l: i32, r: i32| ( l         ==              r        ) as i32,
     2 I32Ne       : i32 -> i32 -> "i32.ne"      -> i32 REIFIED: |l: i32, r: i32| ( l         !=              r        ) as i32,
     3 I32LtS      : i32 -> i32 -> "i32.lt_s"    -> i32 REIFIED: |l: i32, r: i32| ( l         <               r        ) as i32,
     4 I32LtU      : i32 -> i32 -> "i32.lt_u"    -> i32 REIFIED: |l: i32, r: i32| ((l as u32) <              (r as u32)) as i32,
     5 I32GtS      : i32 -> i32 -> "i32.gt_s"    -> i32 REIFIED: |l: i32, r: i32| ( l         >               r        ) as i32,
     6 I32GtU      : i32 -> i32 -> "i32.gt_u"    -> i32 REIFIED: |l: i32, r: i32| ((l as u32) >              (r as u32)) as i32,
     7 I32LeS      : i32 -> i32 -> "i32.le_s"    -> i32 REIFIED: |l: i32, r: i32| ( l         <=              r        ) as i32,
     8 I32LeU      : i32 -> i32 -> "i32.le_u"    -> i32 REIFIED: |l: i32, r: i32| ((l as u32) <=             (r as u32)) as i32,
     9 I32GeS      : i32 -> i32 -> "i32.ge_s"    -> i32 REIFIED: |l: i32, r: i32| ( l         >=              r        ) as i32,
    10 I32GeU      : i32 -> i32 -> "i32.ge_u"    -> i32 REIFIED: |l: i32, r: i32| ((l as u32) >=             (r as u32)) as i32,
    11 I64Eq       : i64 -> i64 -> "i64.eq"      -> i32 REIFIED: |l: i64, r: i64| ( l         ==              r        ) as i32,
    12 I64Ne       : i64 -> i64 -> "i64.ne"      -> i32 REIFIED: |l: i64, r: i64| ( l         !=              r        ) as i32,
    13 I64LtS      : i64 -> i64 -> "i64.lt_s"    -> i32 REIFIED: |l: i64, r: i64| ( l         <               r        ) as i32,
    14 I64LtU      : i64 -> i64 -> "i64.lt_u"    -> i32 REIFIED: |l: i64, r: i64| ((l as u64) <              (r as u64)) as i32,
    15 I64GtS      : i64 -> i64 -> "i64.gt_s"    -> i32 REIFIED: |l: i64, r: i64| ( l         >               r        ) as i32,
    16 I64GtU      : i64 -> i64 -> "i64.gt_u"    -> i32 REIFIED: |l: i64, r: i64| ((l as u64) >              (r as u64)) as i32,
    17 I64LeS      : i64 -> i64 -> "i64.le_s"    -> i32 REIFIED: |l: i64, r: i64| ( l         <=              r        ) as i32,
    18 I64LeU      : i64 -> i64 -> "i64.le_u"    -> i32 REIFIED: |l: i64, r: i64| ((l as u64) <=             (r as u64)) as i32,
    19 I64GeS      : i64 -> i64 -> "i64.ge_s"    -> i32 REIFIED: |l: i64, r: i64| ( l         >=              r        ) as i32,
    20 I64GeU      : i64 -> i64 -> "i64.ge_u"    -> i32 REIFIED: |l: i64, r: i64| ((l as u64) >=             (r as u64)) as i32,
    21 F32Eq       : f32 -> f32 -> "f32.eq"      -> i32 REIFIED: |l: f32, r: f32| ( l         ==              r        ) as i32,
    22 F32Ne       : f32 -> f32 -> "f32.ne"      -> i32 REIFIED: |l: f32, r: f32| ( l         !=              r        ) as i32,
    23 F32Lt       : f32 -> f32 -> "f32.lt"      -> i32 REIFIED: |l: f32, r: f32| ( l         <               r        ) as i32,
    24 F32Gt       : f32 -> f32 -> "f32.gt"      -> i32 REIFIED: |l: f32, r: f32| ( l         >               r        ) as i32,
    25 F32Le       : f32 -> f32 -> "f32.le"      -> i32 REIFIED: |l: f32, r: f32| ( l         <=              r        ) as i32,
    26 F32Ge       : f32 -> f32 -> "f32.ge"      -> i32 REIFIED: |l: f32, r: f32| ( l         >=              r        ) as i32,
    27 F64Eq       : f64 -> f64 -> "f64.eq"      -> i32 REIFIED: |l: f64, r: f64| ( l         ==              r        ) as i32,
    28 F64Ne       : f64 -> f64 -> "f64.ne"      -> i32 REIFIED: |l: f64, r: f64| ( l         !=              r        ) as i32,
    29 F64Lt       : f64 -> f64 -> "f64.lt"      -> i32 REIFIED: |l: f64, r: f64| ( l         <               r        ) as i32,
    30 F64Gt       : f64 -> f64 -> "f64.gt"      -> i32 REIFIED: |l: f64, r: f64| ( l         >               r        ) as i32,
    31 F64Le       : f64 -> f64 -> "f64.le"      -> i32 REIFIED: |l: f64, r: f64| ( l         <=              r        ) as i32,
    32 F64Ge       : f64 -> f64 -> "f64.ge"      -> i32 REIFIED: |l: f64, r: f64| ( l         >=              r        ) as i32,
    33 I32Add      : i32 -> i32 -> "i32.add"     -> i32 REIFIED: |l: i32, r: i32|   l         +               r                ,
    34 I32Sub      : i32 -> i32 -> "i32.sub"     -> i32 REIFIED: |l: i32, r: i32|   l         -               r                ,
    35 I32Mul      : i32 -> i32 -> "i32.mul"     -> i32 REIFIED: |l: i32, r: i32|   l         *               r                ,
    36 I32DivS     : i32 -> i32 -> "i32.div_s"   -> i32 REIFIED: |l: i32, r: i32|   l         /               r                ,
    37 I32DivU     : i32 -> i32 -> "i32.div_u"   -> i32 REIFIED: |l: i32, r: i32| ((l as u32) /              (r as u32)) as i32,
    38 I32RemS     : i32 -> i32 -> "i32.rem_s"   -> i32 REIFIED: |l: i32, r: i32|   l         .wrapping_rem(  r               ),
    39 I32RemU     : i32 -> i32 -> "i32.rem_u"   -> i32 REIFIED: |l: i32, r: i32| ((l as u32) %              (r as u32)) as i32,
    40 I32And      : i32 -> i32 -> "i32.and"     -> i32 REIFIED: |l: i32, r: i32|   l         &               r                ,
    41 I32Or       : i32 -> i32 -> "i32.or"      -> i32 REIFIED: |l: i32, r: i32|   l         |               r                ,
    42 I32Xor      : i32 -> i32 -> "i32.xor"     -> i32 REIFIED: |l: i32, r: i32|   l         ^               r                ,
    43 I32Shl      : i32 -> i32 -> "i32.shl"     -> i32 REIFIED: |l: i32, r: i32|   l         <<              r                ,
    44 I32ShrS     : i32 -> i32 -> "i32.shr_s"   -> i32 REIFIED: |l: i32, r: i32|   l         >>              r                ,
    45 I32ShrU     : i32 -> i32 -> "i32.shr_u"   -> i32 REIFIED: |l: i32, r: i32| ((l as u32) >>             (r as u32)) as i32,
    46 I32Rotl     : i32 -> i32 -> "i32.rotl"    -> i32 REIFIED: |l: i32, r: i32|   l         .rotate_left(   r as u32        ),
    47 I32Rotr     : i32 -> i32 -> "i32.rotr"    -> i32 REIFIED: |l: i32, r: i32|   l         .rotate_right(  r as u32        ),
    48 I64Add      : i64 -> i64 -> "i64.add"     -> i64 REIFIED: |l: i64, r: i64|   l         +               r                ,
    49 I64Sub      : i64 -> i64 -> "i64.sub"     -> i64 REIFIED: |l: i64, r: i64|   l         -               r                ,
    50 I64Mul      : i64 -> i64 -> "i64.mul"     -> i64 REIFIED: |l: i64, r: i64|   l         *               r                ,
    51 I64DivS     : i64 -> i64 -> "i64.div_s"   -> i64 REIFIED: |l: i64, r: i64|   l         /               r                ,
    52 I64DivU     : i64 -> i64 -> "i64.div_u"   -> i64 REIFIED: |l: i64, r: i64| ((l as u64) /              (r as u64)) as i64,
    53 I64RemS     : i64 -> i64 -> "i64.rem_s"   -> i64 REIFIED: |l: i64, r: i64|   l         .wrapping_rem(  r               ),
    54 I64RemU     : i64 -> i64 -> "i64.rem_u"   -> i64 REIFIED: |l: i64, r: i64| ((l as u64) %              (r as u64)) as i64,
    55 I64And      : i64 -> i64 -> "i64.and"     -> i64 REIFIED: |l: i64, r: i64|   l         &               r                ,
    56 I64Or       : i64 -> i64 -> "i64.or"      -> i64 REIFIED: |l: i64, r: i64|   l         |               r                ,
    57 I64Xor      : i64 -> i64 -> "i64.xor"     -> i64 REIFIED: |l: i64, r: i64|   l         ^               r                ,
    58 I64Shl      : i64 -> i64 -> "i64.shl"     -> i64 REIFIED: |l: i64, r: i64|   l         <<              r                ,
    59 I64ShrS     : i64 -> i64 -> "i64.shr_s"   -> i64 REIFIED: |l: i64, r: i64|   l         >>              r                ,
    60 I64ShrU     : i64 -> i64 -> "i64.shr_u"   -> i64 REIFIED: |l: i64, r: i64| ((l as u64) >>             (r as u64)) as i64,
    61 I64Rotl     : i64 -> i64 -> "i64.rotl"    -> i64 REIFIED: |l: i64, r: i64|   l         .rotate_left(   r as u32        ),
    62 I64Rotr     : i64 -> i64 -> "i64.rotr"    -> i64 REIFIED: |l: i64, r: i64|   l         .rotate_right(  r as u32        ),
    63 F32Add      : f32 -> f32 -> "f32.add"     -> f32 REIFIED: |l: f32, r: f32|   l         +               r                ,
    64 F32Sub      : f32 -> f32 -> "f32.sub"     -> f32 REIFIED: |l: f32, r: f32|   l         -               r                ,
    65 F32Mul      : f32 -> f32 -> "f32.mul"     -> f32 REIFIED: |l: f32, r: f32|   l         *               r                ,
    66 F32Div      : f32 -> f32 -> "f32.div"     -> f32 REIFIED: |l: f32, r: f32|   l         /               r                ,
    67 F32Min      : f32 -> f32 -> "f32.min"     -> f32 REIFIED: |l: f32, r: f32|             libm::fminf(    l, r            ),
    68 F32Max      : f32 -> f32 -> "f32.max"     -> f32 REIFIED: |l: f32, r: f32|             libm::fmaxf(    l, r            ),
    69 F32Copysign : f32 -> f32 -> "f32.copysign"-> f32 REIFIED: |l: f32, r: f32|             libm::copysignf(l, r            ),
    70 F64Add      : f64 -> f64 -> "f64.add"     -> f64 REIFIED: |l: f64, r: f64|   l         +               r                ,
    71 F64Sub      : f64 -> f64 -> "f64.sub"     -> f64 REIFIED: |l: f64, r: f64|   l         -               r                ,
    72 F64Mul      : f64 -> f64 -> "f64.mul"     -> f64 REIFIED: |l: f64, r: f64|   l         *               r                ,
    73 F64Div      : f64 -> f64 -> "f64.div"     -> f64 REIFIED: |l: f64, r: f64|   l         /               r                ,
    74 F64Min      : f64 -> f64 -> "f64.min"     -> f64 REIFIED: |l: f64, r: f64|   l         .min(           r               ),
    75 F64Max      : f64 -> f64 -> "f64.max"     -> f64 REIFIED: |l: f64, r: f64|   l         .max(           r               ),
    76 F64Copysign : f64 -> f64 -> "f64.copysign"-> f64 REIFIED: |l: f64, r: f64|             libm::copysign( l, r            ),
);
