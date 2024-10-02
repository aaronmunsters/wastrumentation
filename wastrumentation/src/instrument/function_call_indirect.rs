use crate::parse_nesting::{
    BodyInner, HighLevelBody, HighLevelInstr as Instr, TypedHighLevelInstr,
};
use wasabi_wasm::{Function, Idx, Val};

use super::TransformationStrategy;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Target {
    Pre(Idx<Function>),
    Post(Idx<Function>),
    IndirectPre(Idx<Function>),
    IndirectPost(Idx<Function>),
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

fn transform(body: &BodyInner, target: Target) -> BodyInner {
    let mut result = Vec::new();

    for typed_instr @ TypedHighLevelInstr { instr, .. } in body {
        if typed_instr.is_uninstrumented() {
            result.push(typed_instr.clone());
            continue;
        }

        match (target, instr) {
            (Target::Pre(call_pre_idx), Instr::Call(index)) => {
                result.extend_from_slice(&[
                    // STACK: [type_in]
                    typed_instr.instrument_with(Instr::Const(Val::I32(
                        i32::try_from(index.to_u32()).unwrap(),
                    ))),
                    // STACK: [type_in, f_idx]
                    typed_instr.instrument_with(Instr::Call(call_pre_idx)),
                    // STACK: [type_in]
                    typed_instr.place_original(instr.clone()),
                    // STACK: [type_out]
                ]);
            }
            (Target::Post(call_post_idx), Instr::Call(index)) => {
                result.extend_from_slice(&[
                    // STACK: [type_in]
                    typed_instr.place_original(instr.clone()),
                    // STACK: [type_out]
                    typed_instr.instrument_with(Instr::Const(Val::I32(
                        i32::try_from(index.to_u32()).unwrap(),
                    ))),
                    // STACK: [type_out, f_idx]
                    typed_instr.instrument_with(Instr::Call(call_post_idx)),
                    // STACK: [type_out]
                ]);
            }
            (
                Target::IndirectPre(call_pre_idx),
                Instr::CallIndirect(_function_type, table_index),
            ) => {
                result.extend_from_slice(&[
                    // STACK: [type_in, table_function_index]
                    typed_instr.instrument_with(Instr::Const(Val::I32(
                        i32::try_from(table_index.to_u32()).unwrap(),
                    ))),
                    // STACK: [type_in, table_function_index, table_index]
                    typed_instr.instrument_with(Instr::Call(call_pre_idx)),
                    // STACK: [type_in, table_function_index]
                    typed_instr.place_original(instr.clone()),
                    // STACK: [type_out]
                ]);
            }
            (
                Target::IndirectPost(call_post_idx),
                Instr::CallIndirect(_function_type, table_index),
            ) => {
                result.extend_from_slice(&[
                    // STACK: [type_in, table_function_index]
                    typed_instr.place_original(instr.clone()),
                    // STACK: [type_out]
                    typed_instr.instrument_with(Instr::Const(Val::I32(
                        i32::try_from(table_index.to_u32()).unwrap(),
                    ))),
                    // STACK: [type_out, table_index]
                    typed_instr.instrument_with(Instr::Call(call_post_idx)),
                    // STACK: [type_out]
                ]);
            }

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

// TODO: implement tests
