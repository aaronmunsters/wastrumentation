use crate::WasmValue;
use crate::{
    // load
    instrumented_base_load_f32,
    instrumented_base_load_f64,
    instrumented_base_load_i32,
    instrumented_base_load_i32_16S,
    instrumented_base_load_i32_16U,
    instrumented_base_load_i32_8S,
    instrumented_base_load_i32_8U,
    instrumented_base_load_i64,
    instrumented_base_load_i64_16S,
    instrumented_base_load_i64_16U,
    instrumented_base_load_i64_32S,
    instrumented_base_load_i64_32U,
    instrumented_base_load_i64_8S,
    instrumented_base_load_i64_8U,
};
use crate::{
    // store
    instrumented_base_store_f32,
    instrumented_base_store_f64,
    instrumented_base_store_i32,
    instrumented_base_store_i32_16,
    instrumented_base_store_i32_8,
    instrumented_base_store_i64,
    instrumented_base_store_i64_16,
    instrumented_base_store_i64_32,
    instrumented_base_store_i64_8,
};

macro_rules! generate_wrapper {
    ($name:ident wrapping $type:ident) => {
        #[derive(Debug)]
        pub struct $name(pub $type);

        impl $name {
            pub fn value(&self) -> &$type {
                let Self(value) = self;
                value
            }
        }
    };
}

generate_wrapper!(LoadOffset wrapping i64);
generate_wrapper!(StoreIndex wrapping i32);
generate_wrapper!(StoreOffset wrapping i64);
generate_wrapper!(LoadIndex wrapping i32);
generate_wrapper!(MemoryIndex wrapping i64);

impl MemoryIndex {
    pub fn grow(&self, amount: WasmValue) -> WasmValue {
        let amount = amount.as_i32();
        let index = i32::try_from(*self.value()).unwrap();
        (unsafe { crate::instrumented_memory_grow(amount, index) }).into()
    }
}

#[derive(Debug)]
pub enum StoreOperation {
    I32Store,
    I64Store,
    F32Store,
    F64Store,
    I32Store8,
    I32Store16,
    I64Store8,
    I64Store16,
    I64Store32,
}

#[derive(Debug)]
pub enum LoadOperation {
    I32Load,
    I64Load,
    F32Load,
    F64Load,
    I32Load8S,
    I32Load8U,
    I32Load16S,
    I32Load16U,
    I64Load8S,
    I64Load8U,
    I64Load16S,
    I64Load16U,
    I64Load32S,
    I64Load32U,
}

pub trait Deserialize {
    fn deserialize(s: &i32) -> Self;
}

impl Deserialize for StoreOperation {
    fn deserialize(s: &i32) -> Self {
        match s {
            1 => Self::I32Store,
            2 => Self::I64Store,
            3 => Self::F32Store,
            4 => Self::F64Store,
            5 => Self::I32Store8,
            6 => Self::I32Store16,
            7 => Self::I64Store8,
            8 => Self::I64Store16,
            9 => Self::I64Store32,
            _ => panic!("Deserialize for StoreOperation failed; unkown serialized value: {s}"),
        }
    }
}

impl StoreOperation {
    pub fn perform(&self, store_index: &StoreIndex, value: &WasmValue, offset: &StoreOffset) {
        // Regular
        use StoreOperation::{F32Store, F64Store, I32Store, I64Store};
        // I32 Load
        use StoreOperation::{I32Store16, I32Store8};
        // I64 Load
        use StoreOperation::{I64Store16, I64Store32, I64Store8};

        let ptr = *store_index.value();
        let offset = (*offset.value()).try_into().unwrap();

        match self {
            // Regular
            F32Store => unsafe { instrumented_base_store_f32(ptr, value.as_f32(), offset) },
            F64Store => unsafe { instrumented_base_store_f64(ptr, value.as_f64(), offset) },
            I32Store => unsafe { instrumented_base_store_i32(ptr, value.as_i32(), offset) },
            I64Store => unsafe { instrumented_base_store_i64(ptr, value.as_i64(), offset) },
            // I32 Load
            I32Store16 => unsafe { instrumented_base_store_i32_16(ptr, value.as_i32(), offset) },
            I32Store8 => unsafe { instrumented_base_store_i32_8(ptr, value.as_i32(), offset) },
            // I64 Load
            I64Store16 => unsafe { instrumented_base_store_i64_16(ptr, value.as_i64(), offset) },
            I64Store32 => unsafe { instrumented_base_store_i64_32(ptr, value.as_i64(), offset) },
            I64Store8 => unsafe { instrumented_base_store_i64_8(ptr, value.as_i64(), offset) },
        }
    }
}

impl LoadOperation {
    pub fn perform(&self, load_index: &LoadIndex, offset: &LoadOffset) -> WasmValue {
        // Regular
        use LoadOperation::{F32Load, F64Load, I32Load, I64Load};
        // I32 Load
        use LoadOperation::{I32Load16S, I32Load16U, I32Load8S, I32Load8U};
        // I64 Load
        use LoadOperation::{I64Load16S, I64Load16U, I64Load32S, I64Load32U, I64Load8S, I64Load8U};

        let ptr = *load_index.value();
        let offset = (*offset.value()).try_into().unwrap();

        match self {
            // Regular
            F32Load => unsafe { instrumented_base_load_f32(ptr, offset).into() },
            F64Load => unsafe { instrumented_base_load_f64(ptr, offset).into() },
            I32Load => unsafe { instrumented_base_load_i32(ptr, offset).into() },
            I64Load => unsafe { instrumented_base_load_i64(ptr, offset).into() },
            // I32 Load
            I32Load16S => unsafe { instrumented_base_load_i32_16S(ptr, offset).into() },
            I32Load16U => unsafe { instrumented_base_load_i32_16U(ptr, offset).into() },
            I32Load8S => unsafe { instrumented_base_load_i32_8S(ptr, offset).into() },
            I32Load8U => unsafe { instrumented_base_load_i32_8U(ptr, offset).into() },
            // I64 Load
            I64Load16S => unsafe { instrumented_base_load_i64_16S(ptr, offset).into() },
            I64Load16U => unsafe { instrumented_base_load_i64_16U(ptr, offset).into() },
            I64Load32S => unsafe { instrumented_base_load_i64_32S(ptr, offset).into() },
            I64Load32U => unsafe { instrumented_base_load_i64_32U(ptr, offset).into() },
            I64Load8S => unsafe { instrumented_base_load_i64_8S(ptr, offset).into() },
            I64Load8U => unsafe { instrumented_base_load_i64_8U(ptr, offset).into() },
        }
    }
}

impl Deserialize for LoadOperation {
    fn deserialize(s: &i32) -> Self {
        match s {
            1 => Self::I32Load,
            2 => Self::I64Load,
            3 => Self::F32Load,
            4 => Self::F64Load,
            5 => Self::I32Load8S,
            6 => Self::I32Load8U,
            7 => Self::I32Load16S,
            8 => Self::I32Load16U,
            9 => Self::I64Load8S,
            10 => Self::I64Load8U,
            11 => Self::I64Load16S,
            12 => Self::I64Load16U,
            13 => Self::I64Load32S,
            14 => Self::I64Load32U,
            _ => panic!("Deserialize for StoreOperation failed; unkown serialized value: {s}"),
        }
    }
}
