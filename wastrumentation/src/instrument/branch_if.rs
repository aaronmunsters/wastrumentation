use crate::parse_nesting::{HighLevelBody, Instr};
use wasabi_wasm::{Function, Idx, Val};

use super::TransformationStrategy;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Target {
    IfThen(Idx<Function>),
    IfThenElse(Idx<Function>),
    BrIf(Idx<Function>),
    BrTable(Idx<Function>),
}

// Amount of constant instructions in transformation
const TRANSFORM_COST_PER_IF_THEN_INSTR: usize = 1;
const TRANSFORM_COST_PER_IF_THEN_ELSE_INSTR: usize = 1;
const TRANSFORM_COST_PER_BR_IF: usize = 1;

fn delta_to_instrument_instr(instr: &Instr) -> usize {
    match instr {
        Instr::If(_, _, None) => TRANSFORM_COST_PER_IF_THEN_INSTR,
        Instr::If(_, _, Some(_)) => TRANSFORM_COST_PER_IF_THEN_ELSE_INSTR,
        Instr::BrIf(_) => TRANSFORM_COST_PER_BR_IF,
        _ => 0,
    }
}

#[allow(dead_code)] /* TODO: */
fn delta_to_instrument_body(body: &[Instr]) -> usize {
    let res = body
        .iter()
        .map(|instr| -> usize {
            delta_to_instrument_instr(instr)
                + match instr {
                    Instr::Loop(_, body) | Instr::Block(_, body) | Instr::If(_, body, None) => {
                        delta_to_instrument_body(body)
                    }
                    Instr::If(_, then, Some(els)) => {
                        delta_to_instrument_body(then) + delta_to_instrument_body(els)
                    }
                    _ => 0,
                }
        })
        .sum();
    res
}

impl TransformationStrategy for Target {
    fn transform(&self, high_level_body: &HighLevelBody) -> HighLevelBody {
        let HighLevelBody(body) = high_level_body;
        let transformed_body = HighLevelBody::transform_branch_inner(body, *self);
        HighLevelBody(transformed_body)
    }
}

