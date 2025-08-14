#[cfg(test)]
mod test;

use std::{collections::HashSet, marker::PhantomData, ops::Deref, vec};

use crate::lib_compile::assemblyscript::AssemblyScript;

use wastrumentation::compiler::{LibGeneratable, Library};
use wastrumentation::wasm_constructs::{Signature, SignatureSide, WasmType};

impl LibGeneratable for AssemblyScript {
    fn generate_lib(signatures: &[Signature]) -> Library<Self> {
        Library::<Self> {
            content: generate_lib(signatures),
            language: PhantomData,
        }
    }
}

#[derive(Hash, PartialEq, Eq)]
pub struct WaspSignature<'a>(&'a Signature);

impl Deref for WaspSignature<'_> {
    type Target = Signature;

    fn deref(&self) -> &Self::Target {
        let Self(target) = self;
        target
    }
}

impl WaspSignature<'_> {
    fn mangled_assemblyscript_name_by_count(&self) -> String {
        let signature_rets_count = self.return_types.len();
        let signature_args_count = self.argument_types.len();
        format!("ret_{signature_rets_count}_arg_{signature_args_count}")
    }

    fn generic_assemblyscript_name(
        signature_rets_count: usize,
        signature_args_count: usize,
    ) -> String {
        format!("ret_{signature_rets_count}_arg_{signature_args_count}")
    }

    fn assemblyscript_comma_separated_types(&self) -> String {
        if self.is_empty() {
            String::new()
        } else {
            let comma_separated_types = self
                .return_types
                .iter()
                .chain(self.argument_types.iter())
                .map(ToString::to_string)
                .collect::<Vec<String>>()
                .join(", ");
            format!("<{comma_separated_types}>")
        }
    }

    fn assemblyscript_comma_separated_generics(rets_count: usize, args_count: usize) -> String {
        if rets_count + args_count == 0 {
            String::new()
        } else {
            format!(
                "<{}>",
                Self::assemblyscript_generics(rets_count, args_count).join(", ")
            )
        }
    }

    fn assemblyscript_generics(rets_count: usize, args_count: usize) -> Vec<String> {
        let arg_typs = (0..args_count).map(|n| format!("T{n}"));
        let ret_typs = (0..rets_count).map(|n| format!("R{n}"));
        ret_typs.chain(arg_typs).collect::<Vec<String>>()
    }

    fn assemblyscript_generic_typed_arguments(args_count: usize) -> Vec<String> {
        (0..args_count)
            .clone()
            .map(|n| format!("a{n}: T{n}"))
            .collect::<Vec<String>>()
    }

    fn assemblyscript_generic_comma_separated_typed_arguments(args_count: usize) -> String {
        Self::assemblyscript_generic_typed_arguments(args_count).join(", ")
    }
}

// Updated offset calculations to use index-based addressing for WasmValue unions
fn value_index_offset(position: usize) -> String {
    if position == 0 {
        "0".into()
    } else {
        position.to_string()
    }
}

fn arg_index_offset(arg_pos: usize, rets_count: usize) -> String {
    value_index_offset(rets_count + arg_pos)
}

fn ret_index_offset(ret_pos: usize) -> String {
    value_index_offset(ret_pos)
}

fn generate_allocate_generic(rets_count: usize, args_count: usize) -> String {
    let string_all_generics =
        WaspSignature::assemblyscript_comma_separated_generics(rets_count, args_count);
    let signature =
        WaspSignature::assemblyscript_generic_comma_separated_typed_arguments(args_count);
    let generic_name = WaspSignature::generic_assemblyscript_name(rets_count, args_count);

    let total_allocation = rets_count + args_count;

    let all_stores_followed_by_return = if args_count == 0 {
        "return stack_begin;".to_string()
    } else {
        (0..args_count)
            .map(|n| {
                let offset = arg_index_offset(n, rets_count);
                format!(
                    "// store a{n}
    wastrumentation_memory_store<T{n}>(stack_begin, a{n}, {offset});"
                )
            })
            .chain(vec!["return stack_begin;".into()])
            .collect::<Vec<String>>()
            .join("\n    ")
    };

    format!(
        "
@inline
function allocate_{generic_name}{string_all_generics}({signature}): usize {{
    const stack_begin = stack_allocate_values({total_allocation}); // inlined
    {all_stores_followed_by_return}
}}"
    )
}

