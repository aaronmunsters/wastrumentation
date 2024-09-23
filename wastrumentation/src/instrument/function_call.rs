use std::collections::HashSet;

use crate::analysis::WasmExport;
use crate::parse_nesting::{
    BodyInner, HighLevelBody, HighLevelInstr as Instr, LowLevelBody, TypedHighLevelInstr,
};
use wasabi_wasm::{Code, Function, Idx, ImportOrPresent, Module, Val};

use super::InstrumentationError;
use super::{function_application::INSTRUMENTATION_ANALYSIS_MODULE, FunctionTypeConvertible};

#[derive(PartialEq, Eq, Debug)]
pub enum TargetCall<T> {
    None,
    Pre(T),
    Post(T),
    Both { pre_call_trap: T, post_call_trap: T },
}

impl TargetCall<&WasmExport> {
    pub fn instrument(
        &self,
        module: &mut Module,
        target_functions: &HashSet<Idx<Function>>,
    ) -> Result<(), InstrumentationError> {
        let target = match &self {
            TargetCall::None => TargetCall::None,
            TargetCall::Pre(trap) => {
                let trap = module.add_function_import(
                    trap.as_function_type(),
                    INSTRUMENTATION_ANALYSIS_MODULE.to_string(),
                    trap.name.to_string(),
                );
                TargetCall::Pre(trap)
            }
            TargetCall::Post(trap) => {
                let trap = module.add_function_import(
                    trap.as_function_type(),
                    INSTRUMENTATION_ANALYSIS_MODULE.to_string(),
                    trap.name.to_string(),
                );
                TargetCall::Post(trap)
            }
            TargetCall::Both {
                pre_call_trap,
                post_call_trap,
            } => {
                let pre_call_trap = module.add_function_import(
                    pre_call_trap.as_function_type(),
                    INSTRUMENTATION_ANALYSIS_MODULE.to_string(),
                    pre_call_trap.name.to_string(),
                );
                let post_call_trap = module.add_function_import(
                    post_call_trap.as_function_type(),
                    INSTRUMENTATION_ANALYSIS_MODULE.to_string(),
                    post_call_trap.name.to_string(),
                );
                TargetCall::Both {
                    pre_call_trap,
                    post_call_trap,
                }
            }
        };
        target.instrument(module, target_functions)
    }
}

// TODO: since this code shares so much with the trait Instrumentable>>instrument_function_bodies
//       can I not merge them?
impl TargetCall<Idx<Function>> {
    fn instrument(
        &self,
        module: &mut Module,
        target_functions: &HashSet<Idx<Function>>,
    ) -> Result<(), InstrumentationError> {
        for target_function_idx in target_functions {
            let target_function = module.function(*target_function_idx);
            let code = target_function.code();
            match code {
                None => return Err(InstrumentationError::AttemptInstrumentImport),
                Some(code) => {
                    let high_level_body: HighLevelBody = ((&*module), target_function, code)
                        .try_into()
                        .map_err(|e| InstrumentationError::LowToHighError { low_to_high_err: e })?;
                    let high_level_body_transformed: HighLevelBody =
                        high_level_body.transform_call(self);
                    let LowLevelBody(transformed_low_level_body) =
                        high_level_body_transformed.into();

                    module.function_mut(*target_function_idx).code =
                        ImportOrPresent::Present(Code {
                            body: transformed_low_level_body,
                            locals: code.locals.clone(),
                        });
                }
            }
        }
        Ok(())
    }
}

impl HighLevelBody {
    #[must_use]
    pub fn transform_call(&self, target: &TargetCall<Idx<Function>>) -> Self {
        let Self(body) = self;
        let transformed_body = transform(body, target);
        Self(transformed_body)
    }
}

fn transform(body: &BodyInner, target: &TargetCall<Idx<Function>>) -> BodyInner {
    let mut result = Vec::new();

    for typed_instr @ TypedHighLevelInstr { instr, .. } in body {
        match (target, instr) {
            (TargetCall::Pre(pre_call_trap), Instr::Call(index)) => {
                result.extend_from_slice(&[
                    // STACK: [type_in]
                    typed_instr.instrument_with(Instr::Const(Val::I32(
                        i32::try_from(index.to_u32()).unwrap(),
                    ))),
                    // STACK: [type_in, f_idx]
                    typed_instr.instrument_with(Instr::Call(*pre_call_trap)),
                    // STACK: [type_in]
                    typed_instr.place_original(instr.clone()),
                    // STACK: [type_out]
                ]);
            }
            (TargetCall::Post(post_call_trap), Instr::Call(index)) => {
                result.extend_from_slice(&[
                    // STACK: [type_in]
                    typed_instr.place_original(instr.clone()),
                    // STACK: [type_out]
                    typed_instr.instrument_with(Instr::Const(Val::I32(
                        i32::try_from(index.to_u32()).unwrap(),
                    ))),
                    // STACK: [type_out, f_idx]
                    typed_instr.instrument_with(Instr::Call(*post_call_trap)),
                    // STACK: [type_out]
                ]);
            }
            (
                TargetCall::Both {
                    pre_call_trap,
                    post_call_trap,
                },
                Instr::Call(index),
            ) => {
                result.extend_from_slice(&[
                    // STACK: [type_in]
                    typed_instr.instrument_with(Instr::Const(Val::I32(
                        i32::try_from(index.to_u32()).unwrap(),
                    ))),
                    // STACK: [type_in, f_idx]
                    typed_instr.instrument_with(Instr::Call(*pre_call_trap)),
                    // STACK: [type_in]
                    typed_instr.place_original(instr.clone()),
                    // STACK: [type_out]
                    typed_instr.instrument_with(Instr::Const(Val::I32(
                        i32::try_from(index.to_u32()).unwrap(),
                    ))),
                    // STACK: [type_out, f_idx]
                    typed_instr.instrument_with(Instr::Call(*post_call_trap)),
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
