use super::{HighLevelBody, HighLevelInstr, LowLevelBody, TypedHighLevelInstr};
use wasabi_wasm::{FunctionType, Val::I32, ValType};

fn typ_void_to_i32() -> FunctionType {
    FunctionType::new(&[], &[ValType::I32])
}

fn typ_i32_to_void() -> FunctionType {
    FunctionType::new(&[ValType::I32], &[])
}

fn const_i32(i: i32) -> HighLevelInstr {
    HighLevelInstr::Const(I32(i))
}

fn wat_to_high_level(wat: &str) -> HighLevelBody {
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let (module, _, _) = wasabi_wasm::Module::from_bytes(&wasm_bytes).unwrap();
    let function = module.function(0_usize.into());
    let code = function.code().unwrap();
    HighLevelBody::try_from((&module, function, code)).unwrap()
}

fn assert_high_and_low(
    wasm_program: &'static str,
    body_expected_low: LowLevelBody,
    body_expected_high: HighLevelBody,
) {
    // Assert expected high-level program
    let high_level_body = wat_to_high_level(wasm_program);
    assert_eq!(high_level_body, body_expected_high);
    // Assert expected low-level program
    let low_level_body: LowLevelBody = high_level_body.into();
    assert_eq!(low_level_body, body_expected_low);
}

// TODO: macro's for huge bodies?

#[test]
fn high_level_low_level_assert() {
    assert_high_and_low(
        include_str!("branch.wat"),
        {
            use wasabi_wasm::{Instr::*, Val::I32};
            LowLevelBody(vec![
                Const(I32(0)),
                If(typ_void_to_i32()),
                Const(I32(1)),
                Else,
                Const(I32(2)),
                End,
                Drop,
                End,
            ])
        },
        HighLevelBody(vec![
            TypedHighLevelInstr::new(0, typ_void_to_i32(), const_i32(0)),
            TypedHighLevelInstr::new(
                1,
                typ_void_to_i32(),
                HighLevelInstr::if_then_else(
                    typ_void_to_i32(),
                    vec![TypedHighLevelInstr::new(2, typ_void_to_i32(), const_i32(1))],
                    vec![TypedHighLevelInstr::new(4, typ_void_to_i32(), const_i32(2))],
                ),
            ),
            TypedHighLevelInstr::new(6, typ_i32_to_void(), HighLevelInstr::Drop),
        ]),
    );
}

