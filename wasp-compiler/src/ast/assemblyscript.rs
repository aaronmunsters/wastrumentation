use std::fmt::Display;

use crate::ast::pest::{
    AdviceDefinition, ApplyGen, ApplyHookSignature, ApplySpe, TrapApply, TrapSignature,
    WasmParameter, WasmType, WaspRoot,
};

#[derive(Debug)]
pub struct TypeScriptProgram {
    pub content: String,
}

impl From<WaspRoot> for TypeScriptProgram {
    fn from(wasp_root: WaspRoot) -> Self {
        let WaspRoot(advice_definitions) = wasp_root;
        let mut program_content = String::new();
        for advice_definition in advice_definitions {
            program_content.push_str(&advice_definition.to_assemblyscript())
        }
        TypeScriptProgram {
            content: program_content,
        }
    }
}

impl AdviceDefinition {
    fn to_assemblyscript(&self) -> String {
        match self {
            AdviceDefinition::AdviceGlobal(program) => program.to_string(),
            AdviceDefinition::AdviceTrap(trap) => trap.to_assemblyscript(),
        }
    }
}

impl TrapSignature {
    fn to_assemblyscript(&self) -> String {
        let Self::TrapApply(TrapApply {
            apply_hook_signature,
            body,
        }) = self;
        match apply_hook_signature {
            ApplyHookSignature::Gen(apply_gen) => apply_gen.to_assemblyscript(body),
            ApplyHookSignature::Spe(apply_spe) => apply_spe.to_assemblyscript(body),
        }
    }
}

impl ApplyGen {
    fn to_assemblyscript(&self, body: &str) -> String {
        format!(
            r#"
            export function generic_apply(f_apply: i32, argc: i32, resc: i32, sigv: i32, sigtypv: i32): void {{
                let func = new WasmFunction(f_apply, sigtypv);
                let argsResults = new MutDynArgsResults(argc, resc, sigv, sigtypv);
                let args = new MutDynArgs(argsResults);
                let results = new MutDynRess(argsResults);
                {}
            }}
            "#,
            body
        ).to_string()
    }
}

impl Display for WasmType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let as_string = match self {
            WasmType::I32 => "i32",
            WasmType::F32 => "f32",
            WasmType::I64 => "i64",
            WasmType::F64 => "f64",
        };
        write!(f, "{as_string}")
    }
}

impl ApplySpe {
    fn to_assemblyscript(&self, body: &str) -> String {
        let full_signature_name = self.to_assemblyscript_full_signature_name();
        let args_signature = self.to_assemblyscript_args_signature();
        let ress_signature = self.to_assemblyscript_ress_signature();
        let external_call_base_name = format!("call_base_{full_signature_name}");
        let exported_func_inst_name = format!("apply_func_{full_signature_name}");
        format!(
            r#"
            @external("instrumented_input", "{external_call_base_name}")
            declare function {external_call_base_name}({args_signature}): {ress_signature};
            export function {exported_func_inst_name}({args_signature}): {ress_signature} {{
                let func = {external_call_base_name};
                {{
                    {body}
                }}
            }}
            "#
        )
    }

    fn to_assemblyscript_full_signature_name(&self) -> String {
        let Self {
            mutable_signature,
            parameters_arguments,
            parameters_results,
            ..
        } = self;
        let mutable_prefix = if *mutable_signature { "mut_" } else { "" };
        let wasm_pars_to_types_string = |arguments: &Vec<WasmParameter>| {
            let arguments_types: Vec<String> = arguments
                .iter()
                .map(|argument| argument.identifier_type.to_string())
                .collect();
            arguments_types.join("_")
        };

        let argument_types_string = wasm_pars_to_types_string(parameters_arguments);
        let arguments_types: &str = argument_types_string.as_str();

        let result_types_string = wasm_pars_to_types_string(parameters_results);
        let results_types: &str = result_types_string.as_str();

        format!("{mutable_prefix}args_{arguments_types}_ress_{results_types}")
    }

    fn to_assemblyscript_args_signature(&self) -> String {
        let parameters: Vec<String> = self
            .parameters_arguments
            .iter()
            .map(|a| format!("{}: {}", a.identifier, a.identifier_type.to_string()))
            .collect();

        parameters.join(", ")
    }

