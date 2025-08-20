use std::{collections::HashSet, marker::PhantomData, ops::Deref, vec};

use crate::lib_compile::rust::options::{ManifestSource, RustSource, RustSourceCode};
use crate::lib_compile::rust::Rust;
use rust_to_wasm_compiler::WasiSupport;
use wastrumentation::wasm_constructs::RefType;
use wastrumentation::{
    compiler::{LibGeneratable, Library},
    wasm_constructs::{Signature, SignatureSide, WasmType},
};

#[cfg(test)]
mod tests;

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

    fn wasmvalue_constructor_for_type(wasm_type: &WasmType) -> String {
        match wasm_type {
            WasmType::I32 => "WasmValue::new_i32".to_string(),
            WasmType::F32 => "WasmValue::new_f32".to_string(),
            WasmType::I64 => "WasmValue::new_i64".to_string(),
            WasmType::F64 => "WasmValue::new_f64".to_string(),
            WasmType::Ref(RefType::ExternRef) => "WasmValue::new_extern_ref".to_string(),
            WasmType::Ref(RefType::FuncRef) => "WasmValue::new_func_ref".to_string(),
        }
    }

    fn wasmvalue_accessor_for_type(wasm_type: &WasmType) -> String {
        match wasm_type {
            WasmType::I32 => "i32".to_string(),
            WasmType::F32 => "f32".to_string(),
            WasmType::I64 => "i64".to_string(),
            WasmType::F64 => "f64".to_string(),
            WasmType::Ref(RefType::ExternRef) => "extern_ref".to_string(),
            WasmType::Ref(RefType::FuncRef) => "func_ref".to_string(),
        }
    }
}

fn generate_allocate_generic(rets_count: usize, args_count: usize) -> String {
    let generic_name = RustSignature::generic_rust_name(rets_count, args_count);
    let total_slots = rets_count + args_count;

    if total_slots == 0 {
        return format!(
            "
#[inline(always)]
fn allocate_{generic_name}() -> usize {{
    0 // Fake shadow pointer for empty signatures
}}"
        );
    }

    let store_args = (0..args_count)
        .map(|n| {
            let slot_index = rets_count + n;
            format!("    wastrumentation_memory_store(frame_ptr, a{n}, {slot_index});")
        })
        .collect::<Vec<String>>()
        .join("\n");

    let signature = (0..args_count)
        .map(|n| format!("a{n}: WasmValue"))
        .collect::<Vec<String>>()
        .join(", ");

    format!(
        "
#[inline(always)]
fn allocate_{generic_name}({signature}) -> usize {{
    let frame_ptr = stack_allocate_values({total_slots});
{store_args}
    frame_ptr
}}"
    )
}

fn generate_allocate_specialized(signature: &RustSignature) -> String {
    let signature_args = &signature.argument_types;
    let signature_args_typs_ident = signature_args
        .iter()
        .enumerate()
        .map(|(index, ty)| format!("a{index}: {ty}"))
        .collect::<Vec<String>>()
        .join(", ");

    let args_converted = signature_args
        .iter()
        .enumerate()
        .map(|(index, ty)| {
            let constructor = RustSignature::wasmvalue_constructor_for_type(ty);
            format!("{constructor}(a{index})")
        })
        .collect::<Vec<String>>()
        .join(", ");

    let mangled_name = signature.generate_allocate_values_buffer_name();
    let mangled_by_count_name = signature.mangled_rust_name_by_count();

    format!(
        "
#[no_mangle]
pub extern \"C\" fn {mangled_name}({signature_args_typs_ident}) -> usize {{
    allocate_{mangled_by_count_name}({args_converted})
}}
"
    )
}

fn generate_allocate_types_buffer_generic(rets_count: usize, args_count: usize) -> String {
    let total_types = rets_count + args_count;
    let generic_name = RustSignature::generic_rust_name(rets_count, args_count);
    format!(
        "
#[inline(always)]
fn allocate_signature_types_buffer_{generic_name}() -> usize {{
    stack_allocate_types({total_types}) // inlined
}}"
    )
}

