const THEN_KONTN: i32 = 0;
const ELSE_KONTN: i32 = 1;
const SKIP_KONTN: i32 = 1;

// TODO:
// IDEA: Allow for 'custom continuation', in which instrumentation platform
//       jumps to custom branch bevaviour and not only the behavior present
//       in the input program.

/// Boolean values in WebAssembly are represented as values of type `i32`.
/// In a boolean context, such as a `br_if` condition, any non-zero value
/// is interpreted as true and `0` is interpreted as false.
///
/// [Link to Wasm reference manual source](
/// https://github.com/sunfishcode/wasm-reference-manual/blob/master/WebAssembly.md#booleans
/// )
const WASM_FALSE: i32 = 0;

use std::collections::HashSet;

use crate::parse_nesting::{HighLevelBody, Instr, LowLevelBody};
use wasabi_wasm::{
    BinaryOp, Code, Function, Idx, ImportOrPresent, Local, LocalOp, Module, Val, ValType,
};
use wasp_compiler::wasp_interface::WasmExport;

use super::{function_application::INSTRUMENTATION_ANALYSIS_MODULE, FunctionTypeConvertible};

type BranchTransformationError = &'static str;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Target {
    IfThen,
    IfThenElse,
    BrIf,
}

pub fn instrument(
    module: &mut Module,
    target_functions: &HashSet<Idx<Function>>,
    trap_export: WasmExport,
    target: Target,
) -> Result<(), BranchTransformationError> {
    let if_k_f_idx = module.add_function_import(
        trap_export.as_function_type(),
        INSTRUMENTATION_ANALYSIS_MODULE.to_string(),
        trap_export.name,
    );

    instrument_bodies(module, target_functions, &if_k_f_idx, target)
}

pub fn instrument_bodies(
    module: &mut Module,
    target_functions: &HashSet<Idx<Function>>,
    if_k_f_idx: &Idx<Function>,
    target: Target,
) -> Result<(), BranchTransformationError> {
    for target_function_idx in target_functions.iter() {
        let target_function = module.function_mut(*target_function_idx);
        if target_function.code().is_none() {
            return Err("Attempt to instrument if-then-else on import function");
        }
        let store_if_continuation = target_function.add_fresh_local(ValType::I32);
        let code = target_function.code_mut().unwrap(); // checked above

        let high_level_body: HighLevelBody = LowLevelBody(code.body.clone()).try_into()?;
        let high_level_body_transformed =
            high_level_body.transform(if_k_f_idx, &store_if_continuation, target);
        let LowLevelBody(transformed_low_level_body) = high_level_body_transformed.into();

        target_function.code = ImportOrPresent::Present(Code {
            body: transformed_low_level_body,
            locals: code.locals.clone(),
        });
    }
    Ok(())
}

// TODO: Correct this / determine this up front
// Amount of constant instructions in transformation
const TRANSFORM_COST_PER_IF_THEN_INSTR: usize = 17;
const TRANSFORM_COST_PER_IF_THEN_ELSE_INSTR: usize = 17;
const TRANSFORM_COST_PER_BR_IF: usize = 17;

fn cost_for(body: &[Instr]) -> usize {
    body.iter()
        .map(|instr| -> usize {
            match instr {
                Instr::Loop(_, body) => cost_for(body),
                Instr::Block(_, body) => cost_for(body),
                Instr::If(_, _, None) => TRANSFORM_COST_PER_IF_THEN_INSTR,
                Instr::If(_, _, Some(_)) => TRANSFORM_COST_PER_IF_THEN_ELSE_INSTR,
                Instr::BrIf(_) => TRANSFORM_COST_PER_BR_IF,
                _ => 1,
            }
        })
        .sum()
}

impl HighLevelBody {
    pub fn transform(
        &self,
        if_k_f_idx: &Idx<Function>,
        store_if_continuation: &Idx<Local>,
        target: Target,
    ) -> Self {
        let Self(body) = self;
        let transformed_body =
            Self::transform_inner(body, if_k_f_idx, store_if_continuation, target);
        Self(transformed_body)
    }

