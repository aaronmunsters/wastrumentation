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
use std::cell::RefCell;

//////////////////////////////////
// compile-time severity checks //
//////////////////////////////////

const TRGT_MEMORY_INITIALIZED_EMPTY: bool = false;
const TRGT_MEMORY_ONLY_AFFECTED_INTERNALLY: bool = false;

pub(crate) fn assert_shadow_memory(loaded_value: &WasmValue, shadow_value: &WasmValue) {
    if TRGT_MEMORY_INITIALIZED_EMPTY && TRGT_MEMORY_ONLY_AFFECTED_INTERNALLY {
        debug_assert_eq!(loaded_value, shadow_value);
    }
}

////////////////
// Public API //
////////////////

thread_local! {
    pub(crate) static SHADOW_MEMORY: RefCell<Memory> = const { RefCell::new(Memory(vec![])) };
}

pub(crate) struct Memory(Vec<u8>);

impl Memory {
    #[must_use]
    pub(crate) fn load(
        &mut self,
        store_index: &WasmValue,
        offset: &'_ LoadOffset,
        operation: LoadOperation,
    ) -> WasmValue {
        let ptr = usize::try_from(store_index.as_i32()).unwrap();
        let offset = usize::try_from(offset.value()).unwrap();
        let addr = ptr + offset;
        self.grow_if_out_of_bounds(addr, operation.target_value_size());
        match operation {
            I32Load => self.memory_load::<i32>(addr).into(),
            I64Load => self.memory_load::<i64>(addr).into(),
            F32Load => self.memory_load::<f32>(addr).into(),
            F64Load => self.memory_load::<f64>(addr).into(),
            I32Load8S => self.memory_load_sub::<i32, i8>(addr).into(),
            I32Load8U => self.memory_load_sub::<i32, u8>(addr).into(),
            I32Load16S => self.memory_load_sub::<i32, i16>(addr).into(),
            I32Load16U => self.memory_load_sub::<i32, u16>(addr).into(),
            I64Load8S => self.memory_load_sub::<i64, i8>(addr).into(),
            I64Load8U => self.memory_load_sub::<i64, u8>(addr).into(),
            I64Load16S => self.memory_load_sub::<i64, i16>(addr).into(),
            I64Load16U => self.memory_load_sub::<i64, u16>(addr).into(),
            I64Load32S => self.memory_load_sub::<i64, i32>(addr).into(),
            I64Load32U => self.memory_load_sub::<i64, u32>(addr).into(),
        }
    }

    pub(crate) fn store(
        &mut self,
        store_index: &WasmValue,
        shadow_value: &WasmValue,
        offset: &'_ StoreOffset,
        operation: StoreOperation,
    ) {
        let ptr = usize::try_from(store_index.as_i32()).unwrap();
        let offset = usize::try_from(offset.value()).unwrap();
        let addr = ptr + offset;
        self.grow_if_out_of_bounds(addr, shadow_value.type_().size());
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        match operation {
            I32Store => self.memory_store::<i32>(addr, shadow_value.as_i32()),
            I64Store => self.memory_store::<i64>(addr, shadow_value.as_i64()),
            F32Store => self.memory_store::<f32>(addr, shadow_value.as_f32()),
            F64Store => self.memory_store::<f64>(addr, shadow_value.as_f64()),
            I32Store8 => self.memory_store::<u8>(addr, shadow_value.as_i32() as u8),
            I64Store8 => self.memory_store::<u8>(addr, shadow_value.as_i64() as u8),
            I32Store16 => self.memory_store::<u16>(addr, shadow_value.as_i32() as u16),
            I64Store16 => self.memory_store::<u16>(addr, shadow_value.as_i64() as u16),
            I64Store32 => self.memory_store::<u32>(addr, shadow_value.as_i64() as u32),
        }
    }

    #[allow(unused)]
    pub(crate) fn as_ptr(&self) -> *const u8 {
        let Self(buffer) = self;
        buffer.as_ptr()
    }

    #[allow(unused)]
    pub(crate) fn len(&self) -> usize {
        let Self(buffer) = self;
        buffer.len()
    }

    #[allow(unused)]
    pub(crate) fn inner(&self) -> &Vec<u8> {
        let Self(inner) = self;
        inner
    }

    #[allow(unused)]
    pub(crate) fn inner_mut(&mut self) -> &mut Vec<u8> {
        let Self(inner) = self;
        inner
    }

    /////////////////
    // private API //
    /////////////////

    fn grow_if_out_of_bounds(&mut self, index: usize, target_size: usize) {
        let Self(buffer) = self;
        if buffer.len() <= index + target_size {
            buffer.resize_with(index + target_size, u8::default);
        }
    }

    fn assert_memory_bounds<Value>(&self, address: usize) {
        // Ensure size follows development expectation
        let size_of_t = core::mem::size_of::<Value>();
        debug_assert!(size_of_t <= 8, "Value larger values than 8 bytes");
        // Ensure address does not overflow
        let eventual_pointer = address
            .checked_add(size_of::<Value>())
            .expect("address computation overflow");
        // Ensure the address is within bounds
        let Self(buffer) = self;
        let out_too_big = eventual_pointer <= buffer.len();
        debug_assert!(out_too_big, "Address out of bounds");
    }

    // Safely loads a value of type T from memory at a given address
    fn memory_load<T: Copy>(&self, address: usize) -> T {
        self.assert_memory_bounds::<T>(address);
        // Safely create a slice from memory and cast it to a reference of type T
        let Self(buffer) = self;
        let eventual_pointer = buffer.as_ptr();
        // Copy the bytes and convert them into type T
        unsafe { core::ptr::read_unaligned(eventual_pointer.add(address).cast::<T>()) }
    }

    // Safely stores a value of type T into memory at a given address
    fn memory_store<T: Copy>(&mut self, address: usize, value: T) {
        self.assert_memory_bounds::<T>(address);
        // Safely create a slice from memory and cast it to a reference of type T
        let Self(buffer) = self;
        let eventual_pointer = buffer.as_mut_ptr();
        // Copy the bytes and convert them into type T
        unsafe { core::ptr::write_unaligned(eventual_pointer.add(address).cast::<T>(), value) }
    }

    fn memory_load_sub<StoreValue, Sub>(&self, address: usize) -> StoreValue
    where
        StoreValue: TryFrom<Sub>,
        StoreValue::Error: core::fmt::Debug,
        Sub: Copy,
    {
        let loaded_value = self.memory_load::<Sub>(address);
        StoreValue::try_from(loaded_value).expect(concat! {
                "Conversion of ", stringify!(Sub), " to ", stringify!(StoreValue), " failed."
        })
    }
}