#[test]
fn test_wat_to_high_level_complex() {
    assert_high_and_low(
        include_str!("nested-branch.wat"),
        {
            use wasabi_wasm::{Instr::*, Val::I32};
            LowLevelBody(vec![
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
            ])
        },
        {
            use super::HighLevelInstr::Drop;
            let if_res_i32 =
                |then, else_| HighLevelInstr::if_then_else(typ_void_to_i32(), then, else_);
            HighLevelBody(vec![
                TypedHighLevelInstr::new(0, typ_void_to_i32(), const_i32(1000)),
                TypedHighLevelInstr::new(
                    1,
                    typ_void_to_i32(),
                    if_res_i32(
                        vec![
                            TypedHighLevelInstr::new(
                                2,
                                typ_void_to_i32(),
                                const_i32(1001),
                            ),
                            TypedHighLevelInstr::new(
                                3,
                                typ_void_to_i32(),
                                if_res_i32(
                                    vec![
                                        TypedHighLevelInstr::new(
                                            4,
                                            typ_void_to_i32(),
                                            const_i32(1002),
                                        ),
                                        TypedHighLevelInstr::new(
                                            5,
                                            typ_void_to_i32(),
                                            if_res_i32(
                                                vec![TypedHighLevelInstr::new(
                                                    6,
                                                    typ_void_to_i32(),
                                                    const_i32(0),
                                                )],
                                                vec![TypedHighLevelInstr::new(
                                                    8,
                                                    typ_void_to_i32(),
                                                    const_i32(1),
                                                )],
                                            ),
                                        ),
                                    ],
                                    vec![
                                        TypedHighLevelInstr::new(
                                            11,
                                            typ_void_to_i32(),
                                            const_i32(1003),
                                        ),
                                        TypedHighLevelInstr::new(
                                            12,
                                            typ_void_to_i32(),
                                            if_res_i32(
                                                vec![TypedHighLevelInstr::new(
                                                    13,
                                                    typ_void_to_i32(),
                                                    const_i32(2),
                                                )],
                                                vec![TypedHighLevelInstr::new(
                                                    15,
                                                    typ_void_to_i32(),
                                                    const_i32(3),
                                                )],
                                            ),
                                        ),
                                    ],
                                ),
                            ),
                            TypedHighLevelInstr::new(18, typ_i32_to_void(), Drop),
                            TypedHighLevelInstr::new(
                                19,
                                typ_void_to_i32(),
                                const_i32(1004),
                            ),
                            TypedHighLevelInstr::new(
                                20,
                                typ_void_to_i32(),
                                if_res_i32(
                                    vec![TypedHighLevelInstr::new(
                                        21,
                                        typ_void_to_i32(),
                                        const_i32(4),
                                    )],
                                    vec![
                                        TypedHighLevelInstr::new(
                                            23,
                                            typ_void_to_i32(),
                                            const_i32(1005),
                                        ),
                                        TypedHighLevelInstr::new(
                                            24,
                                            typ_void_to_i32(),
                                            if_res_i32(
                                                vec![TypedHighLevelInstr::new(
                                                    25,
                                                    typ_void_to_i32(),
                                                    const_i32(5),
                                                )],
                                                vec![TypedHighLevelInstr::new(
                                                    27,
                                                    typ_void_to_i32(),
                                                    const_i32(6),
                                                )],
                                            ),
                                        ),
                                    ],
                                ),
                            ),
                        ],
                        vec![
                            TypedHighLevelInstr::new(
                                31,
                                typ_void_to_i32(),
                                const_i32(1006),
                            ),
                            TypedHighLevelInstr::new(
                                32,
                                typ_void_to_i32(),
                                if_res_i32(
                                    vec![
                                        TypedHighLevelInstr::new(
                                            33,
                                            typ_void_to_i32(),
                                            const_i32(1007),
                                        ),
                                        TypedHighLevelInstr::new(
                                            34,
                                            typ_void_to_i32(),
                                            if_res_i32(
                                                vec![
                                                    TypedHighLevelInstr::new(
                                                        35,
                                                        typ_void_to_i32(),
                                                        const_i32(1008),
                                                    ),
                                                    TypedHighLevelInstr::new(
                                                        36,
                                                        typ_void_to_i32(),
                                                        if_res_i32(
                                                            vec![TypedHighLevelInstr::new(37, typ_void_to_i32(),const_i32(7))],
                                                            vec![TypedHighLevelInstr::new(39, typ_void_to_i32(),const_i32(8))],
                                                        ),
                                                    ),
                                                ],
                                                vec![TypedHighLevelInstr::new(42,typ_void_to_i32(), const_i32(9))],
                                            ),
                                        ),
                                    ],
                                    vec![
                                        TypedHighLevelInstr::new(45, typ_void_to_i32(),const_i32(1009)),
                                        TypedHighLevelInstr::new(
                                            46,
                                            typ_void_to_i32(),
                                            if_res_i32(
                                                vec![
                                                    TypedHighLevelInstr::new(47, typ_void_to_i32(),const_i32(1010)),
                                                    TypedHighLevelInstr::new(
                                                        48,
                                                        typ_void_to_i32(),
                                                        if_res_i32(
                                                            vec![
                                                                TypedHighLevelInstr::new(49, typ_void_to_i32(),const_i32(1011)),
                                                                TypedHighLevelInstr::new(
                                                                    50,
                                                                    typ_void_to_i32(),
                                                                    if_res_i32(
                                                                        vec![TypedHighLevelInstr::new(51,typ_void_to_i32(), const_i32(10))],
                                                                        vec![TypedHighLevelInstr::new(53, typ_void_to_i32(),const_i32(11))],
                                                                    ),
                                                                ),
                                                            ],
                                                            vec![TypedHighLevelInstr::new(56,typ_void_to_i32(), const_i32(12))],
                                                        ),
                                                    ),
                                                ],
                                                vec![TypedHighLevelInstr::new(59, typ_void_to_i32(),const_i32(13))],
                                            ),
                                        ),
                                    ],
                                ),
                            ),
                            TypedHighLevelInstr::new(62, typ_i32_to_void(), Drop),
                            TypedHighLevelInstr::new(63, typ_void_to_i32(),const_i32(1012)),
                            TypedHighLevelInstr::new(
                                64,
                                typ_void_to_i32(),
                                if_res_i32(vec![TypedHighLevelInstr::new(65,typ_void_to_i32(), const_i32(14))], vec![TypedHighLevelInstr::new(67, typ_void_to_i32(), const_i32(15))]),
                            ),
                        ],
                    ),
                ),
                TypedHighLevelInstr::new(70, typ_i32_to_void(), Drop),
            ])
        },
    );
}