fn generate_allocate_specialized(signature: &WaspSignature) -> String {
    // eg: Signature { return_type: [`i64`, `i32`], argument_types: [`f64`, `f32`] }
    // eg: [`f64`, `f32`]
    let signature_args = &signature.argument_types;
    // eg: `a0, a1`
    let args = signature_args
        .iter()
        .enumerate()
        .map(|(index, _ty)| format!("a{index}"))
        .collect::<Vec<String>>()
        .join(", ");
    // eg: `a0: f64, a1: f32`
    let signature_args_typs_ident = signature_args
        .iter()
        .enumerate()
        .map(|(index, ty)| format!("a{index}: {ty}"))
        .collect::<Vec<String>>()
        .join(", ");

    let comma_separated_types = signature.assemblyscript_comma_separated_types();
    let mangled_name = signature.generate_allocate_values_buffer_name();
    let mangled_by_count_name = signature.mangled_assemblyscript_name_by_count();

    format!(
        "
export function {mangled_name}({signature_args_typs_ident}): usize {{
    return allocate_{mangled_by_count_name}{comma_separated_types}({args});
}};
"
    )
}

fn generate_allocate_types_buffer_generic(rets_count: usize, args_count: usize) -> String {
    // Types are stored as i32 values, not WasmValues
    let total_allocation = rets_count + args_count;
    let generic_name = WaspSignature::generic_assemblyscript_name(rets_count, args_count);
    format!(
        "
@inline
function allocate_signature_types_buffer_{generic_name}(): usize {{
    const stack_begin = stack_allocate_types({total_allocation}); // inlined
    return stack_begin;
}}"
    )
}

fn generate_allocate_types_buffer_specialized(signature: &WaspSignature) -> String {
    // eg: [`i64`, `i32`]
    let signature_rets = &signature.return_types;
    // eg: [`f64`, `f32`]
    let signature_args = &signature.argument_types;
    // eg: `i64, i32, f64, f32`
    let all_stores_followed_by_return = signature_rets
        .iter()
        .chain(signature_args.iter())
        .map(WasmType::runtime_enum_value)
        .enumerate()
        .map(|(index, enum_value)| {
            format!("wastrumentation_stack_store_type(types_buffer, {index}, {enum_value});")
        })
        .chain(vec!["return types_buffer;".into()])
        .collect::<Vec<String>>()
        .join("\n    ");

    let mangled_name = signature.generate_allocate_types_buffer_name();
    let mangled_by_count_name = signature.mangled_assemblyscript_name_by_count();

    format!(
        "
export function {mangled_name}(): usize {{
    const types_buffer = allocate_signature_types_buffer_{mangled_by_count_name}();
    {all_stores_followed_by_return}
}}"
    )
}

fn generate_load_generic(rets_count: usize, args_count: usize) -> String {
    let string_all_generics =
        WaspSignature::assemblyscript_comma_separated_generics(rets_count, args_count);
    let generic_name = WaspSignature::generic_assemblyscript_name(rets_count, args_count);

    let all_arg_loads = (0..args_count).map(|n| {
        let an_offset = arg_index_offset(n, rets_count);
        format!(
            "
@inline
function load_arg{n}_{generic_name}{string_all_generics}(stack_ptr: usize): T{n} {{
    return wastrumentation_memory_load<T{n}>(stack_ptr, {an_offset});
}}"
        )
    });
    let all_ret_loads = (0..rets_count).map(|n| {
        let ar_offset = ret_index_offset(n);
        format!(
            "
@inline
function load_ret{n}_{generic_name}{string_all_generics}(stack_ptr: usize): R{n} {{
    return wastrumentation_memory_load<R{n}>(stack_ptr, {ar_offset});
}}"
        )
    });
    all_arg_loads
        .chain(all_ret_loads)
        .collect::<Vec<String>>()
        .join("\n")
}

fn generate_load_specialized(signature: &WaspSignature) -> String {
    // eg: [`i64`, `i32`]
    let signature_rets = &signature.return_types;
    // eg: [`f64`, `f32`]
    let signature_args = &signature.argument_types;

    let comma_separated_types = signature.assemblyscript_comma_separated_types();
    let mangled_by_count_name = signature.mangled_assemblyscript_name_by_count();

    let all_arg_loads = signature_args
        .iter()
        .enumerate()
        .map(|(index, arg_i_ret_type)| {
            let mangled_name = signature.generate_load_name(SignatureSide::Argument, index);
            format!(
                "
export function {mangled_name}(stack_ptr: usize): {arg_i_ret_type} {{
    return load_arg{index}_{mangled_by_count_name}{comma_separated_types}(stack_ptr);
}};"
            )
        });
    let all_ret_loads = signature_rets
        .iter()
        .enumerate()
        .map(|(index, ret_i_ret_type)| {
            let mangled_name = signature.generate_load_name(SignatureSide::Return, index);
            format!(
                "
export function {mangled_name}(stack_ptr: usize): {ret_i_ret_type} {{
    return load_ret{index}_{mangled_by_count_name}{comma_separated_types}(stack_ptr);
}};"
            )
        });

    all_arg_loads
        .chain(all_ret_loads)
        .collect::<Vec<String>>()
        .join("\n")
}

