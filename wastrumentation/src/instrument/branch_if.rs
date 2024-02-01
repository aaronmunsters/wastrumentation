const THEN_KONTN: i32 = 0;
const ELSE_KONTN: i32 = 1;
const CSTM_KONTN: i32 = 2;

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
    BinaryOp, Code, Function, Idx, ImportOrPresent, Local, LocalOp, Module, ValType,
};
use wasp_compiler::wasp_interface::WasmExport;

use super::{function_application::INSTRUMENTATION_ANALYSIS_MODULE, FunctionTypeConvertible};

type BranchTransformationError = &'static str;

pub fn instrument(
    module: &mut Module,
    target_functions: &HashSet<Idx<Function>>,
    if_then_else_trap_export: WasmExport,
) -> Result<(), BranchTransformationError> {
    let if_k_f_idx = module.add_function_import(
        if_then_else_trap_export.as_function_type(),
        INSTRUMENTATION_ANALYSIS_MODULE.to_string(),
        if_then_else_trap_export.name,
    );

    instrument_bodies(module, target_functions, &if_k_f_idx)
}

pub fn instrument_bodies(
    module: &mut Module,
    target_functions: &HashSet<Idx<Function>>,
    if_k_f_idx: &Idx<Function>,
) -> Result<(), BranchTransformationError> {
    for target_function_idx in target_functions.iter() {
        let target_function = module.function_mut(*target_function_idx);
        if target_function.code().is_none() {
            continue;
        };

        let store_if_continuation = target_function.add_fresh_local(ValType::I32);
        let code = target_function
            .code_mut()
            .expect("Just checked for presence");

        let high_level_body: HighLevelBody = LowLevelBody(code.body.clone()).try_into()?;
        let high_level_body_transformed =
            high_level_body.transform(if_k_f_idx, &store_if_continuation);
        let LowLevelBody(transformed_low_level_body) = high_level_body_transformed.into();

        target_function.code = ImportOrPresent::Present(Code {
            body: transformed_low_level_body,
            locals: code.locals.clone(),
        });
    }
    Ok(())
}

const TRANSFORM_COST_PER_INSTR: usize = 14; // Amount of constant instructions in transformation

fn cost_for(body: &[Instr]) -> usize {
    body.iter()
        .map(|instr| -> usize {
            match instr {
                Instr::Loop(_, body) => cost_for(body),
                Instr::Block(_, body) => cost_for(body),
                Instr::If(_, then, None) => TRANSFORM_COST_PER_INSTR + cost_for(then),
                Instr::If(_, then, Some(else_)) => {
                    TRANSFORM_COST_PER_INSTR + cost_for(then) + cost_for(else_)
                }
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
    ) -> Self {
        let Self(body) = self;
        let transformed_body = Self::transform_inner(body, if_k_f_idx, store_if_continuation);
        Self(transformed_body)
    }

    fn transform_inner(
        body: &Vec<Instr>,
        if_k_f_idx: &Idx<Function>,
        store_if_continuation: &Idx<Local>,
    ) -> Vec<Instr> {
        let mut result = Vec::with_capacity(cost_for(body));
        for instr in body {
            match instr {
                Instr::If(type_, then, Some(else_)) => {
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
                            Self::transform_inner(then, if_k_f_idx, store_if_continuation),
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
                                    Self::transform_inner(else_, if_k_f_idx, store_if_continuation),
                                    vec![
                                        // STACK: [type_in]
                                        Instr::Local(LocalOp::Get, *store_if_continuation),
                                        // STACK: [type_in, kontinuation]
                                        Instr::Const(wasabi_wasm::Val::I32(CSTM_KONTN)),
                                        // STACK: [type_in, kontinuation, CSTM_KONTN]
                                        Instr::Binary(wasabi_wasm::BinaryOp::I32Eq),
                                        // STACK: [type_in, condition]
                                        Instr::if_then_else(
                                            *type_,
                                            vec![Instr::Unreachable], // vec![Instr::Call(cstm_kontn)], // TODO:
                                            vec![Instr::Unreachable],
                                        ),
                                    ],
                                ),
                            ],
                        ),
                        // STACK: [type_out]
                    ]);
                }
                _ => result.push(instr.clone()),
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use wasabi_wasm::{FunctionType, ValType};
    use wasmtime::{Engine, Instance, Module, Store};

    use super::*;

    #[derive(Debug)]
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
        )
        .unwrap();

        // Execute uninstrumented:
        assert_outcome(&wasm_module, instrumented_assertions);
    }
}
