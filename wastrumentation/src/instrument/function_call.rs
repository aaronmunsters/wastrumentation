use std::collections::HashSet;

use crate::analysis::WasmExport;
use crate::parse_nesting::{HighLevelBody, Instr, LowLevelBody};
use wasabi_wasm::{Code, Function, Idx, ImportOrPresent, Module, Val};

use super::{function_application::INSTRUMENTATION_ANALYSIS_MODULE, FunctionTypeConvertible};

type CallTransformationError = &'static str;

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
    ) -> Result<(), CallTransformationError> {
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

impl TargetCall<Idx<Function>> {
    fn instrument(
        &self,
        module: &mut Module,
        target_functions: &HashSet<Idx<Function>>,
    ) -> Result<(), CallTransformationError> {
        for target_function_idx in target_functions {
            let target_function = module.function_mut(*target_function_idx);
            if target_function.code().is_none() {
                return Err("Attempt to instrument call on import function");
            }

            let code = target_function.code_mut().unwrap(); // checked above
            let high_level_body: HighLevelBody = LowLevelBody(code.body.clone()).try_into()?;
            let high_level_body_transformed = high_level_body.transform_call(self);
            let LowLevelBody(transformed_low_level_body) = high_level_body_transformed.into();

            target_function.code = ImportOrPresent::Present(Code {
                body: transformed_low_level_body,
                locals: code.locals.clone(),
            });
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

fn transform(body: &Vec<Instr>, target: &TargetCall<Idx<Function>>) -> Vec<Instr> {
    let mut result = Vec::new();

    for instr in body {
        match (target, instr) {
            (TargetCall::Pre(pre_call_trap), Instr::Call(index)) => {
                result.extend_from_slice(&[
                    // STACK: [type_in]
                    Instr::Const(Val::I32(i32::try_from(index.to_u32()).unwrap())),
                    // STACK: [type_in, f_idx]
                    Instr::Call(*pre_call_trap),
                    // STACK: [type_in]
                    instr.clone(),
                    // STACK: [type_out]
                ]);
            }
            (TargetCall::Post(post_call_trap), Instr::Call(index)) => {
                result.extend_from_slice(&[
                    // STACK: [type_in]
                    instr.clone(),
                    // STACK: [type_out]
                    Instr::Const(Val::I32(i32::try_from(index.to_u32()).unwrap())),
                    // STACK: [type_out, f_idx]
                    Instr::Call(*post_call_trap),
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
                    Instr::Const(Val::I32(i32::try_from(index.to_u32()).unwrap())),
                    // STACK: [type_in, f_idx]
                    Instr::Call(*pre_call_trap),
                    // STACK: [type_in]
                    instr.clone(),
                    // STACK: [type_out]
                    Instr::Const(Val::I32(i32::try_from(index.to_u32()).unwrap())),
                    // STACK: [type_out, f_idx]
                    Instr::Call(*post_call_trap),
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
    result
}

// TODO: implement tests
