use crate::parse_nesting::{
    BodyInner, HighLevelBody, HighLevelInstr as Instr, TypedHighLevelInstr,
};
use generate_branch_table::{BrTable, Reified};
use wasabi_wasm::{Function, FunctionType, Idx, Module, Val, ValType};

use super::TransformationStrategy;

mod generate_branch_table;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Target {
    IfThen(Idx<Function>),
    IfThenPost(Idx<Function>),
    IfThenElse(Idx<Function>),
    IfThenElsePost(Idx<Function>),
    Br(Idx<Function>),
    BrIf(Idx<Function>),
    BrTable(Idx<Function>),
}

// Number of constant instructions in transformation
const TRANSFORM_COST_PER_IF_THEN_INSTR: usize = 1;
const TRANSFORM_COST_PER_IF_THEN_ELSE_INSTR: usize = 1;
const TRANSFORM_COST_PER_BR_IF: usize = 1;

fn delta_to_instrument_instr(index_instr: &TypedHighLevelInstr) -> usize {
    let TypedHighLevelInstr { instr, .. } = index_instr;
    match instr {
        Instr::If(_, _, None) => TRANSFORM_COST_PER_IF_THEN_INSTR,
        Instr::If(_, _, Some(_)) => TRANSFORM_COST_PER_IF_THEN_ELSE_INSTR,
        Instr::BrIf(_) => TRANSFORM_COST_PER_BR_IF,
        _ => 0,
    }
}

// TODO: room for optimization - compute delta_to_instrument_body

impl TransformationStrategy for Target {
    fn transform(&self, high_level_body: &HighLevelBody, module: &mut Module) -> HighLevelBody {
        let HighLevelBody(body) = high_level_body;
        let transformed_body = transform(body, *self, module);
        HighLevelBody(transformed_body)
    }
}