fn generate_store_generic(rets_count: usize, args_count: usize) -> String {
    let string_all_generics =
        WaspSignature::assemblyscript_comma_separated_generics(rets_count, args_count);
    let generic_name = WaspSignature::generic_assemblyscript_name(rets_count, args_count);

    let all_arg_stores = (0..args_count).map(|n| {
        let an_offset = arg_index_offset(n, rets_count);
        format!(
            "
@inline
function store_arg{n}_{generic_name}{string_all_generics}(stack_ptr: usize, a{n}: T{n}): void {{
    return wastrumentation_memory_store<T{n}>(stack_ptr, a{n}, {an_offset});
}}"
        )
    });
    let all_ret_stores = (0..rets_count).map(|n| {
        let ar_offset = ret_index_offset(n);
        format!(
            "
@inline
function store_ret{n}_{generic_name}{string_all_generics}(stack_ptr: usize, r{n}: R{n}): void {{
    return wastrumentation_memory_store<R{n}>(stack_ptr, r{n}, {ar_offset});
}}"
        )
    });
    all_arg_stores
        .chain(all_ret_stores)
        .collect::<Vec<String>>()
        .join("\n")
}

fn generate_store_specialized(signature: &WaspSignature) -> String {
    // eg: [`i64`, `i32`]
    let signature_rets = &signature.return_types;
    // eg: [`f64`, `f32`]
    let signature_args = &signature.argument_types;

    let comma_separated_types = signature.assemblyscript_comma_separated_types();
    let mangled_by_count_name = signature.mangled_assemblyscript_name_by_count();

    let all_arg_stores = signature_args
        .iter()
        .enumerate()
        .map(|(index, arg_i_ret_type)| {
            let mangled_name = signature.generate_store_name(SignatureSide::Argument, index);
            format!(
                "
export function {mangled_name}(stack_ptr: usize, a{index}: {arg_i_ret_type}): void {{
    return store_arg{index}_{mangled_by_count_name}{comma_separated_types}(stack_ptr, a{index});
}};"
            )
        });
    let all_ret_stores = signature_rets
        .iter()
        .enumerate()
        .map(|(index, ret_i_ret_type)| {
            let mangled_name = signature.generate_store_name(SignatureSide::Return, index);
            format!(
                "
export function {mangled_name}(stack_ptr: usize, a{index}: {ret_i_ret_type}): void {{
    return store_ret{index}_{mangled_by_count_name}{comma_separated_types}(stack_ptr, a{index});
}};"
            )
        });
    all_arg_stores
        .chain(all_ret_stores)
        .collect::<Vec<String>>()
        .join("\n")
}

fn generate_free_values_buffer_generic(rets_count: usize, args_count: usize) -> String {
    let string_all_generics =
        WaspSignature::assemblyscript_comma_separated_generics(rets_count, args_count);
    let generic_name = WaspSignature::generic_assemblyscript_name(rets_count, args_count);

    // Use the same allocation calculation as allocate function
    let total_allocation = rets_count + args_count;

    format!(
        "
@inline
function free_values_{generic_name}{string_all_generics}(ptr: usize): void {{
    stack_deallocate_values(ptr, {total_allocation}); // inlined
    return;
}}"
    )
}

fn generate_free_values_buffer_specialized(signature: &WaspSignature) -> String {
    format!(
        "
export function {}(ptr: usize): void {{
    return free_values_{}{}(ptr);
}};",
        signature.generate_free_values_buffer_name(),
        signature.mangled_assemblyscript_name_by_count(),
        signature.assemblyscript_comma_separated_types(),
    )
}

fn generate_free_types_buffer_generic(rets_count: usize, args_count: usize) -> String {
    let generic_name = WaspSignature::generic_assemblyscript_name(rets_count, args_count);
    let total_allocation = rets_count + args_count;

    format!(
        "
@inline
function free_types_{generic_name}(ptr: usize): void {{
    stack_deallocate_types(ptr, {total_allocation}); // inlined
    return;
}}"
    )
}

