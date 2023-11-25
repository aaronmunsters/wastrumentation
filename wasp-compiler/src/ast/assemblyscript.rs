use std::fmt::Display;

use crate::{
    ast::wasp::{
        AdviceDefinition, ApplyGen, ApplyHookSignature, ApplySpe, TrapApply, TrapSignature,
        WasmParameter, WasmType, WaspRoot,
    },
    wasp_interface::{WasmExport, WasmImport, GENERIC_APPLY_FUNCTION_NAME, TRANSFORMED_INPUT_NS},
};

use crate::util::Alphabetical;

const STD_INSTRUMENTATION_LIB: &str = include_str!("../std_instrumentation_lib.ts");

#[derive(Debug, PartialEq, Eq)]
pub struct AssemblyScriptProgram {
    pub content: String,
}

impl From<WaspRoot> for AssemblyScriptProgram {
    fn from(wasp_root: WaspRoot) -> Self {
        let mut program_content = if wasp_root.has_generic_apply() {
            String::from(STD_INSTRUMENTATION_LIB)
        } else {
            String::new()
        };
        let WaspRoot(advice_definitions) = wasp_root;
        for advice_definition in advice_definitions {
            program_content.push_str(&advice_definition.to_assemblyscript())
        }
        AssemblyScriptProgram {
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
            export function {GENERIC_APPLY_FUNCTION_NAME}(f_apply: i32, argc: i32, resc: i32, sigv: i32, sigtypv: i32): void {{
                let func = new WasmFunction(f_apply, sigtypv);
                let argsResults = new MutDynArgsResults(argc, resc, sigv, sigtypv);
                let args = new MutDynArgs(argsResults);
                let results = new MutDynRess(argsResults);
                {body}
            }}
            "#,
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

fn signature_to_string(
    mutable_signature: bool,
    parameters_arguments: &Vec<WasmParameter>,
    parameters_results: &Vec<WasmParameter>,
) -> String {
    let mutable_prefix = if mutable_signature { "mut_" } else { "" };
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

impl WasmImport {
    pub fn for_extern_call_base(
        mutable_signature: bool,
        parameters_arguments: &Vec<WasmParameter>,
        parameters_results: &Vec<WasmParameter>,
    ) -> Self {
        let full_signature_name =
            signature_to_string(mutable_signature, parameters_arguments, parameters_results);
        let name = format!("call_base_{full_signature_name}");

        Self {
            namespace: TRANSFORMED_INPUT_NS.into(),
            name,
            args: WasmParameterVec(parameters_arguments).types(),
            results: WasmParameterVec(parameters_results).types(),
        }
    }

    fn to_assemblyscript_declaration(&self) -> String {
        let Self {
            namespace,
            name,
            args,
            results,
        } = self;
        let args_signature = args
            .iter()
            .enumerate()
            .map(|(index, wasm_type)| format!("{}: {wasm_type}", index.to_alphabetic()))
            .collect::<Vec<String>>()
            .join(", ");

        let ress_signature = results
            .iter()
            .map(WasmType::to_string)
            .collect::<Vec<String>>()
            .join(", ");
        format!(
            r#"
        @external("{namespace}", "{name}")
        declare function {name}({args_signature}): {ress_signature};        
        "#
        )
    }
}

impl WasmExport {
    pub fn for_exported_apply_trap(
        mutable_signature: bool,
        parameters_arguments: &Vec<WasmParameter>,
        parameters_results: &Vec<WasmParameter>,
    ) -> Self {
        let full_signature_name =
            signature_to_string(mutable_signature, parameters_arguments, parameters_results);
        let name = format!("apply_func_{full_signature_name}");
        WasmExport {
            name,
            args: WasmParameterVec(parameters_arguments).types(),
            results: WasmParameterVec(parameters_results).types(),
        }
    }
}

struct WasmParameterVec<'a>(&'a [WasmParameter]);

impl<'a> WasmParameterVec<'a> {
    fn types(&self) -> Vec<WasmType> {
        let Self(vec) = self;
        vec.iter()
            .map(|wasm_parameter| wasm_parameter.identifier_type)
            .collect()
    }
}

impl ApplySpe {
    fn to_assemblyscript(&self, body: &str) -> String {
        let Self {
            mutable_signature,
            parameters_arguments,
            parameters_results,
            ..
        } = self;

        let import = WasmImport::for_extern_call_base(
            *mutable_signature,
            parameters_arguments,
            parameters_results,
        );

        let export = WasmExport::for_exported_apply_trap(
            *mutable_signature,
            parameters_arguments,
            parameters_results,
        );

        let import_declaration = import.to_assemblyscript_declaration();
        let external_call_base_name = import.name;

        let args_signature = self.to_assemblyscript_args_signature();
        let ress_signature = self.to_assemblyscript_ress_signature();

        let exported_func_inst_name = export.name;

        format!(
            r#"
            {import_declaration}
            export function {exported_func_inst_name}({args_signature}): {ress_signature} {{
                let func = {external_call_base_name};
                {{
                    {body}
                }}
            }}
            "#
        )
    }

    fn to_assemblyscript_args_signature(&self) -> String {
        let parameters: Vec<String> = self
            .parameters_arguments
            .iter()
            .map(|a| format!("{}: {}", a.identifier, a.identifier_type))
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
    use crate::ast::wasp::{GenericTarget, WasmParameter, WasmType};

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
            format!(
                r#"
            
        @external("transformed_input", "call_base_mut_args_i32_f32_i64_ress_f64")
        declare function call_base_mut_args_i32_f32_i64_ress_f64(a: i32, b: f32, c: i64): f64;        
        
            export function apply_func_mut_args_i32_f32_i64_ress_f64(a: i32, b: f32, c: i64): f64 {{
                let func = call_base_mut_args_i32_f32_i64_ress_f64;
                {{
                     console.log(a); return func(a); 
                }}
            }}
            "#
            )
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
            format!(
                r#"
            
        @external("transformed_input", "call_base_args_i32_f32_i64_ress_f64")
        declare function call_base_args_i32_f32_i64_ress_f64(a: i32, b: f32, c: i64): f64;        
        
            export function apply_func_args_i32_f32_i64_ress_f64(a: i32, b: f32, c: i64): f64 {{
                let func = call_base_args_i32_f32_i64_ress_f64;
                {{
                     console.log(a); return func(a); 
                }}
            }}
            "#
            )
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
        let assemblyscript_program = AssemblyScriptProgram::try_from(input_program).unwrap();
        let mut expected_outcome = String::from(STD_INSTRUMENTATION_LIB);
        expected_outcome.push_str(
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

        assert_eq!(assemblyscript_program.content, expected_outcome);
    }

    #[test]
    fn should_debug() {
        let assemblyscript_program = AssemblyScriptProgram {
            content: r#"console.log(43)"#.to_string(),
        };
        assert_eq!(
            format!("{assemblyscript_program:?}"),
            r#"AssemblyScriptProgram { content: "console.log(43)" }"#
        )
    }
}