/// # Panics
/// When the index cannot be cast from u32 to i32
fn transform(body: &BodyInner, target: Target, module: &mut Module) -> BodyInner {
    let mut result: Vec<TypedHighLevelInstr> =
        Vec::with_capacity(body.iter().map(delta_to_instrument_instr).sum::<usize>() + body.len());

    for typed_instr @ TypedHighLevelInstr { instr, .. } in body {
        if typed_instr.is_uninstrumented() {
            match (target, instr) {
                (Target::Br(br_trap_idx), Instr::Br(label)) => {
                    result.extend_from_slice(&[
                        // STACK: []
                        typed_instr.instrument_with(Instr::Const(Val::I64(label.to_u32().into()))),
                        // STACK: [label]
                        typed_instr.instrument_with(Instr::Call(br_trap_idx)),
                        // STACK: [table_target_index]
                        typed_instr.place_original(instr.clone()),
                    ]);
                    continue;
                }
                (Target::BrTable(br_table_trap_idx), Instr::BrTable { table, default }) => {
                    let reified_branch_table = BrTable {
                        table: table.to_vec(),
                        default: *default,
                    }
                    .reify();
                    let index_to_index_and_effective_target = module.add_function(
                        FunctionType::new(&[ValType::I32], &[ValType::I32, ValType::I32]),
                        reified_branch_table
                            .locals
                            .iter()
                            .map(|l| l.type_)
                            .collect(),
                        reified_branch_table.body,
                    );
                    result.extend_from_slice(&[
                        // STACK: [table_target_index]
                        typed_instr
                            .instrument_with(Instr::Call(index_to_index_and_effective_target)),
                        // STACK: [table_target_index, runtime_label]
                        typed_instr.instrument_with(Instr::Const(Val::I32(
                            i32::try_from(default.to_u32()).expect("i32->u32"),
                        ))),
                        // STACK: [table_target_index, runtime_label, default]
                        typed_instr.instrument_with(Instr::Call(br_table_trap_idx)),
                        // STACK: [table_target_index]
                        typed_instr.place_original(instr.clone()),
                    ]);
                    continue;
                }
                (Target::IfThen(if_then_trap_idx), Instr::If(type_, then, None)) => {
                    result.extend_from_slice(&[
                        // STACK: [type_in, condition]
                        typed_instr.instrument_with(Instr::Const(Val::I32(
                            type_.inputs().len().try_into().unwrap(),
                        ))),
                        // STACK: [type_in, condition, inputs-len:i32]
                        typed_instr.instrument_with(Instr::Const(Val::I32(
                            type_.results().len().try_into().unwrap(),
                        ))),
                        // STACK: [type_in, condition, inputs-len:i32, results-len:i32]
                        typed_instr.instrument_with(Instr::Call(if_then_trap_idx)),
                        // STACK: [type_in, kontinuation]
                        typed_instr.place_original(Instr::if_then(
                            *type_,
                            transform(then, target, module),
                        )),
                        // STACK: [type_out]
                    ]);
                    continue;
                }
                (Target::IfThenPost(if_then_post_trap_idx), Instr::If(type_, then, None)) => {
                    // STACK: [type_in, continuation]
                    let mut injected_then_body = transform(then, target, module);
                    // append to rest of body
                    injected_then_body
                        .push(typed_instr.instrument_with(Instr::Call(if_then_post_trap_idx)));
                    // STACK: [type_in, continuation]
                    let injected_else_body =
                        vec![typed_instr.instrument_with(Instr::Call(if_then_post_trap_idx))];
                    // original instruction
                    result.extend_from_slice(&[
                        // STACK: [type_in, continuation]
                        // FIXME: this is DANGEROUS!
                        //        reason: this implicitly requires the `if_then` to be instrumented
                        //        before the `if_then_post`, since the `if_then` keeps the uninstrumented instruction
                        //        but the `if_then_post` replaces the instruction after which the `if_then` could not
                        //        find its target instruction anymore ...
                        typed_instr.instrument_with(Instr::if_then_else(
                            *type_,
                            injected_then_body,
                            injected_else_body,
                        )),
                        // STACK: [type_out]
                    ]);
                    continue;
                }
                (
                    Target::IfThenElse(if_then_else_trap_idx),
                    Instr::If(type_, then, Some(else_)),
                ) => {
                    result.extend_from_slice(&[
                        // STACK: [type_in, condition]
                        typed_instr.instrument_with(Instr::Const(Val::I32(
                            type_.inputs().len().try_into().unwrap(),
                        ))),
                        // STACK: [type_in, condition, inputs-len:i32]
                        typed_instr.instrument_with(Instr::Const(Val::I32(
                            type_.results().len().try_into().unwrap(),
                        ))),
                        // STACK: [type_in, condition, inputs-len:i32, results-len:i32]
                        typed_instr.instrument_with(Instr::Call(if_then_else_trap_idx)),
                        // STACK: [type_in, kontinuation]
                        typed_instr.place_original(Instr::if_then_else(
                            *type_,
                            // STACK: [type_in]
                            transform(then, target, module),
                            // STACK: [type_in]
                            transform(else_, target, module),
                        )),
                        // STACK: [type_out]
                    ]);
                    continue;
                }
                (
                    Target::IfThenElsePost(if_then_else_post_trap_idx),
                    Instr::If(type_, then, Some(else_)),
                ) => {
                    // Inject into then-body
                    let mut injected_then_body = transform(then, target, module);
                    // append to rest of body
                    injected_then_body
                        .push(typed_instr.instrument_with(Instr::Call(if_then_else_post_trap_idx)));
                    // Inject into else-body
                    let mut injected_else_body = transform(else_, target, module);
                    // append to rest of body
                    injected_else_body
                        .push(typed_instr.instrument_with(Instr::Call(if_then_else_post_trap_idx)));

                    // Original body
                    result.extend_from_slice(&[
                        // STACK: [type_in, continuation]
                        typed_instr.place_original(Instr::if_then_else(
                            *type_,
                            injected_then_body,
                            injected_else_body,
                        )),
                        // STACK: [type_out]
                    ]);
                    continue;
                }
                (Target::BrIf(br_if_trap_idx), Instr::BrIf(label)) => {
                    result.extend_from_slice(&[
                        // STACK: [condition]
                        typed_instr.instrument_with(Instr::Const(Val::I32(
                            i32::try_from(label.to_u32()).unwrap(),
                        ))),
                        // STACK: [condition, label]
                        typed_instr.instrument_with(Instr::Call(br_if_trap_idx)),
                        // STACK: [kontinuation]
                        typed_instr.place_original(instr.clone()),
                        // STACK: []
                    ]);
                    continue;
                }
                _ => (),
            }
        }

        // DEFAULT TRAVERSAL
        match (target, instr) {
            (target, Instr::If(type_, then, None)) => {
                result.push(typed_instr.place_untouched(Instr::If(
                    *type_,
                    transform(then, target, module),
                    None,
                )));
            }
            (target, Instr::If(type_, then, Some(else_))) => {
                result.push(typed_instr.place_untouched(Instr::If(
                    *type_,
                    transform(then, target, module),
                    Some(transform(else_, target, module)),
                )))
            }
            (target, Instr::Loop(type_, body)) => {
                result.push(
                    typed_instr
                        .place_untouched(Instr::Loop(*type_, transform(body, target, module))),
                );
            }
            (target, Instr::Block(type_, body)) => {
                result.push(
                    typed_instr
                        .place_untouched(Instr::Block(*type_, transform(body, target, module))),
                );
            }
            (_, instr) => result.push(typed_instr.place_untouched(instr.clone())),
        }
    }
    result
}

