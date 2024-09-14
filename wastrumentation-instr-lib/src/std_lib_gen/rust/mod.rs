use std::{collections::HashSet, ops::Deref, vec};

use crate::std_lib_compile::rust::{ManifestSource, RustSourceCode};
use wastrumentation::wasm_constructs::{Signature, SignatureSide, WasmType};

// TODO: since this holds:

// fn foo() {
//     let tuple_size = size_of::<(f64, i32, i32)>();
//     let sum_size = size_of::<f64>() + size_of::<i32>() + size_of::<i32>();
//     assert_eq!(tuple_size, sum_size)
// }

// => WARNING: Due to alignment, some structures are padded. As such, size_of::<struct _X(i32,u8)>() != size_of::<(i32,u8)>() ...

// I could move all "+" expressions to a tuple variant ... Should not change anything, but 'enforce' constant folding

#[derive(Hash, PartialEq, Eq)]
pub struct RustSignature<'a>(&'a Signature);

impl Deref for RustSignature<'_> {
    type Target = Signature;

    fn deref(&self) -> &Self::Target {
        let Self(target) = self;
        target
    }
}

impl RustSignature<'_> {
    fn mangled_rust_name_by_count(&self) -> String {
        let signature_rets_count = self.return_types.len();
        let signature_args_count = self.argument_types.len();
        format!("ret_{signature_rets_count}_arg_{signature_args_count}")
    }

    fn generic_rust_name(signature_rets_count: usize, signature_args_count: usize) -> String {
        format!("ret_{signature_rets_count}_arg_{signature_args_count}")
    }

    fn generic_rust_where_clause(rets_count: usize, args_count: usize) -> String {
        if rets_count + args_count == 0 {
            String::new()
        } else {
            format!(
                "where {}",
                Self::rust_generics(rets_count, args_count)
                    .into_iter()
                    .map(|t| format!("{t}: Copy"))
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        }
    }

    fn rust_comma_separated_types(&self) -> String {
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
            format!("::<{comma_separated_types}>")
        }
    }

    fn rust_comma_separated_generics(rets_count: usize, args_count: usize) -> String {
        if rets_count + args_count == 0 {
            String::new()
        } else {
            format!(
                "<{}>",
                Self::rust_generics(rets_count, args_count).join(", ")
            )
        }
    }

    fn rust_generics(rets_count: usize, args_count: usize) -> Vec<String> {
        let arg_typs = (0..args_count).map(|n| format!("T{n}"));
        let ret_typs = (0..rets_count).map(|n| format!("R{n}"));
        ret_typs.chain(arg_typs).collect::<Vec<String>>()
    }

    fn rust_generic_typed_arguments(args_count: usize) -> Vec<String> {
        (0..args_count)
            .clone()
            .map(|n| format!("a{n}: T{n}"))
            .collect::<Vec<String>>()
    }

    fn rust_generic_comma_separated_typed_arguments(args_count: usize) -> String {
        Self::rust_generic_typed_arguments(args_count).join(", ")
    }

    fn rust_compute_type_allocation(rets_count: usize, args_count: usize) -> String {
        if rets_count + args_count == 0 {
            "0".to_string()
        } else {
            Self::rust_generics(rets_count, args_count)
                .iter()
                .map(|ty| format!("size_of::<{ty}>()"))
                .collect::<Vec<String>>()
                .join(" + ")
        }
    }
}

fn generics_offset(position: usize, rets_count: usize, args_offset: usize) -> String {
    if position == 0 {
        return "0".into();
    };
    let ret_offsets: Vec<String> = (0..rets_count)
        .map(|n| format!("size_of::<R{n}>()"))
        .collect();
    let arg_offsets: Vec<String> = (0..args_offset)
        .map(|n| format!("size_of::<T{n}>()"))
        .collect();

    ret_offsets
        .into_iter()
        .chain(arg_offsets)
        .take(position)
        .collect::<Vec<String>>()
        .join(" + ")
}

fn arg_offset(arg_pos: usize, rets_count: usize, args_count: usize) -> String {
    generics_offset(rets_count + arg_pos, rets_count, args_count)
}

fn ret_offset(ret_pos: usize, rets_count: usize, args_count: usize) -> String {
    generics_offset(ret_pos, rets_count, args_count)
}

fn generate_allocate_generic(rets_count: usize, args_count: usize) -> String {
    let string_all_generics = RustSignature::rust_comma_separated_generics(rets_count, args_count);
    let signature = RustSignature::rust_generic_comma_separated_typed_arguments(args_count);
    let generic_name = RustSignature::generic_rust_name(rets_count, args_count);
    let where_clause = RustSignature::generic_rust_where_clause(rets_count, args_count);

    // eg: `size_of::<R0>() +  size_of::<R1>() +  size_of::<T0>() +  size_of::<T1>()`
    let total_allocation = RustSignature::rust_compute_type_allocation(rets_count, args_count);
    let all_stores_followed_by_return = (0..args_count)
        .map(|n| {
            let offset = arg_offset(n, rets_count, args_count);
            format!(
                "// store a{n}
    let a{n}_offset = {offset}; // constant folded
    wastrumentation_memory_store::<T{n}>(shadow_frame_ptr, a{n}, a{n}_offset);
"
            )
        })
        .chain(vec!["shadow_frame_ptr as usize".into()])
        .collect::<Vec<String>>()
        .join("\n    ");

    format!(
        "
#[inline(always)]
fn allocate_{generic_name}{string_all_generics}({signature}) -> usize {where_clause} {{
    let align = align_of::<u8>();
    let size_to_allocate: usize = {total_allocation};
    let layout = unsafe {{ Layout::from_size_align_unchecked(size_to_allocate, align) }};
    let shadow_frame_ptr = unsafe {{ GlobalAlloc::alloc(&ALLOC, layout) }} as usize;
    {all_stores_followed_by_return}
}}"
    )
}

