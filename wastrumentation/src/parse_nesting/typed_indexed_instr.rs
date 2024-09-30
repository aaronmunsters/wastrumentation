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
pub fn type_inference_index_function(
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

#[cfg(test)]
mod tests {
    use super::*;
    use wasabi_wasm::Module;

    // FIXME: This is a bug in Wasabi :(

    #[test]
    fn test_generation() {
        const WAT: &str = r#"
          (module
            (func (export "meet-bottom")
              (block (result f64)
                (block (result f32)
                  (unreachable)
                  (br_table 0 1 1 (i32.const 1))
                )
                (drop)
                (f64.const 0)
              )
              (drop)
            )
          )
        "#;
        let wat = wat::parse_str(WAT).unwrap();
        let (module, _, _) = Module::from_bytes(&wat).unwrap();
        let meet_bottom = module.functions.first().unwrap();
        let meet_bottom_code = meet_bottom.code().unwrap();
        let err = type_inference_index_function(meet_bottom, meet_bottom_code, &module);
        assert!(matches!(err, Err(TypeError(_))))
    }
}