#[cfg(test)]
mod tests {

    const THEN_KONTN: i32 = 1;
    const ELSE_KONTN: i32 = 0;

    use wasabi_wasm::types::InferredInstructionType;
    use wasabi_wasm::{FunctionType, Module, ValType};

    use crate::analysis::{AnalysisInterface, WasmExport};
    use crate::parse_nesting::{LowLevelBody, LowToHighError};

    use super::*;

    fn new_typed_high_level(
        index: usize,
        type_: FunctionType,
        instr: super::Instr,
    ) -> TypedHighLevelInstr {
        TypedHighLevelInstr::new_uninstrumented(
            index,
            InferredInstructionType::Reachable(type_),
            instr,
        )
    }

    struct BranchExpectation {
        input: i32,
        output: i32,
    }

    macro_rules! branch_expect {
        (IN: $i:literal -> OUT: $o:literal) => {
            BranchExpectation {
                input: $i,
                output: $o,
            }
        };
    }

    #[test]
    fn test_wat_branch_simple() {
        use wasabi_wasm::{Instr::*, Val};
        const SIMPLE_BRANCH: &str = r#"
        (module
          (export "main" (func $main))
          (type $void=>i32 (func (result i32)))

        ;; ====================================== ;;
          ;; let main a = if a == 0 then 1 else 2
          (func $main (param $a i32) (result i32)
            (if (type $void=>i32)
              (i32.eq (i32.const 0) (local.get $a))
              (then (i32.const 1))
              (else (i32.const 2))))
        ;; ====================================== ;;

        )"#;
        let uninstrumented_expectation = &[
            branch_expect!(IN: 0   -> OUT: 1),
            branch_expect!(IN: 1   -> OUT: 2),
            branch_expect!(IN: 2   -> OUT: 2),
            branch_expect!(IN: 999 -> OUT: 2),
        ];

        for (analysis_body, instrumented_assertions) in [
            (
                vec![Const(Val::I32(ELSE_KONTN)), End],
                [
                    branch_expect!(IN: 0   -> OUT: 2),
                    branch_expect!(IN: 1   -> OUT: 2),
                    branch_expect!(IN: 2   -> OUT: 2),
                    branch_expect!(IN: 999 -> OUT: 2),
                ],
            ),
            (
                vec![Const(Val::I32(THEN_KONTN)), End],
                [
                    branch_expect!(IN: 0   -> OUT: 1),
                    branch_expect!(IN: 1   -> OUT: 1),
                    branch_expect!(IN: 2   -> OUT: 1),
                    branch_expect!(IN: 999 -> OUT: 1),
                ],
            ),
        ] {
            test_branch_instrumentation_wat(
                SIMPLE_BRANCH,
                uninstrumented_expectation,
                analysis_body,
                &instrumented_assertions,
            )
        }
    }

