use crate::parse_nesting::{Body, HighLevelBody, Instr};
use wasabi_wasm::{Function, Idx};

use super::TransformationStrategy;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Target {
    BlockPre(Idx<Function>),
    BlockPost(Idx<Function>),
    LoopPre(Idx<Function>),
    LoopPost(Idx<Function>),
    Select(Idx<Function>),
}

impl TransformationStrategy for Target {
    fn transform(&self, high_level_body: &HighLevelBody) -> HighLevelBody {
        let HighLevelBody(body) = high_level_body;
        let transformed_body = transform(body, *self);
        HighLevelBody(transformed_body)
    }
}

fn transform(body: &Body, target: Target) -> Body {
    let mut result = Vec::new();

    for (_index, instr) in body {
        match (target, instr) {
            (Target::BlockPre(trap_idx), Instr::Block(type_, body)) => {
                result.extend_from_slice(&[
                    // STACK: [type_in]
                    Instr::Call(trap_idx),
                    // STACK: [type_in]
                    Instr::Block(*type_, transform(body, target)),
                ]);
            }
            (Target::BlockPost(trap_idx), Instr::Block(type_, body)) => {
                result.extend_from_slice(&[
                    // STACK: [type_in]
                    Instr::Block(*type_, transform(body, target)),
                    // STACK: [type_in]
                    Instr::Call(trap_idx),
                ]);
            }
            (Target::LoopPre(trap_idx), Instr::Loop(type_, body)) => {
                result.extend_from_slice(&[
                    // STACK: [type_in]
                    Instr::Call(trap_idx),
                    // STACK: [type_in]
                    Instr::Loop(*type_, transform(body, target)),
                ]);
            }
            (Target::LoopPost(trap_idx), Instr::Loop(type_, body)) => {
                result.extend_from_slice(&[
                    // STACK: [type_in]
                    Instr::Loop(*type_, transform(body, target)),
                    // STACK: [type_in]
                    Instr::Call(trap_idx),
                ]);
            }
            (Target::Select(trap_idx), Instr::Select) => {
                result.extend_from_slice(&[
                    // STACK: [then_type_in, else_type_in, condition_i32]
                    Instr::Call(trap_idx),
                    // STACK: [then_type_in, else_type_in, kontinuation]
                    Instr::Select,
                ]);
            }
            (Target::Select(trap_idx), Instr::TypedSelect(type_)) => {
                result.extend_from_slice(&[
                    // STACK: [then_type_in, else_type_in, condition_i32]
                    Instr::Call(trap_idx),
                    // STACK: [then_type_in, else_type_in, kontinuation]
                    Instr::TypedSelect(*type_),
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

#[cfg(test)]
mod tests {
    // TODO:
}