fn generate_allocate_specialized(signature: &RustSignature) -> String {
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

    let comma_separated_types = signature.rust_comma_separated_types();
    let mangled_name = signature.generate_allocate_values_buffer_name();
    let mangled_by_count_name = signature.mangled_rust_name_by_count();

    format!(
        "
#[no_mangle]
pub fn {mangled_name}({signature_args_typs_ident}) -> usize {{
    return allocate_{mangled_by_count_name}{comma_separated_types}({args});
}}
"
    )
}

fn generate_allocate_types_buffer_generic(rets_count: usize, args_count: usize) -> String {
    let total_allocation = format!("size_of::<i32>() * {}", rets_count + args_count);
    let generic_name = RustSignature::generic_rust_name(rets_count, args_count);
    format!(
        "
#[inline(always)]
fn allocate_signature_types_buffer_{generic_name}() -> usize {{
    let to_allocate = {total_allocation}; // constant folded
    let stack_begin = stack_allocate(to_allocate); // inlined
    return stack_begin;
}}"
    )
}

fn generate_allocate_types_buffer_specialized(signature: &RustSignature) -> String {
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
            format!("wastrumentation_memory_store::<i32>(types_buffer, {enum_value}, size_of::<i32>()*{index});")
        })
        .chain(vec!["return types_buffer;".into()])
        .collect::<Vec<String>>()
        .join("\n    ");

    let mangled_name = signature.generate_allocate_types_buffer_name();
    let mangled_by_count_name = signature.mangled_rust_name_by_count();

    format!(
        "
#[no_mangle]
pub fn {mangled_name}() -> usize {{
    let types_buffer = allocate_signature_types_buffer_{mangled_by_count_name}();
    {all_stores_followed_by_return}
}}"
    )
}

fn generate_load_generic(rets_count: usize, args_count: usize) -> String {
    let string_all_generics = RustSignature::rust_comma_separated_generics(rets_count, args_count);
    let generic_name = RustSignature::generic_rust_name(rets_count, args_count);
    let where_clause = RustSignature::generic_rust_where_clause(rets_count, args_count);

    let all_arg_loads = (0..args_count).map(|n| {
        let an_offset = arg_offset(n, rets_count, args_count);
        format!(
            "
#[inline(always)]
fn load_arg{n}_{generic_name}{string_all_generics}(stack_ptr: usize) -> T{n} {where_clause} {{
    let a{n}_offset = {an_offset}; // constant folded
    return wastrumentation_memory_load::<T{n}>(stack_ptr, a{n}_offset); // inlined
}}"
        )
    });
    let all_ret_loads = (0..rets_count).map(|n| {
        let ar_offset = ret_offset(n, rets_count, args_count);
        format!(
            "
#[inline(always)]
fn load_ret{n}_{generic_name}{string_all_generics}(stack_ptr: usize) -> R{n} {where_clause} {{
    let r{n}_offset = {ar_offset}; // constant folded
    return wastrumentation_memory_load::<R{n}>(stack_ptr, r{n}_offset); // inlined
}}"
        )
    });
    all_arg_loads
        .chain(all_ret_loads)
        .collect::<Vec<String>>()
        .join("\n")
}

fn generate_load_specialized(signature: &RustSignature) -> String {
    // eg: [`i64`, `i32`]
    let signature_rets = &signature.return_types;
    // eg: [`f64`, `f32`]
    let signature_args = &signature.argument_types;

    let comma_separated_types = signature.rust_comma_separated_types();
    let mangled_by_count_name = signature.mangled_rust_name_by_count();

    let all_arg_loads = signature_args
        .iter()
        .enumerate()
        .map(|(index, arg_i_ret_type)| {
            let mangled_name = signature.generate_load_name(SignatureSide::Argument, index);
            format!(
                "
#[no_mangle]
pub fn {mangled_name}(stack_ptr: usize) -> {arg_i_ret_type} {{
    return load_arg{index}_{mangled_by_count_name}{comma_separated_types}(stack_ptr);
}}"
            )
        });
    let all_ret_loads = signature_rets
        .iter()
        .enumerate()
        .map(|(index, ret_i_ret_type)| {
            let mangled_name = signature.generate_load_name(SignatureSide::Return, index);
            format!(
                "
#[no_mangle]
pub fn {mangled_name}(stack_ptr: usize) -> {ret_i_ret_type} {{
    return load_ret{index}_{mangled_by_count_name}{comma_separated_types}(stack_ptr);
}}"
            )
        });

    all_arg_loads
        .chain(all_ret_loads)
        .collect::<Vec<String>>()
        .join("\n")
}

fn generate_store_generic(rets_count: usize, args_count: usize) -> String {
    let string_all_generics = RustSignature::rust_comma_separated_generics(rets_count, args_count);
    let generic_name = RustSignature::generic_rust_name(rets_count, args_count);
    let where_clause = RustSignature::generic_rust_where_clause(rets_count, args_count);

    let all_arg_stores = (0..args_count).map(|n| {
        let an_offset = arg_offset(n, rets_count, args_count);
        format!(
            "
#[inline(always)]
fn store_arg{n}_{generic_name}{string_all_generics}(stack_ptr: usize, a{n}: T{n}) {where_clause} {{
    let a{n}_offset: usize = {an_offset}; // constant folded
    return wastrumentation_memory_store::<T{n}>(stack_ptr, a{n}, a{n}_offset); // inlined
}}"
        )
    });
    let all_ret_stores = (0..rets_count).map(|n| {
        let ar_offset = ret_offset(n, rets_count, args_count);
        format!(
            "
#[inline(always)]
fn store_ret{n}_{generic_name}{string_all_generics}(stack_ptr: usize, r{n}: R{n}) {where_clause} {{
    let r{n}_offset: usize = {ar_offset}; // constant folded
    return wastrumentation_memory_store::<R{n}>(stack_ptr, r{n}, r{n}_offset); // inlined
}}"
        )
    });
    all_arg_stores
        .chain(all_ret_stores)
        .collect::<Vec<String>>()
        .join("\n")
}

