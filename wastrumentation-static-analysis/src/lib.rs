use std::collections::HashSet;

use wasabi_wasm::{Function, Idx, Module};
use wastrumentation::parse_nesting::{HighLevelBody, HighLevelInstr as Instr, TypedHighLevelInstr};

#[derive(Debug, Clone)]
pub enum Purity {
    Unknown(HighLevelBody),
    Pure,
    Impure,
}

#[derive(PartialEq, Eq, Debug)]
pub enum PurityEstimate {
    Unknown,
    Pure,
    Impure,
}

#[derive(Debug, Clone)]
pub struct AnalysisTargetFunction {
    index: Idx<Function>,
    purity: Purity,
}

pub fn immutable_functions_from_binary(module: &[u8]) -> Option<HashSet<u32>> {
    let (module, _, _) = wasabi_wasm::Module::from_bytes(module).ok()?;
    Some(immutable_functions(&module))
}

pub fn immutable_functions(module: &Module) -> HashSet<u32> {
    let mut high_level_functions: Vec<AnalysisTargetFunction> = module
        .functions()
        .map(|(index, function)| {
            if let Some(code) = function.code() {
                let module_function_code_index = (module, function, code, &index);
                let high_level_body = HighLevelBody::try_from(module_function_code_index).unwrap();
                AnalysisTargetFunction {
                    index,
                    purity: Purity::Unknown(high_level_body),
                }
            } else {
                AnalysisTargetFunction {
                    index,
                    purity: Purity::Impure,
                }
            }
        })
        .collect();

    let mut reiterate_no_more_findings = || {
        let reiterate: Vec<AnalysisTargetFunction> = high_level_functions
            .clone()
            .into_iter()
            .map(|f: AnalysisTargetFunction| match &f.purity {
                Purity::Pure => f,
                Purity::Impure => f,
                Purity::Unknown(body) => {
                    let HighLevelBody(body) = body;
                    let previous_purity = is_pure(f.index, body, &high_level_functions);
                    let purity = match previous_purity {
                        PurityEstimate::Unknown => f.purity,
                        PurityEstimate::Pure => Purity::Pure,
                        PurityEstimate::Impure => Purity::Impure,
                    };

                    AnalysisTargetFunction {
                        index: f.index,
                        purity,
                    }
                }
            })
            .collect();

        let pre_iteration_result: Vec<PurityEstimate> = high_level_functions
            .iter()
            .map(|analyzed_function| match analyzed_function.purity {
                Purity::Unknown(_) => PurityEstimate::Unknown,
                Purity::Pure => PurityEstimate::Pure,
                Purity::Impure => PurityEstimate::Impure,
            })
            .collect();
        let post_iteration_result: Vec<PurityEstimate> = reiterate
            .iter()
            .map(|analyzed_function| match analyzed_function.purity {
                Purity::Unknown(_) => PurityEstimate::Unknown,
                Purity::Pure => PurityEstimate::Pure,
                Purity::Impure => PurityEstimate::Impure,
            })
            .collect();

        high_level_functions = reiterate;
        pre_iteration_result == post_iteration_result
    };

    let mut iteration = 0;
    loop {
        if reiterate_no_more_findings() {
            break;
        } else {
            iteration += 1;
            println!("New finding, iteration {iteration}");
        }
    }

    high_level_functions
        .iter()
        .filter(|analyzed_function| matches!(analyzed_function.purity, Purity::Pure))
        .map(|analyzed_function| analyzed_function.index.to_u32())
        .collect()
}

// Is pure when the instructions of that function
// - Do not access global values &&
// - Do not call imported function &&
// - Do not access tables/memory &&
// - Does not perform call indirect (here's potential room for improvement)
fn is_pure(
    index: Idx<Function>,
    high_level_body: &Vec<TypedHighLevelInstr>,
    analysis_target_functions: &Vec<AnalysisTargetFunction>,
) -> PurityEstimate {
    for instr in high_level_body {
        match &instr.instr {
            // Must continue traversal
            Instr::If(_, body, None) | Instr::Block(_, body) | Instr::Loop(_, body) => {
                match is_pure(index, body, analysis_target_functions) {
                    PurityEstimate::Unknown => return PurityEstimate::Unknown,
                    PurityEstimate::Impure => return PurityEstimate::Impure,
                    PurityEstimate::Pure => continue,
                }
            }
            Instr::If(_, then, Some(else_)) => {
                match is_pure(index, then, analysis_target_functions) {
                    PurityEstimate::Unknown => return PurityEstimate::Unknown,
                    PurityEstimate::Impure => return PurityEstimate::Impure,
                    PurityEstimate::Pure => (), // continue, does same hold for `else` ?
                };
                match is_pure(index, else_, analysis_target_functions) {
                    PurityEstimate::Unknown => return PurityEstimate::Unknown,
                    PurityEstimate::Impure => return PurityEstimate::Impure,
                    PurityEstimate::Pure => continue,
                };
            }
            // Pure
            Instr::BrTable {
                table: _,
                default: _,
            }
            | Instr::Br(_)
            | Instr::BrIf(_)
            | Instr::Unreachable
            | Instr::Nop
            | Instr::Return
            | Instr::RefNull(_)
            | Instr::RefIsNull
            | Instr::RefFunc(_)
            | Instr::Drop
            | Instr::Select
            | Instr::TypedSelect(_)
            | Instr::Const(_)
            | Instr::Unary(_)
            | Instr::Local(_, _)
            | Instr::Binary(_) => continue,
            // Must check purity of other function, important: do not 'decide' purity of other, since recursive calls may introduce infinite loop
            Instr::Call(target_index) => {
                if index == *target_index {
                    continue;
                }

                match analysis_target_functions
                    .iter()
                    .find(|analysis_target_function| {
                        analysis_target_function.index == *target_index
                    })
                    .expect("Call target must be present in binary")
                    .purity
                {
                    Purity::Unknown(_) => return PurityEstimate::Unknown,
                    Purity::Impure => return PurityEstimate::Impure,
                    Purity::Pure => continue,
                }
            }
            // Operations below are impure, read / write from non-local state
            Instr::Global(_, _)
            | Instr::TableGet(_)
            | Instr::TableSet(_)
            | Instr::TableSize(_)
            | Instr::TableGrow(_)
            | Instr::TableFill(_)
            | Instr::TableCopy(_, _)
            | Instr::TableInit(_, _)
            | Instr::ElemDrop(_)
            | Instr::Load(_, _)
            | Instr::Store(_, _)
            | Instr::MemorySize(_)
            | Instr::MemoryGrow(_)
            | Instr::MemoryFill
            | Instr::MemoryCopy
            | Instr::MemoryInit(_)
            | Instr::DataDrop(_) => return PurityEstimate::Impure,
            // Optimization? If I can determine the target table is 'pure', then this could be a `true`
            Instr::CallIndirect(_, _) => return PurityEstimate::Impure,
        }
    }
    PurityEstimate::Pure
}

// TODO: implement tests
