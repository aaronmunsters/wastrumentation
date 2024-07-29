#![no_std]

extern crate wee_alloc;

#[global_allocator]
pub static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Optionally use primitives from core::arch::wasm
// https://doc.rust-lang.org/stable/core/arch/wasm/index.html

#[cfg(not(test))]
#[cfg(target_arch = "wasm32")]
#[panic_handler]
fn panic(_panic: &core::panic::PanicInfo<'_>) -> ! {
    core::arch::wasm32::unreachable()
}

#[link(wasm_import_module = "instrumented_input")]
extern "C" {
    fn call_base(f_apply: i32, sigv: i32) -> ();
}

#[link(wasm_import_module = "wastrumentation_stack")]
extern "C" {
    fn wastrumentation_stack_load_i32(ptr: i32, offset: i32) -> i32;
    fn wastrumentation_stack_load_f32(ptr: i32, offset: i32) -> f32;
    fn wastrumentation_stack_load_i64(ptr: i32, offset: i32) -> i64;
    fn wastrumentation_stack_load_f64(ptr: i32, offset: i32) -> f64;
    fn wastrumentation_stack_store_i32(ptr: i32, value: i32, offset: i32) -> ();
    fn wastrumentation_stack_store_f32(ptr: i32, value: f32, offset: i32) -> ();
    fn wastrumentation_stack_store_i64(ptr: i32, value: i64, offset: i32) -> ();
    fn wastrumentation_stack_store_f64(ptr: i32, value: f64, offset: i32) -> ();
}

const TYPE_I32: i32 = 0;
const TYPE_F32: i32 = 1;
const TYPE_I64: i32 = 2;
const TYPE_F64: i32 = 3;

pub enum WasmType {
    I32,
    F32,
    I64,
    F64,
}

impl WasmType {
    fn size(&self) -> usize {
        match self {
            WasmType::I32 => size_of::<i32>(),
            WasmType::F32 => size_of::<f32>(),
            WasmType::I64 => size_of::<i64>(),
            WasmType::F64 => size_of::<f64>(),
        }
    }
}

impl WasmType {
    fn load(&self, ptr: i32, offset: usize) -> WasmValue {
        let offset = offset as i32;
        match self {
            WasmType::I32 => unsafe {
                let res = wastrumentation_stack_load_i32(ptr, offset);
                WasmValue::I32(res)
            },
            WasmType::F32 => unsafe {
                let res = wastrumentation_stack_load_f32(ptr, offset);
                WasmValue::F32(res)
            },
            WasmType::I64 => unsafe {
                let res = wastrumentation_stack_load_i64(ptr, offset);
                WasmValue::I64(res)
            },
            WasmType::F64 => unsafe {
                let res = wastrumentation_stack_load_f64(ptr, offset);
                WasmValue::F64(res)
            },
        }
    }

    fn from(serialized_type: i32) -> Self {
        match serialized_type {
            TYPE_I32 => Self::I32,
            TYPE_F32 => Self::F32,
            TYPE_I64 => Self::I64,
            TYPE_F64 => Self::F64,
            _ => panic!(),
        }
    }
}

pub enum WasmValue {
    I32(i32),
    F32(f32),
    I64(i64),
    F64(f64),
}

impl WasmValue {
    fn store(&self, ptr: i32, offset: usize) -> () {
        let offset = offset as i32;
        match self {
            WasmValue::I32(value) => unsafe {
                wastrumentation_stack_store_i32(ptr, *value, offset)
            },
            WasmValue::F32(value) => unsafe {
                wastrumentation_stack_store_f32(ptr, *value, offset)
            },
            WasmValue::I64(value) => unsafe {
                wastrumentation_stack_store_i64(ptr, *value, offset)
            },
            WasmValue::F64(value) => unsafe {
                wastrumentation_stack_store_f64(ptr, *value, offset)
            },
        }
    }
}

pub type FunctionIndex = i32; // TODO: turn into wrapper type?
pub struct WasmFunction {
    f_apply: i32,
    sigv: i32,
}

pub struct FunctionTableIndex(pub i32);
pub struct FunctionTable(pub i32);

pub struct RuntimeValues {
    argc: i32,
    resc: i32,
    sigv: i32,
    signature_types: Vec<WasmType>,
    signature_offsets: Vec<usize>,
}

impl WasmFunction {
    pub fn new(f_apply: i32, sigv: i32) -> Self {
        WasmFunction { f_apply, sigv }
    }

    pub fn apply(&self) -> () {
        unsafe { call_base(self.f_apply, self.sigv) };
    }
}

pub type MutDynResults = RuntimeValues;
pub type MutDynArgs = RuntimeValues;

extern crate alloc;
use alloc::vec::Vec;