    // TODO: Is is possible to drop the equality check?
    fn transform_inner(
        body: &Vec<Instr>,
        if_k_f_idx: &Idx<Function>,
        store_if_continuation: &Idx<Local>,
        target: Target,
    ) -> Vec<Instr> {
        let mut result = Vec::with_capacity(cost_for(body));
        for instr in body {
            match (target, instr) {
                (Target::IfThen, Instr::If(type_, then, None)) => {
                    result.extend_from_slice(&[
                        // STACK: [type_in, condition]
                        Instr::Const(wasabi_wasm::Val::I32(WASM_FALSE)),
                        // STACK: [type_in, condition, false]
                        Instr::Binary(BinaryOp::I32Eq),
                        // STACK: [type_in, PATH_KONTN]
                        Instr::Call(*if_k_f_idx),
                        // STACK: [type_in, kontinuation]
                        Instr::Local(LocalOp::Tee, *store_if_continuation),
                        // STACK: [type_in, kontinuation], local.store_if_continuation = kontinuation
                        Instr::Const(wasabi_wasm::Val::I32(THEN_KONTN)),
                        // STACK: [type_in, kontinuation, THEN_KONTN]
                        Instr::Binary(wasabi_wasm::BinaryOp::I32Eq),
                        // STACK: [type_in, condition]
                        Instr::if_then_else(
                            *type_,
                            Self::transform_inner(then, if_k_f_idx, store_if_continuation, target),
                            vec![
                                // STACK: [type_in]
                                Instr::Local(LocalOp::Get, *store_if_continuation),
                                // STACK: [type_in, kontinuation]
                                Instr::Const(wasabi_wasm::Val::I32(SKIP_KONTN)),
                                // STACK: [type_in, kontinuation, CSTM_KONTN]
                                Instr::Binary(wasabi_wasm::BinaryOp::I32Eq),
                                // STACK: [type_in, condition]
                                Instr::if_then_else(
                                    *type_,
                                    vec![], // Do nothing
                                    vec![Instr::Unreachable],
                                ),
                            ],
                        ),
                        // STACK: [type_out]
                    ]);
                }
                (Target::IfThenElse, Instr::If(type_, then, Some(else_))) => {
                    result.extend_from_slice(&[
                        // STACK: [type_in, condition]
                        Instr::Const(wasabi_wasm::Val::I32(WASM_FALSE)),
                        // STACK: [type_in, condition, false]
                        Instr::Binary(BinaryOp::I32Eq),
                        // STACK: [type_in, PATH_KONTN]
                        Instr::Call(*if_k_f_idx),
                        // STACK: [type_in, kontinuation]
                        Instr::Local(LocalOp::Tee, *store_if_continuation),
                        // STACK: [type_in, kontinuation], local.store_if_continuation = kontinuation
                        Instr::Const(wasabi_wasm::Val::I32(THEN_KONTN)),
                        // STACK: [type_in, kontinuation, THEN_KONTN]
                        Instr::Binary(wasabi_wasm::BinaryOp::I32Eq),
                        // STACK: [type_in, condition]
                        Instr::if_then_else(
                            *type_,
                            Self::transform_inner(then, if_k_f_idx, store_if_continuation, target),
                            vec![
                                // STACK: [type_in]
                                Instr::Local(LocalOp::Get, *store_if_continuation),
                                // STACK: [type_in, kontinuation]
                                Instr::Const(wasabi_wasm::Val::I32(ELSE_KONTN)),
                                // STACK: [type_in, kontinuation, ELSE_KONTN]
                                Instr::Binary(wasabi_wasm::BinaryOp::I32Eq),
                                // STACK: [type_in, condition]
                                Instr::if_then_else(
                                    *type_,
                                    Self::transform_inner(
                                        else_,
                                        if_k_f_idx,
                                        store_if_continuation,
                                        target,
                                    ),
                                    vec![Instr::Unreachable],
                                ),
                            ],
                        ),
                        // STACK: [type_out]
                    ]);
                }
                (Target::BrIf, Instr::BrIf(label)) => {
                    result.extend_from_slice(&[
                        // STACK: [condition]
                        Instr::Const(wasabi_wasm::Val::I32(WASM_FALSE)),
                        // STACK: [condition, false]
                        Instr::Binary(BinaryOp::I32Eq),
                        // STACK: [PATH_KONTN]
                        Instr::Const(Val::I32(label.to_u32() as i32)),
                        // STACK: [PATH_KONTN, label]
                        Instr::Call(*if_k_f_idx),
                        // STACK: [kontinuation]
                        Instr::Local(LocalOp::Tee, *store_if_continuation),
                        // STACK: [kontinuation], local.store_if_continuation = kontinuation
                        Instr::Const(wasabi_wasm::Val::I32(THEN_KONTN)),
                        // STACK: [kontinuation, THEN_KONTN]
                        Instr::Binary(wasabi_wasm::BinaryOp::I32Eq),
                        // STACK: [condition]
                        Instr::BrIf(*label),
                        // STACK: []
                    ]);
                }
                (target, Instr::Loop(type_, body)) => result.push(Instr::Loop(
                    *type_,
                    Self::transform_inner(body, if_k_f_idx, store_if_continuation, target),
                )),
                (target, Instr::Block(type_, body)) => result.push(Instr::Block(
                    *type_,
                    Self::transform_inner(body, if_k_f_idx, store_if_continuation, target),
                )),

                _ => result.push(instr.clone()),
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use wasabi_wasm::{FunctionType, Val, ValType};
    use wasmtime::{Engine, Instance, Module, Store};

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
        let if_k_f_idx = wasm_module.add_function(
            FunctionType::new(&[ValType::I32], &[ValType::I32]),
            vec![],
            instrumentation_body,
        );
        instrument_bodies(
            &mut wasm_module,
            &HashSet::from_iter(vec![0_usize.into()]),
            &if_k_f_idx,
            Target::IfThenElse,
        )
        .unwrap();

        // Execute uninstrumented:
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
            instrument_bodies(
                &mut wasm_module,
                &HashSet::from_iter(vec![0_usize.into()]),
                &0_usize.into(),
                Target::IfThenElse
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
            Err("Attempt to instrument if-then-else on import function"),
            instrument_bodies(
                &mut wasm_module,
                &HashSet::from_iter(vec![0_usize.into()]),
                &0_usize.into(),
                Target::IfThenElse
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
        let (mut wasm_module, body) = nested_ifs_body();

        // Instrument
        instrument_bodies(
            &mut wasm_module,
            &HashSet::from_iter(vec![0_usize.into()]),
            &0_usize.into(),
            Target::IfThenElse,
        )
        .unwrap();

        assert_eq!(HighLevelBody::try_from(LowLevelBody(body)).unwrap(), {
            let type_ = FunctionType::new(&[], &[]);
            use wasabi_wasm::BinaryOp::*;
            use wasabi_wasm::UnaryOp::*;
            use wasabi_wasm::Val::*;
            use {super::Instr::*, super::LocalOp::*};
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
        let type_void_to_void = FunctionType::new(&[], &[]);
        let instr_loop = || {
            Instr::Loop(
                type_void_to_void,
                vec![Instr::Block(
                    type_void_to_void,
                    vec![Instr::Const(Val::I32(0))],
                )],
            )
        };
        let cost = cost_for(&[Instr::if_then_else(
            type_void_to_void,
            vec![instr_loop()],
            vec![instr_loop()],
        )]);
        assert_eq!(cost, 17);
    }

    #[test]
    fn compute_cost_through_logic() {
        // Assert
        let (mut wasm_module, body) = nested_ifs_body();
        let count_ifs = |body: &[wasabi_wasm::Instr]| {
            body.iter().fold(0, |acc, instr| {
                if let wasabi_wasm::Instr::If(..) = instr {
                    acc + 1
                } else {
                    acc
                }
            })
        };

        let pre_instr_if_count = count_ifs(&body);

        // Instrument
        instrument_bodies(
            &mut wasm_module,
            &HashSet::from_iter(vec![0_usize.into()]),
            &0_usize.into(),
            Target::IfThenElse,
        )
        .unwrap();

        let transformed_body = wasm_module
            .function(0_usize.into())
            .code()
            .unwrap()
            .body
            .clone();

        let post_instrumentation_ifs_count = count_ifs(&transformed_body);

        assert_eq!(pre_instr_if_count, 3);
        assert_eq!(post_instrumentation_ifs_count, pre_instr_if_count * 2)
    }

    #[test]
    fn test_target() {
        let target = Target::IfThen;
        assert_eq!(target.clone(), target);
        assert_eq!(format!("{target:?}"), "IfThen");
    }
}