fn generate_allocate_types_buffer_specialized(signature: &RustSignature) -> String {
    let signature_rets = &signature.return_types;
    let signature_args = &signature.argument_types;

    let all_stores_followed_by_return = signature_rets
        .iter()
        .chain(signature_args.iter())
        .map(WasmType::runtime_enum_variant)
        .enumerate()
        .map(|(index, enum_variant)| {
            format!("    wastrumentation_stack_store_type(types_buffer, {index}, RuntimeType::{enum_variant});")
        })
        .collect::<Vec<String>>()
        .join("\n");

    let mangled_name = signature.generate_allocate_types_buffer_name();
    let mangled_by_count_name = signature.mangled_rust_name_by_count();

    format!(
        "
#[no_mangle]
pub extern \"C\" fn {mangled_name}() -> usize {{
    let types_buffer = allocate_signature_types_buffer_{mangled_by_count_name}();
{all_stores_followed_by_return}
    types_buffer
}}"
    )
}

fn generate_load_generic(rets_count: usize, args_count: usize) -> String {
    let generic_name = RustSignature::generic_rust_name(rets_count, args_count);

    let all_arg_loads = (0..args_count).map(|n| {
        let slot_index = rets_count + n;
        format!(
            "
#[inline(always)]
fn load_arg{n}_{generic_name}(stack_ptr: usize) -> WasmValue {{
    wastrumentation_memory_load(stack_ptr, {slot_index})
}}"
        )
    });

    let all_ret_loads = (0..rets_count).map(|n| {
        format!(
            "
#[inline(always)]
fn load_ret{n}_{generic_name}(stack_ptr: usize) -> WasmValue {{
    wastrumentation_memory_load(stack_ptr, {n})
}}"
        )
    });

    all_arg_loads
        .chain(all_ret_loads)
        .collect::<Vec<String>>()
        .join("\n")
}

fn generate_load_specialized(signature: &RustSignature) -> String {
    let signature_rets = &signature.return_types;
    let signature_args = &signature.argument_types;
    let mangled_by_count_name = signature.mangled_rust_name_by_count();

    let all_arg_loads = signature_args.iter().enumerate().map(|(index, arg_type)| {
        let mangled_name = signature.generate_load_name(SignatureSide::Argument, index);
        let accessor = RustSignature::wasmvalue_accessor_for_type(arg_type);
        format!(
            "
#[no_mangle]
pub extern \"C\" fn {mangled_name}(stack_ptr: usize) -> {arg_type} {{
    let val = load_arg{index}_{mangled_by_count_name}(stack_ptr);
    unsafe {{ val.{accessor} }}
}}"
        )
    });

    let all_ret_loads = signature_rets.iter().enumerate().map(|(index, ret_type)| {
        let mangled_name = signature.generate_load_name(SignatureSide::Return, index);
        let accessor = RustSignature::wasmvalue_accessor_for_type(ret_type);
        format!(
            "
#[no_mangle]
pub extern \"C\" fn {mangled_name}(stack_ptr: usize) -> {ret_type} {{
    let val = load_ret{index}_{mangled_by_count_name}(stack_ptr);
    unsafe {{ val.{accessor} }}
}}"
        )
    });

    all_arg_loads
        .chain(all_ret_loads)
        .collect::<Vec<String>>()
        .join("\n")
}

fn generate_store_generic(rets_count: usize, args_count: usize) -> String {
    let generic_name = RustSignature::generic_rust_name(rets_count, args_count);

    let all_arg_stores = (0..args_count).map(|n| {
        let slot_index = rets_count + n;
        format!(
            "
#[inline(always)]
fn store_arg{n}_{generic_name}(stack_ptr: usize, a{n}: WasmValue) {{
    wastrumentation_memory_store(stack_ptr, a{n}, {slot_index});
}}"
        )
    });

    let all_ret_stores = (0..rets_count).map(|n| {
        format!(
            "
#[inline(always)]
fn store_ret{n}_{generic_name}(stack_ptr: usize, r{n}: WasmValue) {{
    wastrumentation_memory_store(stack_ptr, r{n}, {n});
}}"
        )
    });

    all_arg_stores
        .chain(all_ret_stores)
        .collect::<Vec<String>>()
        .join("\n")
}

