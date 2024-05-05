use std::collections::HashSet;

use crate::parse_nesting::{HighLevelBody, Instr, LowLevelBody};
use wasabi_wasm::{Code, Function, Idx, ImportOrPresent, Module, Val};
use wasp_compiler::wasp_interface::WasmExport;

use super::{function_application::INSTRUMENTATION_ANALYSIS_MODULE, FunctionTypeConvertible};

type CallTransformationError = &'static str;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Target {
    CallIndirectPre,
    CallIndirectPost,
}

pub fn instrument(
    module: &mut Module,
    target_functions: &HashSet<Idx<Function>>,
    trap: WasmExport,
    target: Target,
) -> Result<(), CallTransformationError> {
    let trap = module.add_function_import(
        trap.as_function_type(),
        INSTRUMENTATION_ANALYSIS_MODULE.to_string(),
        trap.name,
    );

    instrument_bodies(module, target_functions, trap, target)
}

pub fn instrument_bodies(
    module: &mut Module,
    target_functions: &HashSet<Idx<Function>>,
    trap: Idx<Function>,
    target: Target,
) -> Result<(), CallTransformationError> {
    for target_function_idx in target_functions {
        let target_function = module.function_mut(*target_function_idx);
        if target_function.code().is_none() {
            return Err("Attempt to instrument call indirect on import function");
        }

        let code = target_function.code_mut().unwrap(); // checked above
        let high_level_body: HighLevelBody = LowLevelBody(code.body.clone()).try_into()?;
        let high_level_body_transformed = high_level_body.transform_call_indirect(trap, target);
        let LowLevelBody(transformed_low_level_body) = high_level_body_transformed.into();

        target_function.code = ImportOrPresent::Present(Code {
            body: transformed_low_level_body,
            locals: code.locals.clone(),
        });
    }
    Ok(())
}

// Amount of constant instructions in transformation
const TRANSFORM_COST_PER_CALL_INDIRECT: usize = 2;

fn delta_to_instrument_instr(instr: &Instr) -> usize {
    match instr {
        Instr::CallIndirect(..) => TRANSFORM_COST_PER_CALL_INDIRECT,
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

impl HighLevelBody {
    #[must_use]
    pub fn transform_call_indirect(&self, trap: Idx<Function>, target: Target) -> Self {
        let Self(body) = self;
        let transformed_body = Self::transform_call_indirect_inner(body, trap, target);
        Self(transformed_body)
    }

    fn transform_call_indirect_inner(
        body: &Vec<Instr>,
        trap: Idx<Function>,
        target: Target,
    ) -> Vec<Instr> {
        let mut result = Vec::new();

        for instr in body {
            match (target, instr) {
                (Target::CallIndirectPre, Instr::CallIndirect(_function_type, table_index)) => {
                    result.extend_from_slice(&[
                        // STACK: [type_in, table_function_index]
                        Instr::Const(Val::I32(i32::try_from(table_index.to_u32()).unwrap())),
                        // STACK: [type_in, table_function_index, table_index]
                        Instr::Call(trap),
                        // STACK: [type_in, table_function_index]
                        instr.clone(),
                        // STACK: [type_out]
                    ]);
                }
                (Target::CallIndirectPost, Instr::CallIndirect(_function_type, table_index)) => {
                    result.extend_from_slice(&[
                        // STACK: [type_in, table_function_index]
                        instr.clone(),
                        // STACK: [type_out]
                        Instr::Const(Val::I32(i32::try_from(table_index.to_u32()).unwrap())),
                        // STACK: [type_out, table_index]
                        Instr::Call(trap),
                        // STACK: [type_out]
                    ]);
                }
                (target, Instr::If(type_, then, None)) => result.push(Instr::If(
                    *type_,
                    Self::transform_call_indirect_inner(then, trap, target),
                    None,
                )),
                (target, Instr::If(type_, then, Some(else_))) => result.push(Instr::If(
                    *type_,
                    Self::transform_call_indirect_inner(then, trap, target),
                    Some(Self::transform_call_indirect_inner(else_, trap, target)),
                )),
                (target, Instr::Loop(type_, body)) => result.push(Instr::Loop(
                    *type_,
                    Self::transform_call_indirect_inner(body, trap, target),
                )),
                (target, Instr::Block(type_, body)) => result.push(Instr::Block(
                    *type_,
                    Self::transform_call_indirect_inner(body, trap, target),
                )),
                _ => result.push(instr.clone()),
            }
        }
        result
    }
}

// TODO: implement tests