impl HighLevelBody {
    /// # Panics
    /// When the index cannot be cast from u32 to i32
    pub fn transform_branch_inner(body: &Vec<Instr>, target: Target) -> Vec<Instr> {
        let mut result = Vec::with_capacity(
            body.iter().map(delta_to_instrument_instr).sum::<usize>() + body.len(),
        );

        for instr in body {
            match (target, instr) {
                (Target::BrTable(br_table_trap_idx), Instr::BrTable { table: _, default }) => {
                    result.extend_from_slice(&[
                        // STACK: [table_target_index]
                        Instr::Const(Val::I32(i32::try_from(default.to_u32()).expect("i32->u32"))),
                        // STACK: [table_target_index, default]
                        Instr::Call(br_table_trap_idx),
                        // STACK: [table_target_index]
                        instr.clone(),
                    ]);
                }
                (Target::IfThen(if_then_trap_idx), Instr::If(type_, then, None)) => result
                    .extend_from_slice(&[
                        // STACK: [type_in, condition]
                        Instr::Call(if_then_trap_idx),
                        // STACK: [type_in, kontinuation]
                        Instr::if_then(*type_, Self::transform_branch_inner(then, target)),
                        // STACK: [type_out]
                    ]),
                (
                    Target::IfThenElse(if_then_else_trap_idx),
                    Instr::If(type_, then, Some(else_)),
                ) => result.extend_from_slice(&[
                    // STACK: [type_in, condition]
                    Instr::Call(if_then_else_trap_idx),
                    // STACK: [type_in, kontinuation]
                    Instr::if_then_else(
                        *type_,
                        // STACK: [type_in]
                        Self::transform_branch_inner(then, target),
                        // STACK: [type_in]
                        Self::transform_branch_inner(else_, target),
                    ),
                    // STACK: [type_out]
                ]),
                (Target::BrIf(br_if_trap_idx), Instr::BrIf(label)) => {
                    result.extend_from_slice(&[
                        // STACK: [condition]
                        Instr::Const(Val::I32(i32::try_from(label.to_u32()).unwrap())),
                        // STACK: [condition, label]
                        Instr::Call(br_if_trap_idx),
                        // STACK: [kontinuation]
                        instr.clone(),
                        // STACK: []
                    ]);
                }

                (target, Instr::Loop(type_, body)) => result.push(Instr::Loop(
                    *type_,
                    Self::transform_branch_inner(body, target),
                )),
                (target, Instr::Block(type_, body)) => result.push(Instr::Block(
                    *type_,
                    Self::transform_branch_inner(body, target),
                )),

                _ => result.push(instr.clone()),
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    const THEN_KONTN: i32 = 1;
    const ELSE_KONTN: i32 = 0;

    use std::collections::HashSet;

    use wasabi_wasm::{FunctionType, Val, ValType};
    use wasmtime::{Engine, Instance, Module, Store};

    use crate::{instrument::Instrumentable, parse_nesting::LowLevelBody};

    use super::*;

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
        // For execution
        let wasm_bytes = wasmer::wat2wasm(branch_program_wasm.as_bytes()).unwrap();
        let (mut wasm_module, _, _) = wasabi_wasm::Module::from_bytes(&wasm_bytes).unwrap();

        let assert_outcome = |module: &wasabi_wasm::Module, assertions| {
            let engine = Engine::default();
            let mut store = Store::new(&engine, ());
            let module = Module::new(&engine, module.to_bytes().unwrap()).unwrap();
            let instance = Instance::new(&mut store, &module, &[]).unwrap();
            let main = instance
                .get_typed_func::<i32, i32>(&mut store, "main")
                .unwrap();

            for &BranchExpectation { input, output } in assertions {
                assert_eq!(main.call(&mut store, input).unwrap(), output);
            }
        };

        // Execute uninstrumented:
        assert_outcome(&wasm_module, uninstrumented_assertions);

        // Instrument
        let if_then_else_trap_idx = wasm_module.add_function(
            FunctionType::new(&[ValType::I32], &[ValType::I32]),
            vec![],
            instrumentation_body,
        );

        wasm_module
            .instrument_function_bodies(
                &HashSet::from_iter(vec![0_usize.into()]),
                &Target::IfThenElse(if_then_else_trap_idx),
            )
            .unwrap();

        // Execute instrumented:
        assert_outcome(&wasm_module, instrumented_assertions);
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

        let wasm_bytes = wasmer::wat2wasm(MINI_PROGRAM.as_bytes()).unwrap();
        let (mut wasm_module, _, _) = wasabi_wasm::Module::from_bytes(&wasm_bytes).unwrap();

        // this 'mimics' that instrumentation before has happened
        // thus this faulty transformation is caught here
        wasm_module
            .function_mut(0_usize.into())
            .code_mut()
            .unwrap()
            .body
            .push(wasabi_wasm::Instr::Call(0_usize.into()));

        assert_eq!(
            Err("Expected low level body to terminate in `End`"),
            wasm_module.instrument_function_bodies(
                &HashSet::from_iter(vec![0_usize.into()]),
                &Target::IfThenElse(0_usize.into())
            )
        );
    }

    #[test]
    fn test_faulty_import_instrumentation() {
        const MINI_PROGRAM: &str = r#"
        (module
          (func $foo (import "bar" "foo")))"#;

        let wasm_bytes = wasmer::wat2wasm(MINI_PROGRAM.as_bytes()).unwrap();
        let (mut wasm_module, _, _) = wasabi_wasm::Module::from_bytes(&wasm_bytes).unwrap();

        // Instrument
        assert_eq!(
            Err("Attempt to instrument an `import` function"),
            wasm_module.instrument_function_bodies(
                &HashSet::from_iter(vec![0_usize.into()]),
                &Target::IfThenElse(0_usize.into())
            )
        );
    }

    fn nested_ifs_body() -> (wasabi_wasm::Module, Vec<wasabi_wasm::Instr>) {
        const NESTED_PROGRAM: &str = r#"
        (module
          (func $main (param i32) (result i32)
            (if (i32.eqz (local.get 0))
              (then
                (block
                  (if (i32.eq (local.get 0) (i32.const 1))
                    (then (block
                      (if (i32.eq (local.get 0) (i32.const 2))
                        (then (block
                          (i32.const 42)))
                        (else (i32.const 21)))))
                    (else (i32.const 10)))))
              (else
                (i32.const 5))))
          (export "nestedIfs" (func $main)))"#;

        let wasm_bytes = wasmer::wat2wasm(NESTED_PROGRAM.as_bytes()).unwrap();
        let (wasm_module, _, _) = wasabi_wasm::Module::from_bytes(&wasm_bytes).unwrap();

        let body = wasm_module
            .function(0_usize.into())
            .code()
            .unwrap()
            .body
            .clone();

        (wasm_module, body)
    }

    #[test]
    fn test_nested_ifs() {
        let (mut _wasm_module, body) = nested_ifs_body();
        assert_eq!(HighLevelBody::try_from(LowLevelBody(body)).unwrap(), {
            let type_ = FunctionType::empty();
            use super::Instr::*;
            use wasabi_wasm::{BinaryOp::*, LocalOp::*, UnaryOp::*, Val::*};
            HighLevelBody(vec![
                Local(Get, 0_usize.into()),
                Unary(I32Eqz),
                If(
                    type_,
                    vec![Block(
                        type_,
                        vec![
                            Local(Get, 0_usize.into()),
                            Const(I32(1)),
                            Binary(I32Eq),
                            If(
                                type_,
                                vec![Block(
                                    type_,
                                    vec![
                                        Local(Get, 0_usize.into()),
                                        Const(I32(2)),
                                        Binary(I32Eq),
                                        If(
                                            type_,
                                            vec![Block(type_, vec![Const(I32(42))])],
                                            Some(vec![Const(I32(21))]),
                                        ),
                                    ],
                                )],
                                Some(vec![Const(I32(10))]),
                            ),
                        ],
                    )],
                    Some(vec![Const(I32(5))]),
                ),
            ])
        },)
    }

    #[test]
    fn compute_cost_blocks_loops() {
        // Include cost computation with loops
        let instr_loop = || {
            Instr::Loop(
                FunctionType::empty(),
                vec![Instr::Block(
                    FunctionType::empty(),
                    vec![Instr::Const(Val::I32(0))],
                )],
            )
        };
        let instrumentation_delta = delta_to_instrument_body(&[Instr::if_then_else(
            FunctionType::empty(),
            vec![instr_loop()],
            vec![instr_loop()],
        )]);
        assert_eq!(instrumentation_delta, TRANSFORM_COST_PER_IF_THEN_ELSE_INSTR);
    }

    #[test]
    fn compute_cost() {
        // TODO: 1. pick a more complex wasm program
        // TODO: 2. use property-based testing
        // FIXME: input program is only using if-then-else, delta_to_instrument would compute for instrumenting all
        let (mut wasm_module, body) = nested_ifs_body();
        let HighLevelBody(high_level_body) = LowLevelBody(body.clone()).try_into().unwrap();
        let delta_to_instrument = delta_to_instrument_body(&high_level_body);

        let body_length = |wasm_module: &wasabi_wasm::Module, index: usize| {
            wasm_module
                .function(index.into())
                .code()
                .unwrap()
                .body
                .len()
        };

        let length_uninstrumented = body_length(&wasm_module, 0);

        wasm_module
            .instrument_function_bodies(
                &HashSet::from_iter(vec![0_usize.into()]),
                &Target::IfThenElse(0_usize.into()),
            )
            .unwrap();

        let length_instrumented = body_length(&wasm_module, 0);
        assert!(length_uninstrumented < length_instrumented);
        assert_eq!(
            length_instrumented,
            length_uninstrumented + delta_to_instrument
        );
    }

    #[test]
    fn test_target() {
        let target = Target::IfThen(0_usize.into());
        assert_eq!(target.clone(), target);
        assert_eq!(format!("{target:?}"), "IfThen(Function 0)");
    }
}