fn generate_store_specialized(signature: &RustSignature) -> String {
    // eg: [`i64`, `i32`]
    let signature_rets = &signature.return_types;
    // eg: [`f64`, `f32`]
    let signature_args = &signature.argument_types;

    let comma_separated_types = signature.rust_comma_separated_types();
    let mangled_by_count_name = signature.mangled_rust_name_by_count();

    let all_arg_stores = signature_args
        .iter()
        .enumerate()
        .map(|(index, arg_i_ret_type)| {
            let mangled_name = signature.generate_store_name(SignatureSide::Argument, index);
            format!(
                "
#[no_mangle]
pub fn {mangled_name}(stack_ptr: usize, a{index}: {arg_i_ret_type}) {{
    return store_arg{index}_{mangled_by_count_name}{comma_separated_types}(stack_ptr, a{index});
}}"
            )
        });
    let all_ret_stores = signature_rets
        .iter()
        .enumerate()
        .map(|(index, ret_i_ret_type)| {
            let mangled_name = signature.generate_store_name(SignatureSide::Return, index);
            format!(
                "
#[no_mangle]
pub fn {mangled_name}(stack_ptr: usize, a{index}: {ret_i_ret_type}) {{
    return store_ret{index}_{mangled_by_count_name}{comma_separated_types}(stack_ptr, a{index});
}}"
            )
        });
    all_arg_stores
        .chain(all_ret_stores)
        .collect::<Vec<String>>()
        .join("\n")
}

fn generate_free_values_buffer_generic(rets_count: usize, args_count: usize) -> String {
    let string_all_generics = RustSignature::rust_comma_separated_generics(rets_count, args_count);
    let generic_name = RustSignature::generic_rust_name(rets_count, args_count);
    let where_clause = RustSignature::generic_rust_where_clause(rets_count, args_count);

    // eg: `size_of::<R0>() +  size_of::<R1>() +  size_of::<T0>() +  size_of::<T1>()`
    let total_allocation = RustSignature::rust_compute_type_allocation(rets_count, args_count);

    format!(
        "
#[inline(always)]
fn free_values_{generic_name}{string_all_generics}(ptr: usize) {where_clause} {{
    let to_deallocate = {total_allocation}; // constant folded
    stack_deallocate(ptr, to_deallocate); // inlined
    return;
}}"
    )
}

fn generate_free_values_buffer_specialized(signature: &RustSignature) -> String {
    format!(
        "
#[no_mangle]
pub fn {}(ptr: usize) {{
    return free_values_{}{}(ptr);
}}",
        signature.generate_free_values_buffer_name(),
        signature.mangled_rust_name_by_count(),
        signature.rust_comma_separated_types(),
    )
}

fn generate_free_types_buffer_generic(rets_count: usize, args_count: usize) -> String {
    let generic_name = RustSignature::generic_rust_name(rets_count, args_count);
    let total_allocation = format!("size_of::<i32>() * {}", rets_count + args_count);

    format!(
        "
#[inline(always)]
fn free_types_{generic_name}(ptr: usize) {{
    let to_deallocate = {total_allocation}; // constant folded
    stack_deallocate(ptr, to_deallocate); // inlined
    return;
}}"
    )
}

fn generate_free_types_buffer_specialized(signature: &RustSignature) -> String {
    format!(
        "
#[no_mangle]
pub fn {}(ptr: usize) {{
    return free_types_{}(ptr);
}}",
        signature.generate_free_types_buffer_name(),
        signature.mangled_rust_name_by_count(),
    )
}

fn generate_store_rets_generic(rets_count: usize, args_count: usize) -> String {
    let string_all_generics = RustSignature::rust_comma_separated_generics(rets_count, args_count);
    let generic_name = RustSignature::generic_rust_name(rets_count, args_count);
    let where_clause = RustSignature::generic_rust_where_clause(rets_count, args_count);

    // eg: [`a0: R0`, `a1: R1`]
    let array_of_rets_signature = (0..rets_count).map(|n| format!("a{n}: R{n}"));
    let stack_ptr = String::from(if rets_count == 0 {
        "_stack_ptr: usize"
    } else {
        "stack_ptr: usize"
    });
    let total_signature = (vec![stack_ptr])
        .into_iter()
        .chain(array_of_rets_signature)
        .collect::<Vec<String>>()
        .join(", ");
    let all_stores = (0..rets_count)
        .flat_map(|n| {
            vec![
                format!("// store a{n}"),
                format!("store_ret{n}_{generic_name}::{string_all_generics}(stack_ptr, a{n});"),
            ]
        })
        .chain(vec!["return;".into()])
        .collect::<Vec<String>>()
        .join("\n    ");

    format!(
        "
#[inline(always)]
fn store_rets_{generic_name}{string_all_generics}({total_signature}) {where_clause} {{
    {all_stores}
}}"
    )
}

fn generate_store_rets_specialized(signature: &RustSignature) -> String {
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

    let comma_separated_types = signature.rust_comma_separated_types();
    let mangled_name = signature.generate_store_rets_name();
    let mangled_by_count_name = signature.mangled_rust_name_by_count();

    format!(
        "
#[no_mangle]
pub fn {mangled_name}({total_signature}) {{
    return store_rets_{mangled_by_count_name}{comma_separated_types}({total_arguments});
}}"
    )
}

const LIB_BOILERPLATE: &str = r#"
#![cfg_attr(not(test), no_std)]

use core::mem::{size_of, align_of};
use core::alloc::{GlobalAlloc, Layout};

extern crate wee_alloc;
#[global_allocator]
pub static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Optionally use primitives from core::arch::wasm
// https://doc.rust-lang.org/stable/core/arch/wasm/index.html

#[cfg(not(test))]
#[panic_handler]
fn panic(_panic: &core::panic::PanicInfo<'_>) -> ! {
    #[cfg(not(target_arch = "wasm32"))]
    core::unreachable!();
    #[cfg(target_arch = "wasm32")]
    core::arch::wasm32::unreachable()
}

