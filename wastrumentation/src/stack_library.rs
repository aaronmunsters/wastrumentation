// The stack library provides the following functionality per instrumented signature:
//
// allocate ...args -> i32
// load_arg_`arg-types`_args_`ret-types`_arg_`n` -> `n-type`
// store_arg_`arg-types`_args_`ret-types`_arg_`n` -> `n-type`
//

use self::stack_library_generator::*;
use crate::INSTRUMENTATION_STACK_MODULE;
use std::{collections::HashMap, fmt::Display};
use wasabi_wasm::{Function, FunctionType, Idx, Module, RefType, ValType};
use wastrumentation_instr_lib::{generate_lib, Signature, WasmType};

// TODO: use some macro's here to generate most of the boilerplate -> this makes it also more maintainable
// TODO: tie this together with the generation library, ie. get names from there!

pub struct StackLibrary {
    pub signature_import_links: HashMap<FunctionType, SignatureStackLibrary>,
    pub assemblyscript_code: String,
}

impl StackLibrary {
    pub fn from_module(module: &mut Module, functions: &[Idx<Function>]) -> Self {
        let signature_import_links: HashMap<FunctionType, SignatureStackLibrary> =
            functions.iter().fold(HashMap::new(), |mut acc, index| {
                let function_type = module.function(*index).type_;
                if acc.contains_key(&function_type) {
                    return acc;
                }
                if function_type.results().is_empty() && function_type.inputs().is_empty() {
                    return acc;
                }
                let stack_library =
                    SignatureStackLibrary::from_function_type_module(function_type, module);
                acc.insert(function_type, stack_library);
                acc
            });
        let signatures: Vec<Signature> = signature_import_links
            .keys()
            .map(WasabiFunctionType)
            .map(Into::into)
            .collect();
        let assemblyscript_code = generate_lib(&signatures);
        Self {
            signature_import_links,
            assemblyscript_code,
        }
    }
}

struct WasabiFunctionType<'a>(&'a FunctionType);

impl WasabiFunctionType<'_> {
    fn val_type_to_wasm_type(v: &ValType) -> WasmType {
        match v {
            ValType::I32 => WasmType::I32,
            ValType::I64 => WasmType::I64,
            ValType::F32 => WasmType::F32,
            ValType::F64 => WasmType::F64,
            ValType::Ref(r) => WasmType::Ref(Self::convert_reftype(r)),
        }
    }

    fn convert_reftype(r: &RefType) -> wastrumentation_instr_lib::RefType {
        match r {
            RefType::ExternRef => wastrumentation_instr_lib::RefType::ExternRef,
            RefType::FuncRef => wastrumentation_instr_lib::RefType::FuncRef,
        }
    }
}

impl<'a> From<WasabiFunctionType<'a>> for Signature {
    fn from(value: WasabiFunctionType) -> Self {
        let WasabiFunctionType(function_type) = value;
        Self {
            return_types: function_type
                .results()
                .iter()
                .map(WasabiFunctionType::val_type_to_wasm_type)
                .collect(),
            argument_types: function_type
                .inputs()
                .iter()
                .map(WasabiFunctionType::val_type_to_wasm_type)
                .collect(),
        }
    }
}

pub struct SignatureStackLibrary {
    pub function_type: FunctionType,
    pub allocate_values_buffer: Idx<Function>,
    pub allocate_types_buffer: Idx<Function>,
    pub free_values_buffer: Idx<Function>,
    pub free_types_buffer: Idx<Function>,
    pub arg_load_n: Vec<Idx<Function>>,
    pub arg_store_n: Vec<Idx<Function>>,
    pub arg_store_all: Idx<Function>,
    pub ret_load_n: Vec<Idx<Function>>,
    pub ret_store_n: Vec<Idx<Function>>,
    pub ret_store_all: Idx<Function>,
}

impl SignatureStackLibrary {
    /// This will add the known imports to the function
    pub fn from_function_type_module(function_type: FunctionType, module: &mut Module) -> Self {
        (function_type, module).into()
    }
}

enum SignatureSide {
    Return,
    Argument,
}

impl Display for SignatureSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::Return => "ret",
            Self::Argument => "arg",
        };
        write!(f, "{}", str)
    }
}

mod stack_library_generator {
    use wasabi_wasm::FunctionType;

    use super::SignatureSide;

    fn generate_name(name: &str, function_type: &FunctionType) -> String {
        format!(
            "{}_ret_{}_arg_{}",
            name,
            function_type
                .results()
                .iter()
                .map(|ft| ft.to_str())
                .collect::<Vec<&str>>()
                .join("_"),
            function_type
                .inputs()
                .iter()
                .map(|ft| ft.to_str())
                .collect::<Vec<&str>>()
                .join("_"),
        )
    }

    pub(super) fn generate_allocate_values_buffer_name(function_type: &FunctionType) -> String {
        generate_name("allocate", function_type)
    }

    pub(super) fn generate_allocate_types_buffer_name(function_type: &FunctionType) -> String {
        generate_name("allocate_types", function_type)
    }

