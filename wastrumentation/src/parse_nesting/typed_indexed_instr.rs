use wasabi_wasm::{
    types::{InferredInstructionType, TypeChecker, TypeError},
    Code, Function, Instr, Module,
};

pub struct TypedIndexedInstr {
    pub index: usize,
    pub type_: InferredInstructionType,
    pub instr: Instr,
}

/// Type checks all instructions in a `function`.
pub fn type_inference_function(
    function: &Function,
    code: &Code,
    module: &Module,
) -> Result<Vec<TypedIndexedInstr>, TypeError> {
    let mut type_checker = TypeChecker::begin_function(function, module);
    code.body
        .iter()
        .enumerate()
        .map(|(index, instr)| {
            type_checker
                .check_next_instr(instr)
                .map(|type_| TypedIndexedInstr {
                    index,
                    type_,
                    instr: instr.clone(),
                })
                // Add type error location information.
                .map_err(|e| {
                    let TypeError(mut err) = e;
                    err.instruction_idx = Some(index.into());
                    err.instruction = Some(instr.clone());
                    err.function_name = function.name.clone();
                    TypeError(err)
                })
        })
        .collect()
}