#[inline(always)]
fn wastrumentation_memory_load<T: Copy>(stack_ptr: usize, offset: usize) -> T {
    let ptr: *const u8 = stack_ptr as *const u8;
    unsafe { *(ptr.offset(offset as isize) as *const T) }
}

#[inline(always)]
fn wastrumentation_memory_store<T>(stack_ptr: usize, value: T, offset: usize) {
    let ptr: *mut u8 = stack_ptr as *mut u8;
    unsafe {
        *(ptr.offset(offset as isize) as *mut T) = value;
    }
}

#[inline(always)]
fn stack_allocate(bytes_to_allocate: usize) -> usize {
    let align = align_of::<u8>();
    let layout: Layout = unsafe { Layout::from_size_align_unchecked(bytes_to_allocate, align) };
    let shadow_frame_ptr = unsafe { ALLOC.alloc(layout) } as usize;
    shadow_frame_ptr
}

#[inline(always)]
fn stack_deallocate(ptr: usize, bytes_to_allocate: usize) {
    let align = align_of::<u8>();
    let layout: Layout = unsafe { Layout::from_size_align_unchecked(bytes_to_allocate, align) };
    unsafe { ALLOC.dealloc(ptr as *mut u8, layout) };
}

#[no_mangle]
#[inline(always)]
pub extern "C" fn wastrumentation_stack_load_i32(ptr: usize, offset: usize) -> i32 {
    wastrumentation_memory_load::<i32>(ptr, offset)
}

#[no_mangle]
#[inline(always)]
pub extern "C" fn wastrumentation_stack_load_f32(ptr: usize, offset: usize) -> f32 {
    wastrumentation_memory_load::<f32>(ptr, offset)
}

#[no_mangle]
#[inline(always)]
pub extern "C" fn wastrumentation_stack_load_i64(ptr: usize, offset: usize) -> i64 {
    wastrumentation_memory_load::<i64>(ptr, offset)
}

#[no_mangle]
#[inline(always)]
pub extern "C" fn wastrumentation_stack_load_f64(ptr: usize, offset: usize) -> f64 {
    wastrumentation_memory_load::<f64>(ptr, offset)
}

#[no_mangle]
#[inline(always)]
pub extern "C" fn wastrumentation_stack_store_i32(ptr: usize, value: i32, offset: usize) {
    wastrumentation_memory_store::<i32>(ptr, value, offset)
}

#[no_mangle]
#[inline(always)]
pub extern "C" fn wastrumentation_stack_store_f32(ptr: usize, value: f32, offset: usize) {
    wastrumentation_memory_store::<f32>(ptr, value, offset)
}

#[no_mangle]
#[inline(always)]
pub extern "C" fn wastrumentation_stack_store_i64(ptr: usize, value: i64, offset: usize) {
    wastrumentation_memory_store::<i64>(ptr, value, offset)
}

#[no_mangle]
#[inline(always)]
pub extern "C" fn wastrumentation_stack_store_f64(ptr: usize, value: f64, offset: usize) {
    wastrumentation_memory_store::<f64>(ptr, value, offset)
}
"#;

const DEFAULT_MANIFEST_SOURCE: &str = include_str!("default_manifest_source.toml");

pub fn generate_lib(signatures: &[Signature]) -> (ManifestSource, RustSourceCode) {
    let mut lib = String::from(LIB_BOILERPLATE);
    lib.push_str(&generate_lib_for(signatures));
    let lib = format!("{lib}\n");
    (
        ManifestSource(DEFAULT_MANIFEST_SOURCE.to_string()),
        RustSourceCode(lib),
    )
}