fn generate_free_types_buffer_specialized(signature: &WaspSignature) -> String {
    format!(
        "
export function {}(ptr: usize): void {{
    return free_types_{}(ptr);
}};",
        signature.generate_free_types_buffer_name(),
        signature.mangled_assemblyscript_name_by_count(),
    )
}

fn generate_store_rets_generic(rets_count: usize, args_count: usize) -> String {
    let string_all_generics =
        WaspSignature::assemblyscript_comma_separated_generics(rets_count, args_count);
    let generic_name = WaspSignature::generic_assemblyscript_name(rets_count, args_count);

    // eg: [`a0: R0`, `a1: R1`]
    let array_of_rets_signature = (0..rets_count).map(|n| format!("a{n}: R{n}"));
    let total_signature = (vec![String::from("stack_ptr: usize")])
        .into_iter()
        .chain(array_of_rets_signature)
        .collect::<Vec<String>>()
        .join(", ");

    let all_stores = if rets_count == 0 {
        "return;".to_string()
    } else {
        (0..rets_count)
            .flat_map(|n| {
                vec![
                    format!("// store a{n}"),
                    format!("store_ret{n}_{generic_name}{string_all_generics}(stack_ptr, a{n});"),
                ]
            })
            .chain(vec!["return;".into()])
            .collect::<Vec<String>>()
            .join("\n    ")
    };

    format!(
        "
@inline
function store_rets_{generic_name}{string_all_generics}({total_signature}): void {{
    {all_stores}
}}"
    )
}

fn generate_store_rets_specialized(signature: &WaspSignature) -> String {
    // eg: [`i64`, `i32`]
    let signature_rets = &signature.return_types;
    // eg: [`a0`, `a1`]
    let rets_arguments = signature_rets
        .iter()
        .enumerate()
        .map(|(index, _type)| format!("a{index}"));
    // eg: [`a0: R0`, `a1: R1`]
    let rets_signature = signature_rets
        .iter()
        .enumerate()
        .map(|(index, ty)| format!("a{index}: {ty}"));
    // eg: `stack_ptr: usize, a0: R0, a1: R1`
    let total_signature = (vec![String::from("stack_ptr: usize")])
        .into_iter()
        .chain(rets_signature)
        .collect::<Vec<String>>()
        .join(", ");
    // eg: `stack_ptr, a0, a1`
    let total_arguments = (vec![String::from("stack_ptr")])
        .into_iter()
        .chain(rets_arguments)
        .collect::<Vec<String>>()
        .join(", ");

    let comma_separated_types = signature.assemblyscript_comma_separated_types();
    let mangled_name = signature.generate_store_rets_name();
    let mangled_by_count_name = signature.mangled_assemblyscript_name_by_count();

    format!(
        "
export function {mangled_name}({total_signature}): void {{
    return store_rets_{mangled_by_count_name}{comma_separated_types}({total_arguments});
}};"
    )
}

const LIB_BOILERPLATE: &str = include_str!("lib_boilerplate.ts");

pub fn generate_lib(signatures: &[Signature]) -> String {
    let mut lib = String::from(LIB_BOILERPLATE);
    lib.push_str(&generate_lib_for(signatures));
    format!("{lib}\n")
}

fn generate_lib_for(signatures: &[Signature]) -> String {
    let mut processed_signature_counts: HashSet<(usize, usize)> = HashSet::new();
    let mut processed_signatures: HashSet<WaspSignature> = HashSet::new();
    let mut program = String::new();
    for signature in signatures {
        let signature = WaspSignature(signature);
        let signature_ret_count = signature.return_types.len();
        let signature_arg_count = signature.argument_types.len();
        let signature_length = (signature.return_types.len(), signature.argument_types.len());
        if !processed_signature_counts.contains(&signature_length) {
            processed_signature_counts.insert(signature_length);
            for generator in [
                generate_allocate_generic,
                generate_load_generic,
                generate_store_generic,
                generate_free_values_buffer_generic,
                generate_store_rets_generic,
                generate_allocate_types_buffer_generic,
                generate_free_types_buffer_generic,
            ] {
                program.push_str(generator(signature_ret_count, signature_arg_count).as_str());
            }
        }
        if !processed_signatures.contains(&signature) {
            for generator in [
                generate_allocate_specialized,
                generate_load_specialized,
                generate_store_specialized,
                generate_free_values_buffer_specialized,
                generate_store_rets_specialized,
                generate_allocate_types_buffer_specialized,
                generate_free_types_buffer_specialized,
            ] {
                program.push_str(&generator(&signature));
            }
            processed_signatures.insert(signature);
        }
    }
    program
}