fn generate_store_specialized(signature: &RustSignature) -> String {
    let signature_rets = &signature.return_types;
    let signature_args = &signature.argument_types;
    let mangled_by_count_name = signature.mangled_rust_name_by_count();

    let all_arg_stores = signature_args.iter().enumerate().map(|(index, arg_type)| {
        let mangled_name = signature.generate_store_name(SignatureSide::Argument, index);
        let constructor = RustSignature::wasmvalue_constructor_for_type(arg_type);
        format!(
            "
#[no_mangle]
pub extern \"C\" fn {mangled_name}(stack_ptr: usize, a{index}: {arg_type}) {{
    store_arg{index}_{mangled_by_count_name}(stack_ptr, {constructor}(a{index}));
}}"
        )
    });

    let all_ret_stores = signature_rets.iter().enumerate().map(|(index, ret_type)| {
        let mangled_name = signature.generate_store_name(SignatureSide::Return, index);
        let constructor = RustSignature::wasmvalue_constructor_for_type(ret_type);
        format!(
            "
#[no_mangle]
pub extern \"C\" fn {mangled_name}(stack_ptr: usize, a{index}: {ret_type}) {{
    store_ret{index}_{mangled_by_count_name}(stack_ptr, {constructor}(a{index}));
}}"
        )
    });

    all_arg_stores
        .chain(all_ret_stores)
        .collect::<Vec<String>>()
        .join("\n")
}

fn generate_free_values_buffer_generic(rets_count: usize, args_count: usize) -> String {
    let generic_name = RustSignature::generic_rust_name(rets_count, args_count);
    let total_slots = rets_count + args_count;

    if total_slots == 0 {
        return format!(
            "
#[inline(always)]
fn free_values_{generic_name}(_ptr: usize) {{
    // No deallocation for empty signatures
}}"
        );
    }

    format!(
        "
#[inline(always)]
fn free_values_{generic_name}(ptr: usize) {{
    stack_deallocate_values(ptr, {total_slots});
}}"
    )
}

fn generate_free_values_buffer_specialized(signature: &RustSignature) -> String {
    format!(
        "
#[no_mangle]
pub extern \"C\" fn {}(ptr: usize) {{
    free_values_{}(ptr);
}}",
        signature.generate_free_values_buffer_name(),
        signature.mangled_rust_name_by_count(),
    )
}

fn generate_free_types_buffer_generic(rets_count: usize, args_count: usize) -> String {
    let generic_name = RustSignature::generic_rust_name(rets_count, args_count);
    let total_types = rets_count + args_count;

    format!(
        "
#[inline(always)]
fn free_types_{generic_name}(ptr: usize) {{
    stack_deallocate_types(ptr, {total_types});
}}"
    )
}

fn generate_free_types_buffer_specialized(signature: &RustSignature) -> String {
    format!(
        "
#[no_mangle]
pub extern \"C\" fn {}(ptr: usize) {{
    free_types_{}(ptr);
}}",
        signature.generate_free_types_buffer_name(),
        signature.mangled_rust_name_by_count(),
    )
}

fn generate_store_rets_generic(rets_count: usize, args_count: usize) -> String {
    let generic_name = RustSignature::generic_rust_name(rets_count, args_count);

    let array_of_rets_signature = (0..rets_count).map(|n| format!("a{n}: WasmValue"));
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
        .map(|n| format!("    store_ret{n}_{generic_name}(stack_ptr, a{n});"))
        .collect::<Vec<String>>()
        .join("\n");

    format!(
        "
#[inline(always)]
fn store_rets_{generic_name}({total_signature}) {{
{all_stores}
}}"
    )
}

fn generate_store_rets_specialized(signature: &RustSignature) -> String {
    let signature_rets = &signature.return_types;

    let rets_signature = signature_rets
        .iter()
        .enumerate()
        .map(|(index, ty)| format!("a{index}: {ty}"));

    let total_signature = (vec![String::from("stack_ptr: usize")])
        .into_iter()
        .chain(rets_signature)
        .collect::<Vec<String>>()
        .join(", ");

    let args_converted = signature_rets
        .iter()
        .enumerate()
        .map(|(index, ty)| {
            let constructor = RustSignature::wasmvalue_constructor_for_type(ty);
            format!("{constructor}(a{index})")
        })
        .collect::<Vec<String>>()
        .join(", ");

    let total_arguments = if signature_rets.is_empty() {
        "stack_ptr".to_string()
    } else {
        format!("stack_ptr, {args_converted}")
    };

    let mangled_name = signature.generate_store_rets_name();
    let mangled_by_count_name = signature.mangled_rust_name_by_count();

    format!(
        "
#[no_mangle]
pub extern \"C\" fn {mangled_name}({total_signature}) {{
    store_rets_{mangled_by_count_name}({total_arguments});
}}"
    )
}