fn generate_lib_for(signatures: &[Signature]) -> String {
    let mut processed_signature_counts: HashSet<(usize, usize)> = HashSet::new();
    let mut processed_signatures: HashSet<RustSignature> = HashSet::new();
    let mut program = String::new();
    for signature in signatures {
        let signature = RustSignature(signature);
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
    use wastrumentation::wasm_constructs::RefType;

    use super::*;
    use rust_to_wasm_compiler::{Profile, RustToWasmCompiler};

    #[test]
    fn test_compiles() {
        let mut hash_set: HashSet<Signature> = HashSet::default();
        hash_set.insert(Signature {
            return_types: vec![WasmType::I32],
            argument_types: vec![WasmType::F64],
        });
        hash_set.insert(Signature {
            return_types: vec![],
            argument_types: vec![],
        });
        hash_set.insert(Signature {
            return_types: vec![],
            argument_types: vec![WasmType::I32],
        });
        hash_set.insert(Signature {
            return_types: vec![WasmType::I32],
            argument_types: vec![],
        });
        let signatures: Vec<Signature> = hash_set.into_iter().collect();

        let (ManifestSource(manifest), RustSourceCode(rust_source)) = generate_lib(&signatures);
        let wasm_module = RustToWasmCompiler::new()
            .unwrap()
            .compile_source(&manifest, &rust_source, Profile::Release)
            .unwrap();

        // Compile result
        assert!(!wasm_module.is_empty());
    }

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
    fn compute_memory_offset_generically_works_correctly() {
        let pos_frst = 0;
        let pos_scnd = 1;
        let pos_thrd = 2;
        let pos_frth = 3;
        for ((position, ret_count, arg_offset), expected) in [
            ((pos_frst, 0, 1), "0"),
            ((pos_frst, 0, 2), "0"),
            ((pos_scnd, 0, 2), "size_of::<T0>()"),
            ((pos_frst, 1, 0), "0"),
            ((pos_frst, 1, 1), "0"),
            ((pos_scnd, 1, 1), "size_of::<R0>()"),
            ((pos_frst, 1, 2), "0"),
            ((pos_scnd, 1, 2), "size_of::<R0>()"),
            ((pos_thrd, 1, 2), "size_of::<R0>() + size_of::<T0>()"),
            ((pos_frst, 2, 0), "0"),
            ((pos_scnd, 2, 0), "size_of::<R0>()"),
            ((pos_frst, 2, 1), "0"),
            ((pos_scnd, 2, 1), "size_of::<R0>()"),
            ((pos_thrd, 2, 1), "size_of::<R0>() + size_of::<R1>()"),
            ((pos_frst, 2, 2), "0"),
            ((pos_scnd, 2, 2), "size_of::<R0>()"),
            ((pos_thrd, 2, 2), "size_of::<R0>() + size_of::<R1>()"),
            (
                (pos_frth, 2, 2),
                "size_of::<R0>() + size_of::<R1>() + size_of::<T0>()",
            ),
        ] {
            assert_eq!(generics_offset(position, ret_count, arg_offset), expected);
        }
    }

    #[test]
    fn generating_allocate_generic_instructions() {
        assert_eq!(
            generate_allocate_generic(0, 0),
            "
#[inline(always)]
fn allocate_ret_0_arg_0() -> usize  {
    let align = align_of::<u8>();
    let size_to_allocate: usize = 0;
    let layout = unsafe { Layout::from_size_align_unchecked(size_to_allocate, align) };
    let shadow_frame_ptr = unsafe { GlobalAlloc::alloc(&ALLOC, layout) } as usize;
    shadow_frame_ptr as usize
}"
        );

        assert_eq!(
            generate_allocate_generic(0, 1),
            "
#[inline(always)]
fn allocate_ret_0_arg_1<T0>(a0: T0) -> usize where T0: Copy {
    let align = align_of::<u8>();
    let size_to_allocate: usize = size_of::<T0>();
    let layout = unsafe { Layout::from_size_align_unchecked(size_to_allocate, align) };
    let shadow_frame_ptr = unsafe { GlobalAlloc::alloc(&ALLOC, layout) } as usize;
    // store a0
    let a0_offset = 0; // constant folded
    wastrumentation_memory_store::<T0>(shadow_frame_ptr, a0, a0_offset);

    shadow_frame_ptr as usize
}"
        );

        assert_eq!(
        generate_allocate_generic(5, 5),
    "
#[inline(always)]
fn allocate_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(a0: T0, a1: T1, a2: T2, a3: T3, a4: T4) -> usize where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let align = align_of::<u8>();
    let size_to_allocate: usize = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>() + size_of::<R4>() + size_of::<T0>() + size_of::<T1>() + size_of::<T2>() + size_of::<T3>() + size_of::<T4>();
    let layout = unsafe { Layout::from_size_align_unchecked(size_to_allocate, align) };
    let shadow_frame_ptr = unsafe { GlobalAlloc::alloc(&ALLOC, layout) } as usize;
    // store a0
    let a0_offset = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>() + size_of::<R4>(); // constant folded
    wastrumentation_memory_store::<T0>(shadow_frame_ptr, a0, a0_offset);

    // store a1
    let a1_offset = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>() + size_of::<R4>() + size_of::<T0>(); // constant folded
    wastrumentation_memory_store::<T1>(shadow_frame_ptr, a1, a1_offset);

    // store a2
    let a2_offset = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>() + size_of::<R4>() + size_of::<T0>() + size_of::<T1>(); // constant folded
    wastrumentation_memory_store::<T2>(shadow_frame_ptr, a2, a2_offset);

    // store a3
    let a3_offset = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>() + size_of::<R4>() + size_of::<T0>() + size_of::<T1>() + size_of::<T2>(); // constant folded
    wastrumentation_memory_store::<T3>(shadow_frame_ptr, a3, a3_offset);

    // store a4
    let a4_offset = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>() + size_of::<R4>() + size_of::<T0>() + size_of::<T1>() + size_of::<T2>() + size_of::<T3>(); // constant folded
    wastrumentation_memory_store::<T4>(shadow_frame_ptr, a4, a4_offset);

    shadow_frame_ptr as usize
}"
    );
    }

    #[test]
    fn generating_allocate_specialized_instructions() {
        assert_eq!(
            generate_allocate_specialized(&RustSignature(&get_ret_f64_f32_arg_i32_i64())),
            "
#[no_mangle]
pub fn allocate_ret_f64_f32_arg_i32_i64(a0: i32, a1: i64) -> usize {
    return allocate_ret_2_arg_2::<f64, f32, i32, i64>(a0, a1);
}
"
        );

        assert_eq!(generate_allocate_specialized(&RustSignature(&get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64())), "
#[no_mangle]
pub fn allocate_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(a0: i64, a1: i32, a2: f32, a3: f64) -> usize {
    return allocate_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(a0, a1, a2, a3);
}
");
    }

    #[test]
    fn generating_load_generic_instructions() {
        assert_eq!(
            generate_load_generic(0, 1),
            "
#[inline(always)]
fn load_arg0_ret_0_arg_1<T0>(stack_ptr: usize) -> T0 where T0: Copy {
    let a0_offset = 0; // constant folded
    return wastrumentation_memory_load::<T0>(stack_ptr, a0_offset); // inlined
}",
        );
        assert_eq!(generate_load_generic(5, 5), "
#[inline(always)]
fn load_arg0_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize) -> T0 where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let a0_offset = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>() + size_of::<R4>(); // constant folded
    return wastrumentation_memory_load::<T0>(stack_ptr, a0_offset); // inlined
}

#[inline(always)]
fn load_arg1_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize) -> T1 where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let a1_offset = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>() + size_of::<R4>() + size_of::<T0>(); // constant folded
    return wastrumentation_memory_load::<T1>(stack_ptr, a1_offset); // inlined
}

#[inline(always)]
fn load_arg2_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize) -> T2 where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let a2_offset = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>() + size_of::<R4>() + size_of::<T0>() + size_of::<T1>(); // constant folded
    return wastrumentation_memory_load::<T2>(stack_ptr, a2_offset); // inlined
}