#[cfg(test)]
mod tests {
    use super::*;
    use wastrumentation::wasm_constructs::{RefType, WasmType};

    // Some sample signatures for testing purposes
    fn get_ret_f64_f32_arg_i32_i64() -> Signature {
        Signature {
            return_types: vec![WasmType::F64, WasmType::F32],
            argument_types: vec![WasmType::I32, WasmType::I64],
        }
    }

    fn get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64() -> Signature {
        Signature {
            return_types: vec![WasmType::F64, WasmType::F32, WasmType::I32, WasmType::I64],
            argument_types: vec![WasmType::I64, WasmType::I32, WasmType::F32, WasmType::F64],
        }
    }

    #[test]
    fn test_signature_uniqueness() {
        let mut hash_set: HashSet<Signature> = HashSet::default();
        hash_set.insert(get_ret_f64_f32_arg_i32_i64());
        hash_set.insert(get_ret_f64_f32_arg_i32_i64());
        hash_set.insert(get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64());
        hash_set.insert(get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64());
        hash_set.insert(Signature {
            return_types: vec![WasmType::Ref(RefType::ExternRef)],
            argument_types: vec![WasmType::Ref(RefType::FuncRef)],
        });
        hash_set.insert(Signature {
            return_types: vec![WasmType::Ref(RefType::ExternRef)],
            argument_types: vec![WasmType::Ref(RefType::FuncRef)],
        });
        assert_eq!(hash_set.len(), 3);
    }

    #[test]
    fn generating_allocate_generic_instructions() {
        assert_eq!(
            generate_allocate_generic(0, 0),
            "
@inline
function allocate_ret_0_arg_0(): usize {
    const stack_begin = stack_allocate_values(0); // inlined
    return stack_begin;
}"
        );

        assert_eq!(
            generate_allocate_generic(0, 1),
            "
@inline
function allocate_ret_0_arg_1<T0>(a0: T0): usize {
    const stack_begin = stack_allocate_values(1); // inlined
    // store a0
    wastrumentation_memory_store<T0>(stack_begin, a0, 0);
    return stack_begin;
}"
        );

        assert_eq!(
        generate_allocate_generic(5, 5),
"
@inline
function allocate_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(a0: T0, a1: T1, a2: T2, a3: T3, a4: T4): usize {
    const stack_begin = stack_allocate_values(10); // inlined
    // store a0
    wastrumentation_memory_store<T0>(stack_begin, a0, 5);
    // store a1
    wastrumentation_memory_store<T1>(stack_begin, a1, 6);
    // store a2
    wastrumentation_memory_store<T2>(stack_begin, a2, 7);
    // store a3
    wastrumentation_memory_store<T3>(stack_begin, a3, 8);
    // store a4
    wastrumentation_memory_store<T4>(stack_begin, a4, 9);
    return stack_begin;
}"
    );
    }

    #[test]
    fn generating_allocate_specialized_instructions() {
        assert_eq!(
            generate_allocate_specialized(&WaspSignature(&get_ret_f64_f32_arg_i32_i64())),
            "
export function allocate_ret_f64_f32_arg_i32_i64(a0: i32, a1: i64): usize {
    return allocate_ret_2_arg_2<f64, f32, i32, i64>(a0, a1);
};
"
        );

        assert_eq!(generate_allocate_specialized(&WaspSignature(&get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64())), "
export function allocate_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(a0: i64, a1: i32, a2: f32, a3: f64): usize {
    return allocate_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(a0, a1, a2, a3);
};
");
    }

    #[test]
    fn generating_load_generic_instructions() {
        assert_eq!(
            generate_load_generic(0, 1),
            "
@inline
function load_arg0_ret_0_arg_1<T0>(stack_ptr: usize): T0 {
    return wastrumentation_memory_load<T0>(stack_ptr, 0);
}",
        );
        assert_eq!(
            generate_load_generic(5, 5),
            "
@inline
function load_arg0_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize): T0 {
    return wastrumentation_memory_load<T0>(stack_ptr, 5);
}

@inline
function load_arg1_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize): T1 {
    return wastrumentation_memory_load<T1>(stack_ptr, 6);
}

@inline
function load_arg2_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize): T2 {
    return wastrumentation_memory_load<T2>(stack_ptr, 7);
}

@inline
function load_arg3_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize): T3 {
    return wastrumentation_memory_load<T3>(stack_ptr, 8);
}

@inline
function load_arg4_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize): T4 {
    return wastrumentation_memory_load<T4>(stack_ptr, 9);
}

