// The stack library provides the following functionality per instrumented signature:
//
// allocate ...args -> i32
// load_arg_`arg-types`_args_`ret-types`_arg_`n` -> `n-type`
// store_arg_`arg-types`_args_`ret-types`_arg_`n` -> `n-type`
//

use self::stack_library_generator::*;
use crate::INSTRUMENTATION_STACK_MODULE;
use std::{collections::HashMap, fmt::Display};
use wasabi_wasm::{Function, FunctionType, Idx, Module, ValType};

// TODO: use some macro's here to generate most of the boilerplate -> this makes it also more maintainable

pub struct StackLibrary {
    pub allocate: Idx<Function>,
    pub allocate_types: Idx<Function>,
    pub free: Idx<Function>,
    pub arg_load_n: Vec<Idx<Function>>,
    pub arg_store_n: Vec<Idx<Function>>,
    pub arg_store_all: Idx<Function>,
    pub ret_load_n: Vec<Idx<Function>>,
    pub ret_store_n: Vec<Idx<Function>>,
    pub ret_store_all: Idx<Function>,
}

impl StackLibrary {
    pub fn from_function_type_module(function_type: FunctionType, module: &mut Module) -> Self {
        (function_type, module).into()
    }

    pub fn from_module(
        module: &mut Module,
        functions: &[Idx<Function>],
    ) -> HashMap<FunctionType, Self> {
        functions.iter().fold(HashMap::new(), |mut acc, index| {
            let function_type = module.function(*index).type_;
            if acc.contains_key(&function_type) {
                return acc;
            }
            let stack_library = StackLibrary::from_function_type_module(function_type, module);
            acc.insert(function_type, stack_library);
            acc
        })
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

    pub(super) fn generate_allocate_name(function_type: &FunctionType) -> String {
        generate_name("allocate", function_type)
    }

    pub(super) fn generate_allocate_types_name(function_type: &FunctionType) -> String {
        generate_name("allocate_types", function_type)
    }

    pub(super) fn generate_free_name(function_type: &FunctionType) -> String {
        generate_name("free", function_type)
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

impl<'a> From<(FunctionType, &mut Module)> for StackLibrary {
    fn from((function_type, module): (FunctionType, &mut Module)) -> Self {
        let allocate_type = FunctionType::new(function_type.inputs(), &[ValType::I32]);
        let allocate = module.add_function_import(
            allocate_type,
            INSTRUMENTATION_STACK_MODULE.into(),
            generate_allocate_name(&function_type),
        );

        let allocate_types_input_type =
            vec![ValType::I32; function_type.inputs().len() + function_type.results().len()];

        let allocate_types_type = FunctionType::new(&allocate_types_input_type, &[ValType::I32]);
        let allocate_types = module.add_function_import(
            allocate_types_type,
            INSTRUMENTATION_STACK_MODULE.into(),
            generate_allocate_types_name(&function_type),
        );

        let free_type = FunctionType::new(&[], &[]);
        let free = module.add_function_import(
            free_type,
            INSTRUMENTATION_STACK_MODULE.into(),
            generate_free_name(&function_type),
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

        StackLibrary {
            allocate,
            allocate_types,
            free,
            arg_load_n,
            arg_store_n,
            arg_store_all,
            ret_load_n,
            ret_store_n,
            ret_store_all,
        }
    }
}

// todo!
#[cfg(test)]
mod test {
    #[test]
    fn foo() {
        assert_eq!(1 + 2, 4)
    }
}