#[inline(always)]
fn load_arg3_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize) -> T3 where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let a3_offset = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>() + size_of::<R4>() + size_of::<T0>() + size_of::<T1>() + size_of::<T2>(); // constant folded
    return wastrumentation_memory_load::<T3>(stack_ptr, a3_offset); // inlined
}

#[inline(always)]
fn load_arg4_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize) -> T4 where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let a4_offset = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>() + size_of::<R4>() + size_of::<T0>() + size_of::<T1>() + size_of::<T2>() + size_of::<T3>(); // constant folded
    return wastrumentation_memory_load::<T4>(stack_ptr, a4_offset); // inlined
}

#[inline(always)]
fn load_ret0_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize) -> R0 where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let r0_offset = 0; // constant folded
    return wastrumentation_memory_load::<R0>(stack_ptr, r0_offset); // inlined
}

#[inline(always)]
fn load_ret1_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize) -> R1 where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let r1_offset = size_of::<R0>(); // constant folded
    return wastrumentation_memory_load::<R1>(stack_ptr, r1_offset); // inlined
}

#[inline(always)]
fn load_ret2_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize) -> R2 where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let r2_offset = size_of::<R0>() + size_of::<R1>(); // constant folded
    return wastrumentation_memory_load::<R2>(stack_ptr, r2_offset); // inlined
}

#[inline(always)]
fn load_ret3_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize) -> R3 where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let r3_offset = size_of::<R0>() + size_of::<R1>() + size_of::<R2>(); // constant folded
    return wastrumentation_memory_load::<R3>(stack_ptr, r3_offset); // inlined
}

#[inline(always)]
fn load_ret4_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize) -> R4 where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let r4_offset = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>(); // constant folded
    return wastrumentation_memory_load::<R4>(stack_ptr, r4_offset); // inlined
}");
    }

    #[test]
    fn generating_load_specialized_instructions() {
        assert_eq!(
            generate_load_specialized(&RustSignature(&get_ret_f64_f32_arg_i32_i64())),
            "
#[no_mangle]
pub fn load_arg0_ret_f64_f32_arg_i32_i64(stack_ptr: usize) -> i32 {
    return load_arg0_ret_2_arg_2::<f64, f32, i32, i64>(stack_ptr);
}

#[no_mangle]
pub fn load_arg1_ret_f64_f32_arg_i32_i64(stack_ptr: usize) -> i64 {
    return load_arg1_ret_2_arg_2::<f64, f32, i32, i64>(stack_ptr);
}

#[no_mangle]
pub fn load_ret0_ret_f64_f32_arg_i32_i64(stack_ptr: usize) -> f64 {
    return load_ret0_ret_2_arg_2::<f64, f32, i32, i64>(stack_ptr);
}

#[no_mangle]
pub fn load_ret1_ret_f64_f32_arg_i32_i64(stack_ptr: usize) -> f32 {
    return load_ret1_ret_2_arg_2::<f64, f32, i32, i64>(stack_ptr);
}"
        );

        assert_eq!(
            generate_load_specialized(&RustSignature(
                &get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64()
            )),
            "
#[no_mangle]
pub fn load_arg0_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize) -> i64 {
    return load_arg0_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
}

#[no_mangle]
pub fn load_arg1_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize) -> i32 {
    return load_arg1_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
}

#[no_mangle]
pub fn load_arg2_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize) -> f32 {
    return load_arg2_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
}

#[no_mangle]
pub fn load_arg3_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize) -> f64 {
    return load_arg3_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
}

#[no_mangle]
pub fn load_ret0_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize) -> f64 {
    return load_ret0_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
}

#[no_mangle]
pub fn load_ret1_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize) -> f32 {
    return load_ret1_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
}

#[no_mangle]
pub fn load_ret2_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize) -> i32 {
    return load_ret2_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
}

#[no_mangle]
pub fn load_ret3_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize) -> i64 {
    return load_ret3_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
}"
        );
    }

    #[test]
    fn generating_store_generic_instructions() {
        assert_eq!(
            generate_store_generic(0, 1),
            "
#[inline(always)]
fn store_arg0_ret_0_arg_1<T0>(stack_ptr: usize, a0: T0) where T0: Copy {
    let a0_offset: usize = 0; // constant folded
    return wastrumentation_memory_store::<T0>(stack_ptr, a0, a0_offset); // inlined
}",
        );
        assert_eq!(generate_store_generic(5, 5), "
#[inline(always)]
fn store_arg0_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, a0: T0) where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let a0_offset: usize = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>() + size_of::<R4>(); // constant folded
    return wastrumentation_memory_store::<T0>(stack_ptr, a0, a0_offset); // inlined
}

#[inline(always)]
fn store_arg1_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, a1: T1) where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let a1_offset: usize = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>() + size_of::<R4>() + size_of::<T0>(); // constant folded
    return wastrumentation_memory_store::<T1>(stack_ptr, a1, a1_offset); // inlined
}

#[inline(always)]
fn store_arg2_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, a2: T2) where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let a2_offset: usize = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>() + size_of::<R4>() + size_of::<T0>() + size_of::<T1>(); // constant folded
    return wastrumentation_memory_store::<T2>(stack_ptr, a2, a2_offset); // inlined
}

#[inline(always)]
fn store_arg3_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, a3: T3) where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let a3_offset: usize = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>() + size_of::<R4>() + size_of::<T0>() + size_of::<T1>() + size_of::<T2>(); // constant folded
    return wastrumentation_memory_store::<T3>(stack_ptr, a3, a3_offset); // inlined
}

#[inline(always)]
fn store_arg4_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, a4: T4) where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let a4_offset: usize = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>() + size_of::<R4>() + size_of::<T0>() + size_of::<T1>() + size_of::<T2>() + size_of::<T3>(); // constant folded
    return wastrumentation_memory_store::<T4>(stack_ptr, a4, a4_offset); // inlined
}