    #[test]
    fn test_wat_branch_complex() {
        use wasabi_wasm::{Instr::*, Val};
        const SIMPLE_BRANCH: &str = r#"
        (module
          (export "main" (func $main))
          (type $void=>i32 (func (result i32)))

        ;; ====================================== ;;
          ;; let main a = if a == 1 then 1 else (if a == 2 then 2 else (if a == 3 then 3 else 4))
          (func $main (param $a i32) (result i32)
            (if (type $void=>i32)
              (i32.eq (i32.const 1) (local.get $a))
              (then (i32.const 1))
              (else
                (if (type $void=>i32)
                  (i32.eq (i32.const 2) (local.get $a))
                  (then (i32.const 2))
                  (else
                    (if (type $void=>i32)
                      (i32.eq (i32.const 3) (local.get $a))
                      (then (i32.const 3))
                      (else (i32.const 4))))))))
        ;; ====================================== ;;

        )"#;
        let uninstrumented_expectation = &[
            branch_expect!(IN: 1 -> OUT: 1),
            branch_expect!(IN: 2 -> OUT: 2),
            branch_expect!(IN: 3 -> OUT: 3),
            branch_expect!(IN: 4 -> OUT: 4), // from 4 only 4
            branch_expect!(IN: 5 -> OUT: 4),
            branch_expect!(IN: 6 -> OUT: 4),
            branch_expect!(IN: 0 -> OUT: 4),
        ];

        for (analysis_body, instrumented_assertions) in [
            (
                vec![Const(Val::I32(ELSE_KONTN)), End],
                [
                    branch_expect!(IN: 1 -> OUT: 4), // always 4
                    branch_expect!(IN: 2 -> OUT: 4),
                    branch_expect!(IN: 3 -> OUT: 4),
                    branch_expect!(IN: 4 -> OUT: 4),
                    branch_expect!(IN: 5 -> OUT: 4),
                    branch_expect!(IN: 6 -> OUT: 4),
                    branch_expect!(IN: 0 -> OUT: 4),
                ],
            ),
            (
                vec![Const(Val::I32(THEN_KONTN)), End],
                [
                    branch_expect!(IN: 1 -> OUT: 1), // always 1
                    branch_expect!(IN: 2 -> OUT: 1),
                    branch_expect!(IN: 3 -> OUT: 1),
                    branch_expect!(IN: 4 -> OUT: 1),
                    branch_expect!(IN: 5 -> OUT: 1),
                    branch_expect!(IN: 6 -> OUT: 1),
                    branch_expect!(IN: 0 -> OUT: 1),
                ],
            ),
        ] {
            test_branch_instrumentation_wat(
                SIMPLE_BRANCH,
                uninstrumented_expectation,
                analysis_body,
                &instrumented_assertions,
            )
        }
    }