@inline
function load_ret0_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize): R0 {
    return wastrumentation_memory_load<R0>(stack_ptr, 0);
}

@inline
function load_ret1_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize): R1 {
    return wastrumentation_memory_load<R1>(stack_ptr, 1);
}

@inline
function load_ret2_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize): R2 {
    return wastrumentation_memory_load<R2>(stack_ptr, 2);
}

@inline
function load_ret3_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize): R3 {
    return wastrumentation_memory_load<R3>(stack_ptr, 3);
}

@inline
function load_ret4_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize): R4 {
    return wastrumentation_memory_load<R4>(stack_ptr, 4);
}"
        );
    }

    #[test]
    fn generating_load_specialized_instructions() {
        assert_eq!(
            generate_load_specialized(&WaspSignature(&get_ret_f64_f32_arg_i32_i64())),
            "
export function load_arg0_ret_f64_f32_arg_i32_i64(stack_ptr: usize): i32 {
    return load_arg0_ret_2_arg_2<f64, f32, i32, i64>(stack_ptr);
};

export function load_arg1_ret_f64_f32_arg_i32_i64(stack_ptr: usize): i64 {
    return load_arg1_ret_2_arg_2<f64, f32, i32, i64>(stack_ptr);
};

export function load_ret0_ret_f64_f32_arg_i32_i64(stack_ptr: usize): f64 {
    return load_ret0_ret_2_arg_2<f64, f32, i32, i64>(stack_ptr);
};

export function load_ret1_ret_f64_f32_arg_i32_i64(stack_ptr: usize): f32 {
    return load_ret1_ret_2_arg_2<f64, f32, i32, i64>(stack_ptr);
};"
        );

        assert_eq!(
            generate_load_specialized(&WaspSignature(
                &get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64()
            )),
            "
export function load_arg0_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize): i64 {
    return load_arg0_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
};

export function load_arg1_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize): i32 {
    return load_arg1_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
};

export function load_arg2_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize): f32 {
    return load_arg2_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
};

export function load_arg3_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize): f64 {
    return load_arg3_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
};

export function load_ret0_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize): f64 {
    return load_ret0_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
};

export function load_ret1_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize): f32 {
    return load_ret1_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
};

export function load_ret2_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize): i32 {
    return load_ret2_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
};

export function load_ret3_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize): i64 {
    return load_ret3_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
};"
        );
    }

    #[test]
    fn generating_store_generic_instructions() {
        assert_eq!(
            generate_store_generic(0, 1),
            "
@inline
function store_arg0_ret_0_arg_1<T0>(stack_ptr: usize, a0: T0): void {
    return wastrumentation_memory_store<T0>(stack_ptr, a0, 0);
}"
        );
        assert_eq!(generate_store_generic(5, 5), "
@inline
function store_arg0_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, a0: T0): void {
    return wastrumentation_memory_store<T0>(stack_ptr, a0, 5);
}

@inline
function store_arg1_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, a1: T1): void {
    return wastrumentation_memory_store<T1>(stack_ptr, a1, 6);
}

@inline
function store_arg2_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, a2: T2): void {
    return wastrumentation_memory_store<T2>(stack_ptr, a2, 7);
}

@inline
function store_arg3_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, a3: T3): void {
    return wastrumentation_memory_store<T3>(stack_ptr, a3, 8);
}

@inline
function store_arg4_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, a4: T4): void {
    return wastrumentation_memory_store<T4>(stack_ptr, a4, 9);
}

@inline
function store_ret0_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, r0: R0): void {
    return wastrumentation_memory_store<R0>(stack_ptr, r0, 0);
}

@inline
function store_ret1_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, r1: R1): void {
    return wastrumentation_memory_store<R1>(stack_ptr, r1, 1);
}

@inline
function store_ret2_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, r2: R2): void {
    return wastrumentation_memory_store<R2>(stack_ptr, r2, 2);
}

@inline
function store_ret3_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, r3: R3): void {
    return wastrumentation_memory_store<R3>(stack_ptr, r3, 3);
}