#[inline(always)]
fn store_ret0_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, r0: R0) where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let r0_offset: usize = 0; // constant folded
    return wastrumentation_memory_store::<R0>(stack_ptr, r0, r0_offset); // inlined
}

#[inline(always)]
fn store_ret1_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, r1: R1) where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let r1_offset: usize = size_of::<R0>(); // constant folded
    return wastrumentation_memory_store::<R1>(stack_ptr, r1, r1_offset); // inlined
}

#[inline(always)]
fn store_ret2_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, r2: R2) where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let r2_offset: usize = size_of::<R0>() + size_of::<R1>(); // constant folded
    return wastrumentation_memory_store::<R2>(stack_ptr, r2, r2_offset); // inlined
}

#[inline(always)]
fn store_ret3_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, r3: R3) where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let r3_offset: usize = size_of::<R0>() + size_of::<R1>() + size_of::<R2>(); // constant folded
    return wastrumentation_memory_store::<R3>(stack_ptr, r3, r3_offset); // inlined
}

#[inline(always)]
fn store_ret4_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, r4: R4) where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let r4_offset: usize = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>(); // constant folded
    return wastrumentation_memory_store::<R4>(stack_ptr, r4, r4_offset); // inlined
}");
    }

    #[test]
    fn generating_store_specialized_instructions() {
        assert_eq!(
            generate_store_specialized(&RustSignature(&get_ret_f64_f32_arg_i32_i64())),
            "
#[no_mangle]
pub fn store_arg0_ret_f64_f32_arg_i32_i64(stack_ptr: usize, a0: i32) {
    return store_arg0_ret_2_arg_2::<f64, f32, i32, i64>(stack_ptr, a0);
}

#[no_mangle]
pub fn store_arg1_ret_f64_f32_arg_i32_i64(stack_ptr: usize, a1: i64) {
    return store_arg1_ret_2_arg_2::<f64, f32, i32, i64>(stack_ptr, a1);
}

#[no_mangle]
pub fn store_ret0_ret_f64_f32_arg_i32_i64(stack_ptr: usize, a0: f64) {
    return store_ret0_ret_2_arg_2::<f64, f32, i32, i64>(stack_ptr, a0);
}

#[no_mangle]
pub fn store_ret1_ret_f64_f32_arg_i32_i64(stack_ptr: usize, a1: f32) {
    return store_ret1_ret_2_arg_2::<f64, f32, i32, i64>(stack_ptr, a1);
}",
        );

        assert_eq!(
            generate_store_specialized(&RustSignature(
                &get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64()
            )),
            "
#[no_mangle]
pub fn store_arg0_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a0: i64) {
    return store_arg0_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a0);
}

#[no_mangle]
pub fn store_arg1_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a1: i32) {
    return store_arg1_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a1);
}

#[no_mangle]
pub fn store_arg2_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a2: f32) {
    return store_arg2_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a2);
}

#[no_mangle]
pub fn store_arg3_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a3: f64) {
    return store_arg3_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a3);
}

#[no_mangle]
pub fn store_ret0_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a0: f64) {
    return store_ret0_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a0);
}

#[no_mangle]
pub fn store_ret1_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a1: f32) {
    return store_ret1_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a1);
}

#[no_mangle]
pub fn store_ret2_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a2: i32) {
    return store_ret2_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a2);
}