    fn test_branch_instrumentation_wat(
        branch_program_wasm: &str,
        uninstrumented_assertions: &[BranchExpectation],
        instrumentation_body: Vec<wasabi_wasm::Instr>,
        instrumented_assertions: &[BranchExpectation],
    ) {
        use wasmtime::{Engine, Instance, Module, Store};
        // For execution
        let wasm_bytes = wat::parse_str(branch_program_wasm).unwrap();
        let (mut wasm_module, _, _) = wasabi_wasm::Module::from_bytes(&wasm_bytes).unwrap();

        let assert_outcome = |module: &wasabi_wasm::Module, assertions, instrumentation_body| {
            let engine = Engine::default();
            let mut store = Store::new(&engine, ());
            let module = Module::new(&engine, module.to_bytes().unwrap()).unwrap();
            let instance = Instance::new(&mut store, &module, &[]).unwrap();
            let main = instance
                .get_typed_func::<i32, i32>(&mut store, "main")
                .unwrap();

            for &BranchExpectation { input, output } in assertions {
                assert_eq!(
                    main.call(&mut store, input).unwrap(),
                    output,
                    "With input {input} the result (l) does not match the expection (r) for instrumentation: {instrumentation_body:?}"
                );
            }
        };

        // Execute uninstrumented:
        assert_outcome(
            &wasm_module,
            uninstrumented_assertions,
            &instrumentation_body,
        );

        fn to_wasm_type(valtype: crate::analysis::WasmType) -> ValType {
            match valtype {
                crate::analysis::WasmType::F32 => ValType::F32,
                crate::analysis::WasmType::I32 => ValType::I32,
                crate::analysis::WasmType::F64 => ValType::F64,
                crate::analysis::WasmType::I64 => ValType::I64,
            }
        }

        let WasmExport { args, results, .. } = AnalysisInterface::interface_if_then_else();
        let args: Vec<ValType> = args.into_iter().map(to_wasm_type).collect();
        let results: Vec<ValType> = results.into_iter().map(to_wasm_type).collect();

        // Instrument
        let if_then_else_trap_idx = wasm_module.add_function(
            FunctionType::new(&args, &results),
            vec![],
            instrumentation_body.clone(),
        );

        let index = 0_u32.into();
        let function = wasm_module.function(index);
        let code = function.code().unwrap();
        let high_level_body: HighLevelBody =
            (&wasm_module, function, code, &index).try_into().unwrap();
        let transformed =
            Target::IfThenElse(if_then_else_trap_idx).transform(&high_level_body, &mut wasm_module);

        let LowLevelBody(low_level_body) = LowLevelBody::from(transformed);
        wasm_module.function_mut(index).code_mut().unwrap().body = low_level_body;

        // Execute instrumented:
        assert_outcome(&wasm_module, instrumented_assertions, &instrumentation_body);
    }

    #[test]
    fn test_faulty_pre_instrumentation() {
        const MINI_PROGRAM: &str = r#"
        (module
          (func $foo
            (i32.const 0)
            (if
              (then (i32.const 1))
              (else (i32.const 2)))))"#;

        let wasm_bytes = wat::parse_str(MINI_PROGRAM).unwrap();
        let (mut wasm_module, _, _) = wasabi_wasm::Module::from_bytes(&wasm_bytes).unwrap();

        // this 'mimics' that instrumentation before has happened
        // thus this faulty transformation is caught here
        wasm_module
            .function_mut(0_usize.into())
            .code_mut()
            .unwrap()
            .body
            .push(wasabi_wasm::Instr::Call(0_usize.into()));

        let index = 0_u32.into();
        let function = wasm_module.function(index);
        let code = function.code().unwrap();

        assert!(matches!(
            HighLevelBody::try_from((&wasm_module, function, code, &index)),
            Err(LowToHighError::TypeInference { .. }),
        ))
    }

    fn nested_ifs_body() -> Module {
        const NESTED_PROGRAM: &str = r#"
        (module
          (func $main (param i32) (result i32)
            (if (result i32)
              (i32.eqz (local.get 0))
              (then
                (block (result i32)
                  (if (result i32)
                    (i32.eq (local.get 0) (i32.const 1))
                    (then
                      (block (result i32)
                        (if (result i32)
                          (i32.eq (local.get 0) (i32.const 2))
                          (then
                            (block (result i32)
                              (i32.const 42)))
                          (else
                            (i32.const 21)))))
                    (else
                      (i32.const 10)))))
              (else
                (i32.const 5)
              )
            )
          )
          (export "nestedIfs" (func $main)))"#;

        let wasm_bytes = wat::parse_str(NESTED_PROGRAM).unwrap();
        let (wasm_module, _, _) = wasabi_wasm::Module::from_bytes(&wasm_bytes).unwrap();
        wasm_module
    }