// The `#[allow(unused)]` attributes are present to allow
// for the generated code to not necesarily make use of all
// surrounding functions.

const LIB_BOILERPLATE: &str = r#"
// TODO: Allow to enable / disable `no_std` based on some flag
#![no_std]

#![allow(clippy::let_and_return)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::inline_always)]

use core::alloc::{GlobalAlloc, Layout};
use core::mem::{align_of, size_of};

extern crate wee_alloc;
#[global_allocator]
pub static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[allow(unused)]
const COMPILE_TIME_CHECKS: bool = {
    assert!(size_of::<RuntimeType>() == size_of::<i32>());
    true
};

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Up-front, it is not known which variants will be constructed
enum RuntimeType {
    I32 = 0,
    F32 = 1,
    I64 = 2,
    F64 = 3,
    FuncRef = 4,
    ExternRef = 5,
}

impl From<RuntimeType> for i32 {
    #[inline(always)]
    fn from(value: RuntimeType) -> Self {
        value as i32
    }
}

#[cfg(debug_assertions)]
impl RuntimeType {
    fn assert_can_come_from(value: i32, cast_runtime_type: Self) {
        let slow_convert = match value {
            0 => Self::I32,
            1 => Self::F32,
            2 => Self::I64,
            3 => Self::F64,
            4 => Self::FuncRef,
            5 => Self::ExternRef,
            _ => panic!("cannot cast {value} to runtime type"),
        };
        assert_eq!(cast_runtime_type, slow_convert);
    }
}

impl From<i32> for RuntimeType {
    #[inline(always)]
    fn from(value: i32) -> Self {
        let cast = unsafe { core::mem::transmute::<i32, RuntimeType>(value) };

        #[cfg(debug_assertions)]
        RuntimeType::assert_can_come_from(value, cast);

        cast
    }
}