@inline
function store_ret4_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, r4: R4): void {
    return wastrumentation_memory_store<R4>(stack_ptr, r4, 4);
}");
    }

    #[test]
    fn generating_store_specialized_instructions() {
        assert_eq!(
            generate_store_specialized(&WaspSignature(&get_ret_f64_f32_arg_i32_i64())),
            "
export function store_arg0_ret_f64_f32_arg_i32_i64(stack_ptr: usize, a0: i32): void {
    return store_arg0_ret_2_arg_2<f64, f32, i32, i64>(stack_ptr, a0);
};

export function store_arg1_ret_f64_f32_arg_i32_i64(stack_ptr: usize, a1: i64): void {
    return store_arg1_ret_2_arg_2<f64, f32, i32, i64>(stack_ptr, a1);
};

export function store_ret0_ret_f64_f32_arg_i32_i64(stack_ptr: usize, a0: f64): void {
    return store_ret0_ret_2_arg_2<f64, f32, i32, i64>(stack_ptr, a0);
};

export function store_ret1_ret_f64_f32_arg_i32_i64(stack_ptr: usize, a1: f32): void {
    return store_ret1_ret_2_arg_2<f64, f32, i32, i64>(stack_ptr, a1);
};",
        );

        assert_eq!(generate_store_specialized(&WaspSignature(&get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64())), "
export function store_arg0_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a0: i64): void {
    return store_arg0_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a0);
};

export function store_arg1_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a1: i32): void {
    return store_arg1_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a1);
};

export function store_arg2_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a2: f32): void {
    return store_arg2_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a2);
};

export function store_arg3_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a3: f64): void {
    return store_arg3_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a3);
};

export function store_ret0_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a0: f64): void {
    return store_ret0_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a0);
};

export function store_ret1_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a1: f32): void {
    return store_ret1_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a1);
};

export function store_ret2_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a2: i32): void {
    return store_ret2_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a2);
};

