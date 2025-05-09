// The stack library provides the following functionality per instrumented signature:
//
// allocate ...args -> i32
// load_arg_`arg-types`_args_`ret-types`_arg_`n` -> `n-type`
// store_arg_`arg-types`_args_`ret-types`_arg_`n` -> `n-type`
//

use crate::instrument::function_application::INSTRUMENTATION_STACK_MODULE;

use crate::compiler::{LibGeneratable, Library};
use crate::wasm_constructs::{RefType as LibGenRefType, Signature, SignatureSide, WasmType};
use std::collections::{HashMap, HashSet};
use wasabi_wasm::{Function, FunctionType, Idx, Module, RefType, ValType};

// TODO: use some macro's here to generate most of the boilerplate -> this makes it also more maintainable
// TODO: tie this together with the generation library, ie. get names from there!

pub struct StackLibrary<Language: LibGeneratable> {
    pub signature_import_links: HashMap<FunctionType, ModuleLinkedStackHooks>,
    pub library: Library<Language>,
}

impl<Language: LibGeneratable> StackLibrary<Language> {
    pub fn from_module(module: &mut Module, functions: &HashSet<Idx<Function>>) -> Self {
        let signature_import_links: HashMap<FunctionType, ModuleLinkedStackHooks> =
            functions.iter().fold(HashMap::new(), |mut acc, index| {
                let function_type = module.function(*index).type_;
                if acc.contains_key(&function_type) {
                    return acc;
                }
                let stack_library =
                    ModuleLinkedStackHooks::from_function_type_module(function_type, module);
                acc.insert(function_type, stack_library);
                acc
            });
        let signatures: Vec<Signature> = signature_import_links
            .keys()
            .map(WasabiFunctionType)
            .map(Into::into)
            .collect();

        let library = Language::generate_lib(&signatures);
        Self {
            signature_import_links,
            library,
        }
    }
}

struct WasabiFunctionType<'a>(&'a FunctionType);

impl WasabiFunctionType<'_> {
    fn val_type_to_wasm_type(v: ValType) -> WasmType {
        match v {
            ValType::I32 => WasmType::I32,
            ValType::I64 => WasmType::I64,
            ValType::F32 => WasmType::F32,
            ValType::F64 => WasmType::F64,
            ValType::Ref(r) => WasmType::Ref(Self::convert_reftype(r)),
        }
    }

    fn convert_reftype(r: RefType) -> LibGenRefType {
        match r {
            RefType::ExternRef => LibGenRefType::ExternRef,
            RefType::FuncRef => LibGenRefType::FuncRef,
        }
    }
}

impl From<WasabiFunctionType<'_>> for Signature {
    fn from(value: WasabiFunctionType) -> Self {
        let WasabiFunctionType(function_type) = value;
        Self {
            return_types: function_type
                .results()
                .iter()
                .map(|v: &ValType| WasabiFunctionType::val_type_to_wasm_type(*v))
                .collect(),
            argument_types: function_type
                .inputs()
                .iter()
                .map(|v: &ValType| WasabiFunctionType::val_type_to_wasm_type(*v))
                .collect(),
        }
    }
}

// TODO: remove the dead code, this might be related to the specialized instrumentation code
pub struct ModuleLinkedStackHooks {
    #[allow(dead_code)]
    pub function_type: FunctionType,
    pub allocate_values_buffer: Idx<Function>,
    pub allocate_types_buffer: Idx<Function>,
    pub free_values_buffer: Idx<Function>,
    pub free_types_buffer: Idx<Function>,
    pub arg_load_n: Vec<Idx<Function>>,
    #[allow(dead_code)]
    pub arg_store_n: Vec<Idx<Function>>,
    pub ret_load_n: Vec<Idx<Function>>,
    #[allow(dead_code)]
    pub ret_store_n: Vec<Idx<Function>>,
    pub ret_store_all: Idx<Function>,
}

impl ModuleLinkedStackHooks {
    /// This will add the known imports to the function
    pub fn from_function_type_module(function_type: FunctionType, module: &mut Module) -> Self {
        (function_type, module).into()
    }
}

impl From<(FunctionType, &mut Module)> for ModuleLinkedStackHooks {
    fn from((function_type, module): (FunctionType, &mut Module)) -> Self {
        let lib_gen_signature: Signature = WasabiFunctionType(&function_type).into();
        let allocate_values_buffer_type =
            FunctionType::new(function_type.inputs(), &[ValType::I32]);
        let allocate_values_buffer = module.add_function_import(
            allocate_values_buffer_type,
            INSTRUMENTATION_STACK_MODULE.into(),
            lib_gen_signature.generate_allocate_values_buffer_name(),
        );

        let free_values_buffer_type = FunctionType::new(&[ValType::I32], &[]);
        let free_values_buffer = module.add_function_import(
            free_values_buffer_type,
            INSTRUMENTATION_STACK_MODULE.into(),
            lib_gen_signature.generate_free_values_buffer_name(),
        );

        let allocate_types_buffer_type = FunctionType::new(&[], &[ValType::I32]);
        let allocate_types_buffer = module.add_function_import(
            allocate_types_buffer_type,
            INSTRUMENTATION_STACK_MODULE.into(),
            lib_gen_signature.generate_allocate_types_buffer_name(),
        );

        let free_types_buffer_type = FunctionType::new(&[ValType::I32], &[]);
        let free_types_buffer = module.add_function_import(
            free_types_buffer_type,
            INSTRUMENTATION_STACK_MODULE.into(),
            lib_gen_signature.generate_free_types_buffer_name(),
        );

        let arg_load_n = function_type
            .inputs()
            .iter()
            .enumerate()
            .map(|(index, val_type)| {
                module.add_function_import(
                    FunctionType::new(&[ValType::I32], &[*val_type]),
                    INSTRUMENTATION_STACK_MODULE.into(),
                    lib_gen_signature.generate_load_name(SignatureSide::Argument, index),
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
                    lib_gen_signature.generate_load_name(SignatureSide::Return, index),
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
                    lib_gen_signature.generate_store_name(SignatureSide::Argument, index),
                )
            })
            .collect();

        let ret_store_n = function_type
            .results()
            .iter()
            .enumerate()
            .map(|(index, val_type)| {
                module.add_function_import(
                    FunctionType::new(&[ValType::I32, *val_type], &[]),
                    INSTRUMENTATION_STACK_MODULE.into(),
                    lib_gen_signature.generate_store_name(SignatureSide::Return, index),
                )
            })
            .collect();

        let mut store_rets_signature = vec![ValType::I32];
        store_rets_signature.extend(function_type.results());
        let ret_store_all = module.add_function_import(
            FunctionType::new(&store_rets_signature, &[]),
            INSTRUMENTATION_STACK_MODULE.into(),
            lib_gen_signature.generate_store_rets_name(),
        );

        ModuleLinkedStackHooks {
            function_type,
            allocate_values_buffer,
            allocate_types_buffer,
            free_values_buffer,
            free_types_buffer,
            arg_load_n,
            arg_store_n,
            ret_load_n,
            ret_store_n,
            ret_store_all,
        }
    }
}

// TODO: For tests, test hashing function types does not collide on similar signature