    fn to_assemblyscript_ress_signature(&self) -> String {
        let results: Vec<String> = self
            .parameters_results
            .iter()
            .map(|a| a.identifier_type.to_string())
            .collect();

        results.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::pest::{GenericTarget, WasmParameter, WasmType};

    #[test]
    fn generate_apply_spe_mut() {
        let ast: TrapSignature = TrapSignature::TrapApply(TrapApply {
            apply_hook_signature: ApplyHookSignature::Spe(ApplySpe {
                mutable_signature: true,
                apply_parameter: "func".into(),
                parameters_arguments: vec![
                    WasmParameter {
                        identifier: "a".into(),
                        identifier_type: WasmType::I32,
                    },
                    WasmParameter {
                        identifier: "b".into(),
                        identifier_type: WasmType::F32,
                    },
                    WasmParameter {
                        identifier: "c".into(),
                        identifier_type: WasmType::I64,
                    },
                ],
                parameters_results: vec![WasmParameter {
                    identifier: "r".into(),
                    identifier_type: WasmType::F64,
                }],
            }),
            body: " console.log(a); return func(a); ".into(),
        });

        assert_eq!(
            ast.to_assemblyscript(),
            r#"
            @external("instrumented_input", "call_base_mut_args_i32_f32_i64_ress_f64")
            declare function call_base_mut_args_i32_f32_i64_ress_f64(a: i32, b: f32, c: i64): f64;
            export function apply_func_mut_args_i32_f32_i64_ress_f64(a: i32, b: f32, c: i64): f64 {
                let func = call_base_mut_args_i32_f32_i64_ress_f64;
                {
                     console.log(a); return func(a); 
                }
            }
            "#
        )
    }

    #[test]
    fn generate_apply_spe_imut() {
        let ast: TrapSignature = TrapSignature::TrapApply(TrapApply {
            apply_hook_signature: ApplyHookSignature::Spe(ApplySpe {
                mutable_signature: false,
                apply_parameter: "func".into(),
                parameters_arguments: vec![
                    WasmParameter {
                        identifier: "a".into(),
                        identifier_type: WasmType::I32,
                    },
                    WasmParameter {
                        identifier: "b".into(),
                        identifier_type: WasmType::F32,
                    },
                    WasmParameter {
                        identifier: "c".into(),
                        identifier_type: WasmType::I64,
                    },
                ],
                parameters_results: vec![WasmParameter {
                    identifier: "r".into(),
                    identifier_type: WasmType::F64,
                }],
            }),
            body: " console.log(a); return func(a); ".into(),
        });

        assert_eq!(
            ast.to_assemblyscript(),
            r#"
            @external("instrumented_input", "call_base_args_i32_f32_i64_ress_f64")
            declare function call_base_args_i32_f32_i64_ress_f64(a: i32, b: f32, c: i64): f64;
            export function apply_func_args_i32_f32_i64_ress_f64(a: i32, b: f32, c: i64): f64 {
                let func = call_base_args_i32_f32_i64_ress_f64;
                {
                     console.log(a); return func(a); 
                }
            }
            "#
        )
    }

    #[test]
    fn generate_apply_gen() {
        let ast: TrapSignature = TrapSignature::TrapApply(TrapApply {
            apply_hook_signature: ApplyHookSignature::Gen(ApplyGen {
                generic_means: GenericTarget::MutableDynamic,
                parameter_apply: "func".to_string(),
                parameter_arguments: "args".to_string(),
                parameter_results: "results".to_string(),
            }),
            body: " console.log(args.get<i32>(0)); func.apply(); ".into(),
        });

        assert_eq!(
            ast.to_assemblyscript(),
            r#"
            export function generic_apply(f_apply: i32, argc: i32, resc: i32, sigv: i32, sigtypv: i32): void {
                let func = new WasmFunction(f_apply, sigtypv);
                let argsResults = new MutDynArgsResults(argc, resc, sigv, sigtypv);
                let args = new MutDynArgs(argsResults);
                let results = new MutDynRess(argsResults);
                 console.log(args.get<i32>(0)); func.apply(); 
            }
            "#
        )
    }

    #[test]
    fn from_input_program() {
        let input_program = r#"
        (aspect
            (global >>>GUEST>>>
                console.log("Hello world!");
            <<<GUEST<<<)
        
            (advice apply (func    WasmFunction)
                          (args    MutDynArgs)
                          (results MutDynResults) >>>GUEST>>>
                 console.log(args.get<i32>(0));
                 func.apply();
            <<<GUEST<<<)
        )
        "#;
        let typescript_program = TypeScriptProgram::try_from(input_program).unwrap();

        assert_eq!(
            typescript_program.content,
            r#"
                console.log("Hello world!");
            
            export function generic_apply(f_apply: i32, argc: i32, resc: i32, sigv: i32, sigtypv: i32): void {
                let func = new WasmFunction(f_apply, sigtypv);
                let argsResults = new MutDynArgsResults(argc, resc, sigv, sigtypv);
                let args = new MutDynArgs(argsResults);
                let results = new MutDynRess(argsResults);
                
                 console.log(args.get<i32>(0));
                 func.apply();
            
            }
            "#
        );
    }

    #[test]
    fn should_debug() {
        let typescript_program = TypeScriptProgram {
            content: r#"console.log(43)"#.to_string(),
        };
        assert_eq!(
            format!("{typescript_program:?}"),
            r#"TypeScriptProgram { content: "console.log(43)" }"#
        )
    }
}
