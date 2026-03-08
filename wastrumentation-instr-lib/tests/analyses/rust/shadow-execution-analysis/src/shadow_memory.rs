use crate::{ ShadowMeta, ShadowValue };

use super::LoadOffset;
use super::LoadOperation::{ self, F32Load, F64Load, I32Load, I64Load };
use super::LoadOperation::{ I32Load16S, I32Load16U, I32Load8S, I32Load8U };
use super::LoadOperation::{ I64Load16S, I64Load16U, I64Load32S, I64Load32U, I64Load8S, I64Load8U };

use super::StoreOffset;
use super::StoreOperation::{ self, F32Store, F64Store, I32Store, I64Store };
use super::StoreOperation::{ I32Store16, I32Store8 };
use super::StoreOperation::{ I64Store16, I64Store32, I64Store8 };

use super::WasmValue;

//////////////////////////////////
// compile-time severity checks //
//////////////////////////////////

const TRGT_MEMORY_INITIALIZED_EMPTY: bool = false;
const TRGT_MEMORY_ONLY_AFFECTED_INTERNALLY: bool = false;

pub fn assert_shadow_memory<M: ShadowMeta>(
    loaded_value: &ShadowValue<M>,
    shadow_value: &ShadowValue<M>
) {
    if TRGT_MEMORY_INITIALIZED_EMPTY && TRGT_MEMORY_ONLY_AFFECTED_INTERNALLY {
        debug_assert_eq!(loaded_value, shadow_value);
    }
}

pub struct Memory<M: ShadowMeta> {
    buffer: Vec<(u8, M::ShadowMetaByte)>,
}

impl<M: ShadowMeta> Memory<M> {
    pub const fn new() -> Self {
        Self { buffer: vec![] }
    }
}

impl<M: ShadowMeta> Default for Memory<M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<M: ShadowMeta> Memory<M> {
    #[must_use]
    pub fn load(
        &mut self,
        store_index: &ShadowValue<M>,
        offset: &'_ LoadOffset,
        operation: LoadOperation
    ) -> ShadowValue<M> {
        let ptr = usize::try_from(store_index.value.as_i32()).unwrap();
        let offset = usize::try_from(offset.value()).unwrap();
        let addr = ptr + offset;
        let size = operation.target_value_size();
        self.grow_if_out_of_bounds(addr, size);

        let byte_metas: Vec<M::ShadowMetaByte> = self.buffer[addr..addr + size]
            .iter()
            .map(|(_, m)| m.clone())
            .collect();
        let meta = M::recompose(byte_metas);

        let raw_value: WasmValue = match operation {
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
        };
        ShadowValue::new(raw_value, Some(meta))
    }

    pub fn store(
        &mut self,
        store_index: &ShadowValue<M>,
        shadow_value: &ShadowValue<M>,
        offset: &'_ StoreOffset,
        operation: StoreOperation
    ) {
        let ptr = usize::try_from(store_index.value.as_i32()).unwrap();
        let offset = usize::try_from(offset.value()).unwrap();
        let addr = ptr + offset;
        let size = shadow_value.value.type_().size();
        self.grow_if_out_of_bounds(addr, size);

        let byte_metas = shadow_value.meta.decompose(size);
        let wasm_value = &shadow_value.value;

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let raw_bytes: Vec<u8> = match operation {
            I32Store => wasm_value.as_i32().to_le_bytes().to_vec(),
            I64Store => wasm_value.as_i64().to_le_bytes().to_vec(),
            F32Store => wasm_value.as_f32().to_le_bytes().to_vec(),
            F64Store => wasm_value.as_f64().to_le_bytes().to_vec(),
            I32Store8 => (wasm_value.as_i32() as u8).to_le_bytes().to_vec(),
            I64Store8 => (wasm_value.as_i64() as u8).to_le_bytes().to_vec(),
            I32Store16 => (wasm_value.as_i32() as u16).to_le_bytes().to_vec(),
            I64Store16 => (wasm_value.as_i64() as u16).to_le_bytes().to_vec(),
            I64Store32 => (wasm_value.as_i64() as u32).to_le_bytes().to_vec(),
        };

        for (i, (byte, bm)) in raw_bytes.into_iter().zip(byte_metas).enumerate() {
            self.buffer[addr + i] = (byte, bm);
        }
    }

    pub fn size(&self) -> usize {
        self.buffer.len()
    }

    fn grow_if_out_of_bounds(&mut self, index: usize, target_size: usize) {
        if self.buffer.len() <= index + target_size {
            self.buffer.resize_with(index + target_size, || (0u8, M::ShadowMetaByte::default()));
        }
    }

    fn assert_memory_bounds<Value>(&self, address: usize) {
        let size_of_t = core::mem::size_of::<Value>();
        debug_assert!(size_of_t <= 8, "value larger than 8 bytes");
        let eventual_pointer = address
            .checked_add(size_of_t)
            .expect("address computation overflow");
        debug_assert!(eventual_pointer <= self.buffer.len(), "address out of bounds");
    }

    fn memory_load<T: Copy>(&self, address: usize) -> T {
        self.assert_memory_bounds::<T>(address);
        let bytes: Vec<u8> = self.buffer[address..address + core::mem::size_of::<T>()]
            .iter()
            .map(|(b, _)| *b)
            .collect();
        unsafe { core::ptr::read_unaligned(bytes.as_ptr().cast::<T>()) }
    }

    // fn memory_store<T: Copy>(&mut self, address: usize, value: T) {
    //     self.assert_memory_bounds::<T>(address);
    //     let size = core::mem::size_of::<T>();
    //     let mut bytes = vec![0u8; size];
    //     unsafe {
    //         core::ptr::write_unaligned(bytes.as_mut_ptr().cast::<T>(), value);
    //     }
    //     for (i, byte) in bytes.into_iter().enumerate() {
    //         self.buffer[address + i].0 = byte;
    //     }
    // }

    fn memory_load_sub<StoreValue, Sub>(&self, address: usize) -> StoreValue
        where StoreValue: TryFrom<Sub>, StoreValue::Error: core::fmt::Debug, Sub: Copy
    {
        let loaded_value = self.memory_load::<Sub>(address);
        StoreValue::try_from(loaded_value).expect("conversion of sub-word load failed")
    }
}
