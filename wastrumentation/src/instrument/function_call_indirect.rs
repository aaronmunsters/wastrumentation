use crate::parse_nesting::{Body, HighLevelBody, Instr};
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

impl HighLevelBody {
    #[must_use]
    pub fn transform_call_indirect(&self, target: Target) -> Self {
        let Self(body) = self;
        let transformed_body = transform(body, target);
        Self(transformed_body)
    }
}

fn transform(body: &Body, target: Target) -> Body {
    let mut result = Vec::new();

    for (_, instr) in body {
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
            (target, Instr::If(type_, then, None)) => {
                result.push(Instr::If(*type_, transform(then, target), None));
            }
            (target, Instr::If(type_, then, Some(else_))) => result.push(Instr::If(
                *type_,
                transform(then, target),
                Some(transform(else_, target)),
            )),
            (target, Instr::Loop(type_, body)) => {
                result.push(Instr::Loop(*type_, transform(body, target)));
            }
            (target, Instr::Block(type_, body)) => {
                result.push(Instr::Block(*type_, transform(body, target)));
            }
            _ => result.push(instr.clone()),
        }
    }
    result.into_iter().map(|i| (0, i)).collect()
}

// TODO: implement tests
