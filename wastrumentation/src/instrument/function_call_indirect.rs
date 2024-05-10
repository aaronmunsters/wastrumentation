use crate::parse_nesting::{HighLevelBody, Instr};
use wasabi_wasm::{Function, Idx, Val};

use super::TransformationStrategy;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Target {
    CallIndirectPre(Idx<Function>),
    CallIndirectPost(Idx<Function>),
}

impl TransformationStrategy for Target {
    fn transform(&self, high_level_body: &HighLevelBody) -> HighLevelBody {
        high_level_body.transform_call_indirect(*self)
    }
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
    pub fn transform_call_indirect(&self, target: Target) -> Self {
        let Self(body) = self;
        let transformed_body = Self::transform_call_indirect_inner(body, target);
        Self(transformed_body)
    }

    fn transform_call_indirect_inner(body: &Vec<Instr>, target: Target) -> Vec<Instr> {
        let mut result = Vec::new();

        for instr in body {
            match (target, instr) {
                (
                    Target::CallIndirectPre(call_pre_idx),
                    Instr::CallIndirect(_function_type, table_index),
                ) => {
                    result.extend_from_slice(&[
                        // STACK: [type_in, table_function_index]
                        Instr::Const(Val::I32(i32::try_from(table_index.to_u32()).unwrap())),
                        // STACK: [type_in, table_function_index, table_index]
                        Instr::Call(call_pre_idx),
                        // STACK: [type_in, table_function_index]
                        instr.clone(),
                        // STACK: [type_out]
                    ]);
                }
                (
                    Target::CallIndirectPost(call_post_idx),
                    Instr::CallIndirect(_function_type, table_index),
                ) => {
                    result.extend_from_slice(&[
                        // STACK: [type_in, table_function_index]
                        instr.clone(),
                        // STACK: [type_out]
                        Instr::Const(Val::I32(i32::try_from(table_index.to_u32()).unwrap())),
                        // STACK: [type_out, table_index]
                        Instr::Call(call_post_idx),
                        // STACK: [type_out]
                    ]);
                }
                (target, Instr::If(type_, then, None)) => result.push(Instr::If(
                    *type_,
                    Self::transform_call_indirect_inner(then, target),
                    None,
                )),
                (target, Instr::If(type_, then, Some(else_))) => result.push(Instr::If(
                    *type_,
                    Self::transform_call_indirect_inner(then, target),
                    Some(Self::transform_call_indirect_inner(else_, target)),
                )),
                (target, Instr::Loop(type_, body)) => result.push(Instr::Loop(
                    *type_,
                    Self::transform_call_indirect_inner(body, target),
                )),
                (target, Instr::Block(type_, body)) => result.push(Instr::Block(
                    *type_,
                    Self::transform_call_indirect_inner(body, target),
                )),
                _ => result.push(instr.clone()),
            }
        }
        result
    }
}

// TODO: implement tests
