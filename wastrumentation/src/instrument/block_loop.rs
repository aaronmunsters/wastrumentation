use crate::parse_nesting::{HighLevelBody, Instr};
use wasabi_wasm::{Function, Idx};

use super::TransformationStrategy;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Target {
    BlockPre(Idx<Function>),
    BlockPost(Idx<Function>),
    LoopPre(Idx<Function>),
    LoopPost(Idx<Function>),
}

impl TransformationStrategy for Target {
    fn transform(&self, high_level_body: &HighLevelBody) -> HighLevelBody {
        let HighLevelBody(body) = high_level_body;
        let transformed_body = transform(body, *self);
        HighLevelBody(transformed_body)
    }
}

fn transform(body: &Vec<Instr>, target: Target) -> Vec<Instr> {
    let mut result = Vec::new();

    for instr in body {
        match (target, instr) {
            (Target::BlockPre(block_trap_idx), Instr::Block(type_, body)) => {
                result.extend_from_slice(&[
                    // STACK: [type_in]
                    Instr::Call(block_trap_idx),
                    // STACK: [type_in]
                    Instr::Block(*type_, body.clone()),
                ]);
            }
            (Target::BlockPost(block_trap_idx), Instr::Block(type_, body)) => {
                result.extend_from_slice(&[
                    // STACK: [type_in]
                    Instr::Block(*type_, body.clone()),
                    // STACK: [type_in]
                    Instr::Call(block_trap_idx),
                ]);
            }
            (Target::LoopPre(block_trap_idx), Instr::Loop(type_, body)) => {
                result.extend_from_slice(&[
                    // STACK: [type_in]
                    Instr::Call(block_trap_idx),
                    // STACK: [type_in]
                    Instr::Loop(*type_, body.clone()),
                ]);
            }
            (Target::LoopPost(block_trap_idx), Instr::Loop(type_, body)) => {
                result.extend_from_slice(&[
                    // STACK: [type_in]
                    Instr::Loop(*type_, body.clone()),
                    // STACK: [type_in]
                    Instr::Call(block_trap_idx),
                ]);
            }

            (target, Instr::If(type_, then, None)) => {
                result.push(Instr::If(*type_, transform(then, target), None))
            }
            (target, Instr::If(type_, then, Some(else_))) => result.push(Instr::If(
                *type_,
                transform(then, target),
                Some(transform(else_, target)),
            )),
            (target, Instr::Loop(type_, body)) => {
                result.push(Instr::Loop(*type_, transform(body, target)))
            }
            (target, Instr::Block(type_, body)) => {
                result.push(Instr::Block(*type_, transform(body, target)))
            }
            _ => result.push(instr.clone()),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    // TODO:
}
