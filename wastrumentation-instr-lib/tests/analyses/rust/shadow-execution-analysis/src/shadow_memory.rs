// Imports

use super::LoadOffset;
use super::LoadOperation::{self, F32Load, F64Load, I32Load, I64Load}; // Generic
use super::LoadOperation::{I32Load16S, I32Load16U, I32Load8S, I32Load8U}; // I32
use super::LoadOperation::{I64Load16S, I64Load16U, I64Load32S, I64Load32U, I64Load8S, I64Load8U}; // I64

use super::StoreOffset;
use super::StoreOperation::{self, F32Store, F64Store, I32Store, I64Store}; // Generic
use super::StoreOperation::{I32Store16, I32Store8}; // I32
use super::StoreOperation::{I64Store16, I64Store32, I64Store8}; // I64

use super::WasmValue;

use core::ptr::addr_of_mut;

//////////////////////////////////
// compile-time severity checks //
//////////////////////////////////

static mut TRGT_MEMORY_INITIALIZED_EMPTY: bool = false;
static mut TRGT_MEMORY_ONLY_AFFECTED_INTERNALLY: bool = false;

static mut SHADOW_MEMRY: Vec<u8> = vec![];
pub(crate) fn assert_shadow_memory(loaded_value: &WasmValue, shadow_value: &WasmValue) {
    let target_memory_initialized_empty = unsafe { TRGT_MEMORY_INITIALIZED_EMPTY };
    let target_memory_only_affected_internally = unsafe { TRGT_MEMORY_ONLY_AFFECTED_INTERNALLY };
    let assertion_should_hold =
        target_memory_initialized_empty && target_memory_only_affected_internally;
    if assertion_should_hold {
        assert_eq!(loaded_value, shadow_value);
    }
}

////////////////
// Public API //
////////////////

#[must_use]
pub(crate) fn shadow_memory_load(
    store_index: WasmValue,
    offset: &'_ LoadOffset,
    operation: LoadOperation,
) -> WasmValue {
    let ptr = usize::try_from(store_index.as_i32()).unwrap();
    let offset = usize::try_from(offset.value()).unwrap();
    let addr = ptr + offset;
    let shadow_memory = unsafe { addr_of_mut!(SHADOW_MEMRY).as_mut().unwrap() };
    grow_if_out_of_bounds(shadow_memory, addr, operation.target_value_size());
    match operation {
        I32Load => memory_load::<i32>(shadow_memory, addr).into(),
        I64Load => memory_load::<i64>(shadow_memory, addr).into(),
        F32Load => memory_load::<f32>(shadow_memory, addr).into(),
        F64Load => memory_load::<f64>(shadow_memory, addr).into(),
        I32Load8S => memory_load_sub::<i32, u8>(shadow_memory, addr).into(),
        I32Load8U => memory_load_sub::<i32, u8>(shadow_memory, addr).into(),
        I32Load16S => memory_load_sub::<i32, i16>(shadow_memory, addr).into(),
        I32Load16U => memory_load_sub::<i32, u16>(shadow_memory, addr).into(),
        I64Load8S => memory_load_sub::<i64, i8>(shadow_memory, addr).into(),
        I64Load8U => memory_load_sub::<i64, u8>(shadow_memory, addr).into(),
        I64Load16S => memory_load_sub::<i64, i16>(shadow_memory, addr).into(),
        I64Load16U => memory_load_sub::<i64, u16>(shadow_memory, addr).into(),
        I64Load32S => memory_load_sub::<i64, i32>(shadow_memory, addr).into(),
        I64Load32U => memory_load_sub::<i64, u32>(shadow_memory, addr).into(),
    }
}

pub(crate) fn shadow_memory_store(
    store_index: WasmValue,
    shadow_value: WasmValue,
    offset: &'_ StoreOffset,
    operation: StoreOperation,
) {
    let ptr = usize::try_from(store_index.as_i32()).unwrap();
    let offset = usize::try_from(offset.value()).unwrap();
    let addr = ptr + offset;
    let shadow_memory = unsafe { addr_of_mut!(SHADOW_MEMRY).as_mut().unwrap() };
    grow_if_out_of_bounds(shadow_memory, addr, shadow_value.type_().size());
    match operation {
        I32Store => memory_store::<i32>(shadow_memory, addr, shadow_value.as_i32()),
        I64Store => memory_store::<i64>(shadow_memory, addr, shadow_value.as_i64()),
        F32Store => memory_store::<f32>(shadow_memory, addr, shadow_value.as_f32()),
        F64Store => memory_store::<f64>(shadow_memory, addr, shadow_value.as_f64()),
        I32Store8 => memory_store::<u8>(shadow_memory, addr, shadow_value.as_i32() as u8),
        I64Store8 => memory_store::<u8>(shadow_memory, addr, shadow_value.as_i64() as u8),
        I32Store16 => memory_store::<u16>(shadow_memory, addr, shadow_value.as_i32() as u16),
        I64Store16 => memory_store::<u16>(shadow_memory, addr, shadow_value.as_i64() as u16),
        I64Store32 => memory_store::<u32>(shadow_memory, addr, shadow_value.as_i64() as u32),
    }
}

/////////////////
// private API //
/////////////////

fn grow_if_out_of_bounds<T: Default>(buffer: &mut Vec<T>, index: usize, target_size: usize) {
    if buffer.len() <= index + target_size {
        buffer.resize_with(index + target_size, T::default);
    }
}

fn assert_memory_bounds<Value>(memory: &[u8], address: usize) {
    // Ensure size follows development expectation
    let size_of_t = core::mem::size_of::<Value>();
    assert!(size_of_t <= 8, "Value larger values than 8 bytes");
    // Ensure address does not overflow
    let eventual_pointer = address
        .checked_add(size_of::<Value>())
        .expect("address computation overflow");
    // Ensure the address is within bounds
    let out_too_big = eventual_pointer <= memory.len();
    assert!(out_too_big, "Address out of bounds");
}

// Safely loads a value of type T from memory at a given address
fn memory_load<T: Copy>(memory: &[u8], address: usize) -> T {
    assert_memory_bounds::<T>(memory, address);
    // Safely create a slice from memory and cast it to a reference of type T
    let eventual_pointer = memory.as_ptr();
    // Copy the bytes and convert them into type T
    unsafe { core::ptr::read_unaligned(eventual_pointer.add(address) as *const T) }
}

// Safely stores a value of type T into memory at a given address
fn memory_store<T: Copy>(memory: &mut [u8], address: usize, value: T) {
    assert_memory_bounds::<T>(memory, address);
    // Safely create a slice from memory and cast it to a reference of type T
    let eventual_pointer = memory.as_ptr();
    // Copy the bytes and convert them into type T
    unsafe { core::ptr::write_unaligned(eventual_pointer.add(address) as *mut T, value) }
}

fn memory_load_sub<StoreValue, Sub>(memory: &[u8], address: usize) -> StoreValue
where
    StoreValue: TryFrom<Sub>,
    StoreValue::Error: core::fmt::Debug,
    Sub: Copy,
{
    let loaded_value = memory_load::<Sub>(memory, address);
    StoreValue::try_from(loaded_value).expect(concat! {
            "Conversion of ", stringify!(Sub), " to ", stringify!(StoreValue), " failed."
    })
}
