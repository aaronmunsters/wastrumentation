use crate::parse_nesting::{
    BodyInner, HighLevelBody, HighLevelInstr as Instr, TypedHighLevelInstr,
};
use wasabi_wasm::{Function, Idx, Module};

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
    fn transform(&self, high_level_body: &HighLevelBody, _: &mut Module) -> HighLevelBody {
        let HighLevelBody(body) = high_level_body;
        let transformed_body = transform(body, *self);
        HighLevelBody(transformed_body)
    }
}

fn transform(body: &BodyInner, target: Target) -> BodyInner {
    let mut result = Vec::new();

    for typed_instr @ TypedHighLevelInstr { instr, .. } in body {
        if typed_instr.is_uninstrumented() {
            match (target, instr) {
                (Target::BlockPre(trap_idx), Instr::Block(type_, body)) => {
                    let mut injected_body = vec![
                        // STACK: [type_in]
                        typed_instr.instrument_with(Instr::Const(wasabi_wasm::Val::I32(
                            type_.inputs().len().try_into().unwrap(),
                        ))),
                        // STACK: [type_in, input_c:i32]
                        typed_instr.instrument_with(Instr::Const(wasabi_wasm::Val::I32(
                            type_.results().len().try_into().unwrap(),
                        ))),
                    ];
                    // STACK: [type_in, input_c:i32, arity:i32]
                    injected_body.extend_from_slice(&typed_instr.to_trap_call(&trap_idx));
                    // append rest of body
                    injected_body.extend_from_slice(&transform(body, target));
                    // STACK: [type_in]
                    result.push(typed_instr.place_original(Instr::Block(*type_, injected_body)));
                    continue;
                }
                (Target::BlockPost(trap_idx), Instr::Block(type_, body)) => {
                    // STACK: [type_in]
                    let mut injected_body = transform(body, target);
                    // append to rest of body
                    injected_body.extend_from_slice(&typed_instr.to_trap_call(&trap_idx));
                    // STACK: [type_in]
                    result.push(typed_instr.place_original(Instr::Block(*type_, injected_body)));
                    continue;
                }
                (Target::LoopPre(trap_idx), Instr::Loop(type_, body)) => {
                    let mut injected_body = vec![
                        // STACK: [type_in]
                        typed_instr.instrument_with(Instr::Const(wasabi_wasm::Val::I32(
                            type_.inputs().len().try_into().unwrap(),
                        ))),
                        // STACK: [type_in, input_c:i32]
                        typed_instr.instrument_with(Instr::Const(wasabi_wasm::Val::I32(
                            type_.results().len().try_into().unwrap(),
                        ))),
                    ];
                    // STACK: [type_in, input_c:i32, arity:i32]
                    injected_body.extend_from_slice(&typed_instr.to_trap_call(&trap_idx));
                    // append rest of body
                    injected_body.extend_from_slice(&transform(body, target));
                    // STACK: [type_in]
                    result.push(typed_instr.place_original(Instr::Loop(*type_, injected_body)));
                    continue;
                }
                (Target::LoopPost(trap_idx), Instr::Loop(type_, body)) => {
                    // STACK: [type_in]
                    let mut injected_body = transform(body, target);
                    // append to rest of body
                    injected_body.extend_from_slice(&typed_instr.to_trap_call(&trap_idx));
                    // STACK: [type_in]
                    result.push(typed_instr.place_original(Instr::Loop(*type_, injected_body)));
                    continue;
                }
                (Target::Select(trap_idx), Instr::Select) => {
                    // STACK: [then_type_in, else_type_in, condition_i32]
                    result.extend_from_slice(&typed_instr.to_trap_call(&trap_idx));
                    // STACK: [then_type_in, else_type_in, kontinuation]
                    result.push(typed_instr.place_original(Instr::Select));
                    continue;
                }
                (Target::Select(trap_idx), Instr::TypedSelect(type_)) => {
                    // STACK: [then_type_in, else_type_in, condition_i32]
                    result.extend_from_slice(&typed_instr.to_trap_call(&trap_idx));
                    // STACK: [then_type_in, else_type_in, kontinuation]
                    result.push(typed_instr.place_original(Instr::TypedSelect(*type_)));
                    continue;
                }
                _ => (),
            }
        }

        match (target, instr) {
            // DEFAULT TRAVERSAL
            (target, Instr::If(type_, then, None)) => {
                result.push(typed_instr.place_untouched(Instr::If(
                    *type_,
                    transform(then, target),
                    None,
                )));
            }
            (target, Instr::If(type_, then, Some(else_))) => {
                result.push(typed_instr.place_untouched(Instr::If(
                    *type_,
                    transform(then, target),
                    Some(transform(else_, target)),
                )))
            }
            (target, Instr::Loop(type_, body)) => {
                result.push(
                    typed_instr.place_untouched(Instr::Loop(*type_, transform(body, target))),
                );
            }
            (target, Instr::Block(type_, body)) => {
                result.push(
                    typed_instr.place_untouched(Instr::Block(*type_, transform(body, target))),
                );
            }
            (_, instr) => result.push(typed_instr.place_untouched(instr.clone())),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    // TODO:
}