    fn typ_void_to_i32() -> FunctionType {
        FunctionType::new(&[], &[ValType::I32])
    }

    fn typ_i32_to_i32() -> FunctionType {
        FunctionType::new(&[ValType::I32], &[ValType::I32])
    }

    fn typ_i32_i32_to_i32() -> FunctionType {
        FunctionType::new(&[ValType::I32, ValType::I32], &[ValType::I32])
    }

    #[test]
    fn test_nested_ifs() {
        let wasm_module = nested_ifs_body();

        let index = 0_usize.into();
        let function: &Function = wasm_module.function(index);
        let code = function.code().unwrap();

        assert_eq!(
            HighLevelBody::try_from((&wasm_module, function, code, &index)).unwrap(),
            {
                use super::Instr::*;
                use wasabi_wasm::{BinaryOp::*, LocalOp::*, UnaryOp::*, Val::*};

                HighLevelBody(vec![
                    new_typed_high_level(0, typ_void_to_i32(), Local(Get, 0_usize.into())),
                    new_typed_high_level(1, typ_i32_to_i32(), Unary(I32Eqz)),
                    new_typed_high_level(
                        2,
                        typ_void_to_i32(),
                        If(
                            typ_void_to_i32(),
                            vec![new_typed_high_level(
                                3,
                                typ_void_to_i32(),
                                Block(
                                    typ_void_to_i32(),
                                    vec![
                                        new_typed_high_level(
                                            4,
                                            typ_void_to_i32(),
                                            Local(Get, 0_usize.into()),
                                        ),
                                        new_typed_high_level(5, typ_void_to_i32(), Const(I32(1))),
                                        new_typed_high_level(6, typ_i32_i32_to_i32(), Binary(I32Eq)),
                                        new_typed_high_level(
                                            7,
                                            typ_void_to_i32(),
                                            If(
                                                typ_void_to_i32(),
                                                vec![new_typed_high_level(
                                                    8,
                                                    typ_void_to_i32(),
                                                    Block(
                                                        typ_void_to_i32(),
                                                        vec![
                                                            new_typed_high_level(
                                                                9,
                                                                typ_void_to_i32(),
                                                                Local(Get, 0_usize.into()),
                                                            ),
                                                            new_typed_high_level(
                                                                10,
                                                                typ_void_to_i32(),
                                                                Const(I32(2)),
                                                            ),
                                                            new_typed_high_level(
                                                                11,
                                                                typ_i32_i32_to_i32(),
                                                                Binary(I32Eq),
                                                            ),
                                                            new_typed_high_level(
                                                                12,
                                                                typ_void_to_i32(),
                                                                If(
                                                                    typ_void_to_i32(),
                                                                    vec![new_typed_high_level(
                                                                        13,
                                                                        typ_void_to_i32(),
                                                                        Block(
                                                                            typ_void_to_i32(),
                                                                            vec![new_typed_high_level(
                                                                                14, typ_void_to_i32(),
                                                                                Const(I32(42)),
                                                                            )],
                                                                        ),
                                                                    )],
                                                                    Some(vec![new_typed_high_level(
                                                                        17,typ_void_to_i32(),
                                                                        Const(I32(21)),
                                                                    )]),
                                                                ),
                                                            ),
                                                        ],
                                                    ),
                                                )],
                                                Some(vec![new_typed_high_level(21, typ_void_to_i32(), Const(I32(10)))]),
                                            ),
                                        ),
                                    ],
                                ),
                            )],
                            Some(vec![new_typed_high_level(25, typ_void_to_i32(),Const(I32(5)))]),
                        ),
                    ),
                ])
            },
        )
    }

    #[test]
    fn test_target() {
        let target = Target::IfThen(0_usize.into());
        assert_eq!(target.clone(), target);
        assert_eq!(format!("{target:?}"), "IfThen(Function 0)");
    }
}
