use std::{collections::HashSet, marker::PhantomData, ops::Deref, vec};

use crate::lib_compile::rust::options::{ManifestSource, RustSource, RustSourceCode};
use crate::lib_compile::rust::Rust;
use std::iter;
use rust_to_wasm_compiler::WasiSupport;
use wastrumentation::{
    compiler::{LibGeneratable, Library},
    wasm_constructs::{Signature, SignatureSide, WasmType},
};

#[cfg(test)]
mod tests;

// TODO: since this holds:

// fn foo() {
//     let tuple_size = size_of::<(f64, i32, i32)>();
//     let sum_size = size_of::<f64>() + size_of::<i32>() + size_of::<i32>();
//     assert_eq!(tuple_size, sum_size)
// }

// => WARNING: Due to alignment, some structures are padded. As such, size_of::<struct _X(i32,u8)>() != size_of::<(i32,u8)>() ...

// I could move all "+" expressions to a tuple variant ... Should not change anything, but 'enforce' constant folding

impl LibGeneratable for Rust {
    fn generate_lib(signatures: &[Signature]) -> Library<Self> {
        let (manifest_source, rust_source) = generate_lib(signatures);
        Library::<Self> {
            content: RustSource::SourceCode(WasiSupport::Disabled, manifest_source, rust_source),
            language: PhantomData,
        }
    }
}

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
            let tys = Self::rust_generics(rets_count, args_count);
            // types shifted with an offset of 1
            let tys1 = &tys.as_slice()[1..];
            tys.iter()
                .zip(tys1.iter().chain(iter::once(&"()".to_string())))
                .fold("0".to_string(), |state, (ty, ty1)|
                    format!("{state} + size_of::<{ty}>() + roundup::<{ty1}>(size_of::<{ty}>() + {state})"))
        }
    }
}

fn generics_offset(position: usize, rets_count: usize, args_offset: usize) -> String {
    if position == 0 {
        return "0".into();
    };

    let ret_names  =  (0..rets_count).map(|n| format!("R{n}")).collect::<Vec<String>>();
    let ret_names1 = &ret_names.as_slice()[1..];
    let arg_names = (0..args_offset).map(|n| format!("T{n}")).collect::<Vec<String>>();

    ret_names.iter().chain(arg_names.iter())
        .zip(ret_names1.iter().chain(arg_names.iter()).chain(iter::once(&"()".to_string())))
        .take(position)
        .fold("0".to_string(), |state, (ty, ty1)| format!("{state} + size_of::<{ty}>() + roundup::<{ty1}>(size_of::<{ty}>() + {state})"))
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
fn roundup<T2>(siz: usize) -> usize {
    if siz % align_of::<T2>() == 0 { return 0; }
    align_of::<T2>() - (siz % align_of::<T2>())
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
