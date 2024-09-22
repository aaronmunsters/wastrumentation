use super::{Body, HighLevelBody, Instr, LowLevelBody};
use wasabi_wasm::{FunctionType, Val::I32, ValType};

fn typ_void_to_i32() -> FunctionType {
    FunctionType::new(&[], &[ValType::I32])
}

fn const_i32(i: i32) -> Instr {
    Instr::Const(I32(i))
}

#[test]
fn test_parse_simple() {
    let low_level_body = LowLevelBody({
        use wasabi_wasm::{Instr::*, Val::I32};
        vec![
            // Body begin
            If(FunctionType::empty()),
            Const(I32(0)),
            Else,
            Const(I32(1)),
            End,
            End, // Body end
        ]
    });

    let high_level_body_expected = HighLevelBody({
        vec![(
            0,
            Instr::if_then_else(
                FunctionType::empty(),
                vec![(1, const_i32(0))],
                vec![(3, const_i32(1))],
            ),
        )]
    });

    assert_eq!(high_level_body_expected, low_level_body.try_into().unwrap());
}

fn wat_to_low_level(wat: &str) -> LowLevelBody {
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let (module, _, _) = wasabi_wasm::Module::from_bytes(&wasm_bytes).unwrap();
    let foo_body = &module.function(0_usize.into()).code().unwrap().body;
    LowLevelBody(foo_body.to_vec())
}

fn assert_high_and_low(
    wasm_program: &'static str,
    body_expected_low: &Vec<wasabi_wasm::Instr>,
    body_expected_high: &Body,
) {
    let LowLevelBody(low_level_body) = wat_to_low_level(wasm_program);
    // Assert expected input program
    assert_eq!(body_expected_low, &low_level_body);
    let HighLevelBody(high_level_body) = LowLevelBody(low_level_body).try_into().unwrap();
    // Assert succesful nesting parse after parse to high-level representation
    assert_eq!(&high_level_body, body_expected_high);
    let LowLevelBody(low_level_body) = HighLevelBody(high_level_body).into();
    // Assert that the conversion to low-level representation is equivalent
    assert_eq!(body_expected_low, &low_level_body);
}

// TODO: macro's for huge bodies?

#[test]
fn high_level_low_level_assert() {
    assert_high_and_low(
        include_str!("branch.wat"),
        {
            use wasabi_wasm::{Instr::*, Val::I32};
            &vec![
                Const(I32(0)),
                If(typ_void_to_i32()),
                Const(I32(1)),
                Else,
                Const(I32(2)),
                End,
                Drop,
                End,
            ]
        },
        &vec![
            (0, const_i32(0)),
            (
                1,
                Instr::if_then_else(
                    typ_void_to_i32(),
                    vec![(2, const_i32(1))],
                    vec![(4, const_i32(2))],
                ),
            ),
            (6, Instr::Drop),
        ],
    );
}

#[test]
fn test_wat_to_high_level_complex() {
    assert_high_and_low(
        include_str!("nested-branch.wat"),
        {
            use wasabi_wasm::{Instr::*, Val::I32};
            &vec![
                Const(I32(1000)),
                If(typ_void_to_i32()),
                Const(I32(1001)),
                If(typ_void_to_i32()),
                Const(I32(1002)),
                If(typ_void_to_i32()),
                Const(I32(0)),
                Else,
                Const(I32(1)),
                End,
                Else,
                Const(I32(1003)),
                If(typ_void_to_i32()),
                Const(I32(2)),
                Else,
                Const(I32(3)),
                End,
                End,
                Drop,
                Const(I32(1004)),
                If(typ_void_to_i32()),
                Const(I32(4)),
                Else,
                Const(I32(1005)),
                If(typ_void_to_i32()),
                Const(I32(5)),
                Else,
                Const(I32(6)),
                End,
                End,
                Else,
                Const(I32(1006)),
                If(typ_void_to_i32()),
                Const(I32(1007)),
                If(typ_void_to_i32()),
                Const(I32(1008)),
                If(typ_void_to_i32()),
                Const(I32(7)),
                Else,
                Const(I32(8)),
                End,
                Else,
                Const(I32(9)),
                End,
                Else,
                Const(I32(1009)),
                If(typ_void_to_i32()),
                Const(I32(1010)),
                If(typ_void_to_i32()),
                Const(I32(1011)),
                If(typ_void_to_i32()),
                Const(I32(10)),
                Else,
                Const(I32(11)),
                End,
                Else,
                Const(I32(12)),
                End,
                Else,
                Const(I32(13)),
                End,
                End,
                Drop,
                Const(I32(1012)),
                If(typ_void_to_i32()),
                Const(I32(14)),
                Else,
                Const(I32(15)),
                End,
                End,
                Drop,
                End,
            ]
        },
        {
            use super::Instr::Drop;
            let if_res_i32 = |then, else_| Instr::if_then_else(typ_void_to_i32(), then, else_);
            &vec![
                (0, const_i32(1000)),
                (
                    1,
                    if_res_i32(
                        vec![
                            (2, const_i32(1001)),
                            (
                                3,
                                if_res_i32(
                                    vec![
                                        (4, const_i32(1002)),
                                        (
                                            5,
                                            if_res_i32(
                                                vec![(6, const_i32(0))],
                                                vec![(8, const_i32(1))],
                                            ),
                                        ),
                                    ],
                                    vec![
                                        (11, const_i32(1003)),
                                        (
                                            12,
                                            if_res_i32(
                                                vec![(13, const_i32(2))],
                                                vec![(15, const_i32(3))],
                                            ),
                                        ),
                                    ],
                                ),
                            ),
                            (18, Drop),
                            (19, const_i32(1004)),
                            (
                                20,
                                if_res_i32(
                                    vec![(21, const_i32(4))],
                                    vec![
                                        (23, const_i32(1005)),
                                        (
                                            24,
                                            if_res_i32(
                                                vec![(25, const_i32(5))],
                                                vec![(27, const_i32(6))],
                                            ),
                                        ),
                                    ],
                                ),
                            ),
                        ],
                        vec![
                            (31, const_i32(1006)),
                            (
                                32,
                                if_res_i32(
                                    vec![
                                        (33, const_i32(1007)),
                                        (
                                            34,
                                            if_res_i32(
                                                vec![
                                                    (35, const_i32(1008)),
                                                    (
                                                        36,
                                                        if_res_i32(
                                                            vec![(37, const_i32(7))],
                                                            vec![(39, const_i32(8))],
                                                        ),
                                                    ),
                                                ],
                                                vec![(42, const_i32(9))],
                                            ),
                                        ),
                                    ],
                                    vec![
                                        (45, const_i32(1009)),
                                        (
                                            46,
                                            if_res_i32(
                                                vec![
                                                    (47, const_i32(1010)),
                                                    (
                                                        48,
                                                        if_res_i32(
                                                            vec![
                                                                (49, const_i32(1011)),
                                                                (
                                                                    50,
                                                                    if_res_i32(
                                                                        vec![(51, const_i32(10))],
                                                                        vec![(53, const_i32(11))],
                                                                    ),
                                                                ),
                                                            ],
                                                            vec![(56, const_i32(12))],
                                                        ),
                                                    ),
                                                ],
                                                vec![(59, const_i32(13))],
                                            ),
                                        ),
                                    ],
                                ),
                            ),
                            (62, Drop),
                            (63, const_i32(1012)),
                            (
                                64,
                                if_res_i32(vec![(65, const_i32(14))], vec![(67, const_i32(15))]),
                            ),
                        ],
                    ),
                ),
                (70, Drop),
            ]
        },
    );
}