export function store_ret3_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a3: i64): void {
    return store_ret3_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a3);
};");
    }

    #[test]
    fn generating_free_generic_instruction() {
        assert_eq!(
            generate_free_values_buffer_generic(0, 1),
            "
@inline
function free_values_ret_0_arg_1<T0>(ptr: usize): void {
    stack_deallocate_values(ptr, 1); // inlined
    return;
}"
        );
        assert_eq!(
            generate_free_values_buffer_generic(5, 5),
            "
@inline
function free_values_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(ptr: usize): void {
    stack_deallocate_values(ptr, 10); // inlined
    return;
}"
        );
    }

    #[test]
    fn generating_free_specialized_instruction() {
        assert_eq!(
            generate_free_values_buffer_specialized(&WaspSignature(&get_ret_f64_f32_arg_i32_i64())),
            "
export function free_values_ret_f64_f32_arg_i32_i64(ptr: usize): void {
    return free_values_ret_2_arg_2<f64, f32, i32, i64>(ptr);
};",
        );

        assert_eq!(
            generate_free_values_buffer_specialized(&WaspSignature(
                &get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64()
            )),
            "
export function free_values_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(ptr: usize): void {
    return free_values_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(ptr);
};",
        );
    }

    #[test]
    fn generating_free_types_generic_instruction() {
        assert_eq!(
            generate_free_types_buffer_generic(0, 1),
            "
@inline
function free_types_ret_0_arg_1(ptr: usize): void {
    stack_deallocate_types(ptr, 1); // inlined
    return;
}"
        );
        assert_eq!(
            generate_free_types_buffer_generic(5, 5),
            "
@inline
function free_types_ret_5_arg_5(ptr: usize): void {
    stack_deallocate_types(ptr, 10); // inlined
    return;
}"
        );
    }

    #[test]
    fn generating_free_types_specialized_instruction() {
        assert_eq!(
            generate_free_types_buffer_specialized(&WaspSignature(&get_ret_f64_f32_arg_i32_i64())),
            "
export function free_types_ret_f64_f32_arg_i32_i64(ptr: usize): void {
    return free_types_ret_2_arg_2(ptr);
};"
        );
        assert_eq!(
            generate_free_types_buffer_specialized(&WaspSignature(
                &get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64()
            )),
            "
export function free_types_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(ptr: usize): void {
    return free_types_ret_4_arg_4(ptr);
};"
        );
    }

    #[test]
    fn generating_store_rets_generic_instruction() {
        assert_eq!(
            generate_store_rets_generic(0, 1),
            "
@inline
function store_rets_ret_0_arg_1<T0>(stack_ptr: usize): void {
    return;
}",
        );
        assert_eq!(generate_store_rets_generic(5, 5), "
@inline
function store_rets_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, a0: R0, a1: R1, a2: R2, a3: R3, a4: R4): void {
    // store a0
    store_ret0_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr, a0);
    // store a1
    store_ret1_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr, a1);
    // store a2
    store_ret2_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr, a2);
    // store a3
    store_ret3_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr, a3);
    // store a4
    store_ret4_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr, a4);
    return;
}");
    }

    #[test]
    fn generating_store_rets_specialized_instruction() {
        assert_eq!(
            generate_store_rets_specialized(&WaspSignature(&get_ret_f64_f32_arg_i32_i64())),
            "
export function store_rets_ret_f64_f32_arg_i32_i64(stack_ptr: usize, a0: f64, a1: f32): void {
    return store_rets_ret_2_arg_2<f64, f32, i32, i64>(stack_ptr, a0, a1);
};"
        );

        assert_eq!(generate_store_rets_specialized(&WaspSignature(&get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64())), "
export function store_rets_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a0: f64, a1: f32, a2: i32, a3: i64): void {
    return store_rets_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a0, a1, a2, a3);
};");
    }

    #[test]
    fn generating_allocate_types_generic_specialized() {
        assert_eq!(
            generate_allocate_types_buffer_generic(0, 1),
            "
@inline
function allocate_signature_types_buffer_ret_0_arg_1(): usize {
    const stack_begin = stack_allocate_types(1); // inlined
    return stack_begin;
}"
        );

        assert_eq!(
            generate_allocate_types_buffer_generic(5, 5),
            "
@inline
function allocate_signature_types_buffer_ret_5_arg_5(): usize {
    const stack_begin = stack_allocate_types(10); // inlined
    return stack_begin;
}"
        )
    }

    #[test]
    fn generating_allocate_types_buffer_specialized_instruction() {
        let signature_empty = Signature {
            return_types: vec![],
            argument_types: vec![],
        };
        assert_eq!(
            generate_allocate_types_buffer_specialized(&WaspSignature(&signature_empty)),
            "
export function allocate_types_ret_arg(): usize {
    const types_buffer = allocate_signature_types_buffer_ret_0_arg_0();
    return types_buffer;
}"
        );
        let signature_0 = Signature {
            return_types: vec![WasmType::F64, WasmType::F32],
            argument_types: vec![WasmType::I32, WasmType::I64],
        };
        assert_eq!(
            generate_allocate_types_buffer_specialized(&WaspSignature(&signature_0)),
            "
export function allocate_types_ret_f64_f32_arg_i32_i64(): usize {
    const types_buffer = allocate_signature_types_buffer_ret_2_arg_2();
    wastrumentation_stack_store_type(types_buffer, 0, 3);
    wastrumentation_stack_store_type(types_buffer, 1, 1);
    wastrumentation_stack_store_type(types_buffer, 2, 0);
    wastrumentation_stack_store_type(types_buffer, 3, 2);
    return types_buffer;
}"
        );
        let signature_1 = Signature {
            return_types: vec![WasmType::F64, WasmType::F32, WasmType::I32, WasmType::I64],
            argument_types: vec![WasmType::I64, WasmType::I32, WasmType::F32, WasmType::F64],
        };
        assert_eq!(
            generate_allocate_types_buffer_specialized(&WaspSignature(&signature_1)),
            "
export function allocate_types_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(): usize {
    const types_buffer = allocate_signature_types_buffer_ret_4_arg_4();
    wastrumentation_stack_store_type(types_buffer, 0, 3);
    wastrumentation_stack_store_type(types_buffer, 1, 1);
    wastrumentation_stack_store_type(types_buffer, 2, 0);
    wastrumentation_stack_store_type(types_buffer, 3, 2);
    wastrumentation_stack_store_type(types_buffer, 4, 2);
    wastrumentation_stack_store_type(types_buffer, 5, 0);
    wastrumentation_stack_store_type(types_buffer, 6, 1);
    wastrumentation_stack_store_type(types_buffer, 7, 3);
    return types_buffer;
}"
        );

        let signature_2 = Signature {
            return_types: vec![
                WasmType::Ref(RefType::FuncRef),
                WasmType::Ref(RefType::FuncRef),
            ],
            argument_types: vec![
                WasmType::Ref(RefType::ExternRef),
                WasmType::Ref(RefType::ExternRef),
            ],
        };
        assert_eq!(
            generate_allocate_types_buffer_specialized(&WaspSignature(&signature_2)),
            "
export function allocate_types_ret_ref_func_ref_func_arg_ref_extern_ref_extern(): usize {
    const types_buffer = allocate_signature_types_buffer_ret_2_arg_2();
    wastrumentation_stack_store_type(types_buffer, 0, 4);
    wastrumentation_stack_store_type(types_buffer, 1, 4);
    wastrumentation_stack_store_type(types_buffer, 2, 5);
    wastrumentation_stack_store_type(types_buffer, 3, 5);
    return types_buffer;
}"
        );
    }
}