    pub(super) fn generate_free_values_buffer_name(function_type: &FunctionType) -> String {
        generate_name("free_values", function_type)
    }

    pub(super) fn generate_free_types_buffer_name(function_type: &FunctionType) -> String {
        generate_name("free_types", function_type)
    }

    fn generate_indexed_name(
        name: &str,
        function_type: &FunctionType,
        signature_side: SignatureSide,
        index: usize,
    ) -> String {
        let prefix = format!("{}_{}{}", name, signature_side, index);
        generate_name(&prefix, function_type)
    }

    pub(super) fn generate_load_name(
        function_type: &FunctionType,
        signature_side: SignatureSide,
        index: usize,
    ) -> String {
        generate_indexed_name("load", function_type, signature_side, index)
    }

    pub(super) fn generate_store_name(
        function_type: &FunctionType,
        signature_side: SignatureSide,
        index: usize,
    ) -> String {
        generate_indexed_name("store", function_type, signature_side, index)
    }

    pub(super) fn generate_store_args_name(function_type: &FunctionType) -> String {
        generate_name("store_args", function_type)
    }

    pub(super) fn generate_store_rets_name(function_type: &FunctionType) -> String {
        generate_name("store_rets", function_type)
    }
}

impl<'a> From<(FunctionType, &mut Module)> for SignatureStackLibrary {
    fn from((function_type, module): (FunctionType, &mut Module)) -> Self {
        let allocate_values_buffer_type =
            FunctionType::new(function_type.inputs(), &[ValType::I32]);
        let allocate_values_buffer = module.add_function_import(
            allocate_values_buffer_type,
            INSTRUMENTATION_STACK_MODULE.into(),
            generate_allocate_values_buffer_name(&function_type),
        );

        let free_values_buffer_type = FunctionType::new(&[], &[]);
        let free_values_buffer = module.add_function_import(
            free_values_buffer_type,
            INSTRUMENTATION_STACK_MODULE.into(),
            generate_free_values_buffer_name(&function_type),
        );

        let allocate_types_buffer_type = FunctionType::new(&[], &[ValType::I32]);
        let allocate_types_buffer = module.add_function_import(
            allocate_types_buffer_type,
            INSTRUMENTATION_STACK_MODULE.into(),
            generate_allocate_types_buffer_name(&function_type),
        );

        let free_types_buffer_type = FunctionType::new(&[], &[]);
        let free_types_buffer = module.add_function_import(
            free_types_buffer_type,
            INSTRUMENTATION_STACK_MODULE.into(),
            generate_free_types_buffer_name(&function_type),
        );

        let arg_load_n = function_type
            .inputs()
            .iter()
            .enumerate()
            .map(|(index, val_type)| {
                module.add_function_import(
                    FunctionType::new(&[ValType::I32], &[*val_type]),
                    INSTRUMENTATION_STACK_MODULE.into(),
                    generate_load_name(&function_type, SignatureSide::Argument, index),
                )
            })
            .collect();

        let ret_load_n = function_type
            .results()
            .iter()
            .enumerate()
            .map(|(index, val_type)| {
                module.add_function_import(
                    FunctionType::new(&[ValType::I32], &[*val_type]),
                    INSTRUMENTATION_STACK_MODULE.into(),
                    generate_load_name(&function_type, SignatureSide::Return, index),
                )
            })
            .collect();

        let arg_store_n = function_type
            .inputs()
            .iter()
            .enumerate()
            .map(|(index, val_type)| {
                module.add_function_import(
                    FunctionType::new(&[ValType::I32, *val_type], &[]),
                    INSTRUMENTATION_STACK_MODULE.into(),
                    generate_store_name(&function_type, SignatureSide::Argument, index),
                )
            })
            .collect();

        let mut store_args_signature = vec![ValType::I32];
        store_args_signature.extend(function_type.inputs());
        let arg_store_all = module.add_function_import(
            FunctionType::new(&store_args_signature, &[]),
            INSTRUMENTATION_STACK_MODULE.into(),
            generate_store_args_name(&function_type),
        );

        let ret_store_n = function_type
            .results()
            .iter()
            .enumerate()
            .map(|(index, val_type)| {
                module.add_function_import(
                    FunctionType::new(&[ValType::I32, *val_type], &[]),
                    INSTRUMENTATION_STACK_MODULE.into(),
                    generate_store_name(&function_type, SignatureSide::Return, index),
                )
            })
            .collect();

        let mut store_rets_signature = vec![ValType::I32];
        store_rets_signature.extend(function_type.results());
        let ret_store_all = module.add_function_import(
            FunctionType::new(&store_rets_signature, &[]),
            INSTRUMENTATION_STACK_MODULE.into(),
            generate_store_rets_name(&function_type),
        );

        SignatureStackLibrary {
            function_type,
            allocate_values_buffer,
            allocate_types_buffer,
            free_values_buffer,
            free_types_buffer,
            arg_load_n,
            arg_store_n,
            arg_store_all,
            ret_load_n,
            ret_store_n,
            ret_store_all,
        }
    }
}