// Union for WASM values
#[repr(C)]
#[derive(Clone, Copy)]
pub union WasmValue {
    pub i32: i32,
    pub f32: f32,
    pub i64: i64,
    pub f64: f64,
    pub ref_type: RefType,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub enum RefType {
    FuncRef,
    ExternRef,
}

impl WasmValue {
    #[inline(always)] #[must_use]
    pub const fn new_i32(val: i32) -> Self { WasmValue { i32: val } }
    #[inline(always)] #[must_use]
    pub const fn new_f32(val: f32) -> Self { WasmValue { f32: val } }
    #[inline(always)] #[must_use]
    pub const fn new_i64(val: i64) -> Self { WasmValue { i64: val } }
    #[inline(always)] #[must_use]
    pub const fn new_f64(val: f64) -> Self { WasmValue { f64: val } }
    #[inline(always)] #[must_use]
    pub const fn new_func_ref() -> Self { WasmValue { ref_type: RefType::FuncRef } }
    #[inline(always)] #[must_use]
    pub const fn new_extern_ref() -> Self { WasmValue { ref_type: RefType::ExternRef } }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_panic: &core::panic::PanicInfo<'_>) -> ! {
    #[cfg(not(target_arch = "wasm32"))]
    core::unreachable!();
    #[cfg(target_arch = "wasm32")]
    core::arch::wasm32::unreachable()
}

#[inline(always)]
fn wastrumentation_memory_load(stack_ptr: usize, offset: usize) -> WasmValue {
    let ptr = stack_ptr as *const WasmValue;
    unsafe { *ptr.add(offset) }
}

#[inline(always)]
fn wastrumentation_memory_store(stack_ptr: usize, value: WasmValue, offset: usize) {
    let ptr = stack_ptr as *mut WasmValue;
    unsafe { *ptr.add(offset) = value };
}

// Pre-compute common layout for RuntimeType
const RUNTIME_TYPE_LAYOUT: Layout = unsafe {
    Layout::from_size_align_unchecked(size_of::<RuntimeType>(), align_of::<RuntimeType>())
};

#[inline(always)] #[allow(unused)]
fn stack_allocate_types(count: usize) -> usize {
    let size = size_of::<RuntimeType>() * count;
    let layout = unsafe { Layout::from_size_align_unchecked(size, RUNTIME_TYPE_LAYOUT.align()) };
    unsafe { ALLOC.alloc(layout) as usize }
}

#[inline(always)] #[allow(unused)]
fn stack_deallocate_types(ptr: usize, count: usize) {
    let size = size_of::<RuntimeType>() * count;
    let layout = unsafe { Layout::from_size_align_unchecked(size, RUNTIME_TYPE_LAYOUT.align()) };
    let ptr = ptr as *mut u8;
    unsafe { ALLOC.dealloc(ptr, layout) };
}

// Pre-compute common layout for WasmValue
const WASM_VALUE_LAYOUT: Layout =
    unsafe { Layout::from_size_align_unchecked(size_of::<WasmValue>(), align_of::<WasmValue>()) };

#[inline(always)] #[allow(unused)]
fn stack_allocate_values(num_values: usize) -> usize {
    let size = num_values * WASM_VALUE_LAYOUT.size();
    let layout = unsafe { Layout::from_size_align_unchecked(size, WASM_VALUE_LAYOUT.align()) };
    (unsafe { ALLOC.alloc(layout) } as usize)
}

#[inline(always)] #[allow(unused)]
fn stack_deallocate_values(ptr: usize, num_values: usize) {
    let size = num_values * WASM_VALUE_LAYOUT.size();
    let layout = unsafe { Layout::from_size_align_unchecked(size, WASM_VALUE_LAYOUT.align()) };
    unsafe { ALLOC.dealloc(ptr as *mut u8, layout) };
}

#[inline(always)] #[allow(unused)]
fn wastrumentation_stack_store_type(ptr: usize, offset: usize, ty: RuntimeType) {
    let ptr = ptr as *mut RuntimeType;
    unsafe { *ptr.add(offset) = ty };
}

#[no_mangle]
pub extern "C" fn wastrumentation_stack_load_type(ptr: usize, offset: usize) -> i32 {
    let ptr = ptr as *const RuntimeType;
    let ty = unsafe { *ptr.add(offset) };
    ty.into()
}

// Optimized load/store functions
#[no_mangle]
pub extern "C" fn wastrumentation_stack_load_i32(ptr: usize, offset: usize) -> i32 {
    let wasm_val = wastrumentation_memory_load(ptr, offset);
    unsafe { wasm_val.i32 }
}

#[no_mangle]
pub extern "C" fn wastrumentation_stack_load_f32(ptr: usize, offset: usize) -> f32 {
    let wasm_val = wastrumentation_memory_load(ptr, offset);
    unsafe { wasm_val.f32 }
}

#[no_mangle]
pub extern "C" fn wastrumentation_stack_load_i64(ptr: usize, offset: usize) -> i64 {
    let wasm_val = wastrumentation_memory_load(ptr, offset);
    unsafe { wasm_val.i64 }
}

#[no_mangle]
pub extern "C" fn wastrumentation_stack_load_f64(ptr: usize, offset: usize) -> f64 {
    let wasm_val = wastrumentation_memory_load(ptr, offset);
    unsafe { wasm_val.f64 }
}

#[no_mangle]
pub extern "C" fn wastrumentation_stack_store_i32(ptr: usize, value: i32, offset: usize) {
    wastrumentation_memory_store(ptr, WasmValue::new_i32(value), offset);
}

#[no_mangle]
pub extern "C" fn wastrumentation_stack_store_f32(ptr: usize, value: f32, offset: usize) {
    wastrumentation_memory_store(ptr, WasmValue::new_f32(value), offset);
}

#[no_mangle]
pub extern "C" fn wastrumentation_stack_store_i64(ptr: usize, value: i64, offset: usize) {
    wastrumentation_memory_store(ptr, WasmValue::new_i64(value), offset);
}

#[no_mangle]
pub extern "C" fn wastrumentation_stack_store_f64(ptr: usize, value: f64, offset: usize) {
    wastrumentation_memory_store(ptr, WasmValue::new_f64(value), offset);
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