impl RuntimeValues {
    pub fn new(argc: i32, resc: i32, sigv: i32, sigtypv: i32) -> Self {
        let total_values = usize::try_from(argc + resc).unwrap();
        let signature_type_slice = unsafe { from_raw_parts(sigtypv as *const i32, total_values) };
        let mut signature_types: Vec<WasmType> = Vec::with_capacity(total_values);
        let mut signature_offsets: Vec<usize> = Vec::with_capacity(total_values);
        let mut offset_so_far = 0;
        for serialized_type in signature_type_slice.iter() {
            let wasm_type = WasmType::from(*serialized_type);
            signature_offsets.push(offset_so_far);
            offset_so_far += wasm_type.size();
            signature_types.push(wasm_type);
        }
        Self {
            argc,
            resc,
            sigv,
            signature_types,
            signature_offsets,
        }
    }

    fn check_bounds(&self, count: i32, index: i32) -> () {
        if !(index >= 0) {
            panic!()
        };
        if index >= count {
            panic!()
        };
    }

    fn get_value(&self, index: i32) -> WasmValue {
        let index = index as usize;
        self.signature_types[index as usize].load(self.sigv, self.signature_offsets[index])
    }

    fn set_value(&mut self, index: i32, value: WasmValue) -> () {
        let index = index as usize;
        value.store(self.sigv, self.signature_offsets[index]);
    }

    fn arg_base_offset(&self) -> i32 {
        self.resc
    }

    fn res_base_offset(&self) -> i32 {
        0
    }

    pub fn get_arg(&self, index: i32) -> WasmValue {
        self.check_bounds(self.argc, index);
        self.get_value(self.arg_base_offset() + index)
    }

    pub fn get_res(&self, index: i32) -> WasmValue {
        self.check_bounds(self.resc, index);
        self.get_value(self.res_base_offset() + index)
    }

    pub fn set_arg(&mut self, index: i32, value: WasmValue) -> () {
        self.check_bounds(self.argc, index);
        self.set_value(self.arg_base_offset() + index, value);
    }

    pub fn set_res(&mut self, index: i32, value: WasmValue) -> () {
        self.check_bounds(self.resc, index);
        self.set_value(self.res_base_offset() + index, value);
    }

    pub fn get_arg_type(&self, index: i32) -> &WasmType {
        self.check_bounds(self.argc, index);
        &self.signature_types[(self.arg_base_offset() + index) as usize]
    }

    pub fn get_res_type(&self, index: i32) -> &WasmType {
        self.check_bounds(self.resc, index);
        &self.signature_types[(self.res_base_offset() + index) as usize]
    }
}

#[macro_export]
macro_rules! advice {
    (advice call before
        ($func_ident: ident : FunctionIndex) $body:block
    ) => {
        #[no_mangle]
        pub extern "C"
        fn specialized_call_pre ($func_ident:FunctionIndex) -> ()
        $body
    };
    (advice call after
        ($func_ident: ident : FunctionIndex) $body:block
    ) => {
        #[no_mangle]
        pub extern "C"
        fn specialized_call_post ($func_ident:FunctionIndex) -> ()
        $body
    };
    (advice call-indirect before
        ($func_table_index_ident: ident : FunctionTableIndex, $func_table_ident: ident : FunctionTable) $body:block
    ) => {
        #[no_mangle]
        pub extern "C"
        fn specialized_call_indirect_pre (
            function_table_index: i32,
            function_table: i32,
        ) -> i32 {
            let $func_table_index_ident = FunctionTableIndex(function_table_index);
            let $func_table_ident = FunctionTable(function_table);
            let FunctionTableIndex(final_index) = $body;
            final_index
        }
    };
    (advice call-indirect after
        ($func_table_ident: ident : FunctionTable) $body:block
    ) => {
        #[no_mangle]
        pub extern "C"
        fn specialized_call_indirect_post (
            function_table: i32,
        ) -> () {
            let $func_table_ident = FunctionTable(function_table);
            $body
        }
    };
    (advice apply
        (
            $func_ident: ident : WasmFunction,
            $args_ident: ident : MutDynArgs,
            $ress_ident: ident : MutDynResults
        ) $body:block
    ) => {
        #[no_mangle]
        pub extern "C"
        fn generic_apply (f_apply: i32, argc: i32, resc: i32, sigv: i32, sigtypv: i32) -> () {
            let $func_ident = WasmFunction::new(f_apply, sigv);
            let $args_ident = MutDynResults::new(argc, resc, sigv, sigtypv);
            let $ress_ident = MutDynArgs::new(argc, resc, sigv, sigtypv);
            $body
        }
    };
}

use core::{mem::size_of, slice::from_raw_parts};