#[no_mangle]
pub fn store_ret3_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a3: i64) {
    return store_ret3_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a3);
}"
        );
    }

    #[test]
    fn generating_free_generic_instruction() {
        assert_eq!(
            generate_free_values_buffer_generic(0, 1),
            "
#[inline(always)]
fn free_values_ret_0_arg_1<T0>(ptr: usize) where T0: Copy {
    let to_deallocate = size_of::<T0>(); // constant folded
    stack_deallocate(ptr, to_deallocate); // inlined
    return;
}"
        );
        assert_eq!(generate_free_values_buffer_generic(5, 5), "
#[inline(always)]
fn free_values_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(ptr: usize) where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let to_deallocate = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>() + size_of::<R4>() + size_of::<T0>() + size_of::<T1>() + size_of::<T2>() + size_of::<T3>() + size_of::<T4>(); // constant folded
    stack_deallocate(ptr, to_deallocate); // inlined
    return;
}");
    }

    #[test]
    fn generating_free_specialized_instruction() {
        assert_eq!(
            generate_free_values_buffer_specialized(&RustSignature(&get_ret_f64_f32_arg_i32_i64())),
            "
#[no_mangle]
pub fn free_values_ret_f64_f32_arg_i32_i64(ptr: usize) {
    return free_values_ret_2_arg_2::<f64, f32, i32, i64>(ptr);
}",
        );

        assert_eq!(
            generate_free_values_buffer_specialized(&RustSignature(
                &get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64()
            )),
            "
#[no_mangle]
pub fn free_values_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(ptr: usize) {
    return free_values_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(ptr);
}",
        );
    }

    #[test]
    fn generating_free_types_generic_instruction() {
        assert_eq!(
            generate_free_types_buffer_generic(0, 1),
            "
#[inline(always)]
fn free_types_ret_0_arg_1(ptr: usize) {
    let to_deallocate = size_of::<i32>() * 1; // constant folded
    stack_deallocate(ptr, to_deallocate); // inlined
    return;
}"
        );
        assert_eq!(
            generate_free_types_buffer_generic(5, 5),
            "
#[inline(always)]
fn free_types_ret_5_arg_5(ptr: usize) {
    let to_deallocate = size_of::<i32>() * 10; // constant folded
    stack_deallocate(ptr, to_deallocate); // inlined
    return;
}"
        );
    }

    #[test]
    fn generating_free_types_specialized_instruction() {
        assert_eq!(
            generate_free_types_buffer_specialized(&RustSignature(&get_ret_f64_f32_arg_i32_i64())),
            "
#[no_mangle]
pub fn free_types_ret_f64_f32_arg_i32_i64(ptr: usize) {
    return free_types_ret_2_arg_2(ptr);
}"
        );
        assert_eq!(
            generate_free_types_buffer_specialized(&RustSignature(
                &get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64()
            )),
            "
#[no_mangle]
pub fn free_types_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(ptr: usize) {
    return free_types_ret_4_arg_4(ptr);
}"
        );
    }

    #[test]
    fn generating_store_rets_generic_instruction() {
        assert_eq!(
            generate_store_rets_generic(0, 1),
            "
#[inline(always)]
fn store_rets_ret_0_arg_1<T0>(_stack_ptr: usize) where T0: Copy {
    return;
}",
        );
        assert_eq!(generate_store_rets_generic(5, 5), "
#[inline(always)]
fn store_rets_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, a0: R0, a1: R1, a2: R2, a3: R3, a4: R4) where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    // store a0
    store_ret0_ret_5_arg_5::<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr, a0);
    // store a1
    store_ret1_ret_5_arg_5::<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr, a1);
    // store a2
    store_ret2_ret_5_arg_5::<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr, a2);
    // store a3
    store_ret3_ret_5_arg_5::<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr, a3);
    // store a4
    store_ret4_ret_5_arg_5::<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr, a4);
    return;
}");
    }

    #[test]
    fn generating_store_rets_specialized_instruction() {
        assert_eq!(
            generate_store_rets_specialized(&RustSignature(&get_ret_f64_f32_arg_i32_i64())),
            "
#[no_mangle]
pub fn store_rets_ret_f64_f32_arg_i32_i64(stack_ptr: usize, a0: f64, a1: f32) {
    return store_rets_ret_2_arg_2::<f64, f32, i32, i64>(stack_ptr, a0, a1);
}"
        );

        assert_eq!(generate_store_rets_specialized(&RustSignature(&get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64())), "
#[no_mangle]
pub fn store_rets_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a0: f64, a1: f32, a2: i32, a3: i64) {
    return store_rets_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a0, a1, a2, a3);
}");
    }

    #[test]
    fn generating_allocate_types_generic_specialized() {
        assert_eq!(
            generate_allocate_types_buffer_generic(0, 1),
            "
#[inline(always)]
fn allocate_signature_types_buffer_ret_0_arg_1() -> usize {
    let to_allocate = size_of::<i32>() * 1; // constant folded
    let stack_begin = stack_allocate(to_allocate); // inlined
    return stack_begin;
}"
        );

        assert_eq!(
            generate_allocate_types_buffer_generic(5, 5),
            "
#[inline(always)]
fn allocate_signature_types_buffer_ret_5_arg_5() -> usize {
    let to_allocate = size_of::<i32>() * 10; // constant folded
    let stack_begin = stack_allocate(to_allocate); // inlined
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
            generate_allocate_types_buffer_specialized(&RustSignature(&signature_empty)),
            "
#[no_mangle]
pub fn allocate_types_ret_arg() -> usize {
    let types_buffer = allocate_signature_types_buffer_ret_0_arg_0();
    return types_buffer;
}"
        );
        let signature_0 = Signature {
            return_types: vec![WasmType::F64, WasmType::F32],
            argument_types: vec![WasmType::I32, WasmType::I64],
        };
        assert_eq!(
            generate_allocate_types_buffer_specialized(&RustSignature(&signature_0)),
            "
#[no_mangle]
pub fn allocate_types_ret_f64_f32_arg_i32_i64() -> usize {
    let types_buffer = allocate_signature_types_buffer_ret_2_arg_2();
    wastrumentation_memory_store::<i32>(types_buffer, 3, size_of::<i32>()*0);
    wastrumentation_memory_store::<i32>(types_buffer, 1, size_of::<i32>()*1);
    wastrumentation_memory_store::<i32>(types_buffer, 0, size_of::<i32>()*2);
    wastrumentation_memory_store::<i32>(types_buffer, 2, size_of::<i32>()*3);
    return types_buffer;
}"
        );
        let signature_1 = Signature {
            return_types: vec![WasmType::F64, WasmType::F32, WasmType::I32, WasmType::I64],
            argument_types: vec![WasmType::I64, WasmType::I32, WasmType::F32, WasmType::F64],
        };
        assert_eq!(
            generate_allocate_types_buffer_specialized(&RustSignature(&signature_1)),
            "
#[no_mangle]
pub fn allocate_types_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64() -> usize {
    let types_buffer = allocate_signature_types_buffer_ret_4_arg_4();
    wastrumentation_memory_store::<i32>(types_buffer, 3, size_of::<i32>()*0);
    wastrumentation_memory_store::<i32>(types_buffer, 1, size_of::<i32>()*1);
    wastrumentation_memory_store::<i32>(types_buffer, 0, size_of::<i32>()*2);
    wastrumentation_memory_store::<i32>(types_buffer, 2, size_of::<i32>()*3);
    wastrumentation_memory_store::<i32>(types_buffer, 2, size_of::<i32>()*4);
    wastrumentation_memory_store::<i32>(types_buffer, 0, size_of::<i32>()*5);
    wastrumentation_memory_store::<i32>(types_buffer, 1, size_of::<i32>()*6);
    wastrumentation_memory_store::<i32>(types_buffer, 3, size_of::<i32>()*7);
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
            generate_allocate_types_buffer_specialized(&RustSignature(&signature_2)),
            "
#[no_mangle]
pub fn allocate_types_ret_ref_func_ref_func_arg_ref_extern_ref_extern() -> usize {
    let types_buffer = allocate_signature_types_buffer_ret_2_arg_2();
    wastrumentation_memory_store::<i32>(types_buffer, 4, size_of::<i32>()*0);
    wastrumentation_memory_store::<i32>(types_buffer, 4, size_of::<i32>()*1);
    wastrumentation_memory_store::<i32>(types_buffer, 5, size_of::<i32>()*2);
    wastrumentation_memory_store::<i32>(types_buffer, 5, size_of::<i32>()*3);
    return types_buffer;
}"
        );
    }
}
