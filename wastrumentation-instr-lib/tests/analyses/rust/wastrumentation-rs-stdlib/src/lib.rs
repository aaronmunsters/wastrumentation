#![cfg_attr(not(feature = "std"), no_std)]
#![feature(asm_experimental_arch)]
#[cfg(not(feature = "std"))]
extern crate wee_alloc;
#[cfg(not(feature = "std"))]
#[global_allocator]
pub static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub mod unary;
pub use unary::UnaryOperator;

pub mod binary;
pub use binary::BinaryOperator;

mod memory;
pub use memory::{
    base_memory_size, Deserialize, LoadIndex, LoadOffset, LoadOperation, MemoryIndex, StoreIndex,
    StoreOffset, StoreOperation,
};

extern crate alloc;
use alloc::vec::Vec;
use core::mem::size_of;

// Optionally use primitives from core::arch::wasm
// https://doc.rust-lang.org/stable/core/arch/wasm/index.html
#[cfg(all(not(feature = "std"), target_arch = "wasm32"))]
#[panic_handler]
fn panic(_panic: &core::panic::PanicInfo<'_>) -> ! {
    core::arch::wasm32::unreachable()
}

#[link(wasm_import_module = "instrumented_input")]
extern "C" {
    // Base apply
    fn call_base(f_apply: i32, sigv: i32);
    // Base load
    fn instrumented_base_load_i32(ptr: i32, offset: i32) -> i32;
    fn instrumented_base_load_i64(ptr: i32, offset: i32) -> i64;
    fn instrumented_base_load_f32(ptr: i32, offset: i32) -> f32;
    fn instrumented_base_load_f64(ptr: i32, offset: i32) -> f64;
    fn instrumented_base_load_i32_8S(ptr: i32, offset: i32) -> i32;
    fn instrumented_base_load_i32_8U(ptr: i32, offset: i32) -> i32;
    fn instrumented_base_load_i32_16S(ptr: i32, offset: i32) -> i32;
    fn instrumented_base_load_i32_16U(ptr: i32, offset: i32) -> i32;
    fn instrumented_base_load_i64_8S(ptr: i32, offset: i32) -> i64;
    fn instrumented_base_load_i64_8U(ptr: i32, offset: i32) -> i64;
    fn instrumented_base_load_i64_16S(ptr: i32, offset: i32) -> i64;
    fn instrumented_base_load_i64_16U(ptr: i32, offset: i32) -> i64;
    fn instrumented_base_load_i64_32S(ptr: i32, offset: i32) -> i64;
    fn instrumented_base_load_i64_32U(ptr: i32, offset: i32) -> i64;
    // Base store
    fn instrumented_base_store_i32(ptr: i32, value: i32, offset: i32);
    fn instrumented_base_store_i64(ptr: i32, value: i64, offset: i32);
    fn instrumented_base_store_f32(ptr: i32, value: f32, offset: i32);
    fn instrumented_base_store_f64(ptr: i32, value: f64, offset: i32);
    fn instrumented_base_store_i32_8(ptr: i32, value: i32, offset: i32);
    fn instrumented_base_store_i32_16(ptr: i32, value: i32, offset: i32);
    fn instrumented_base_store_i64_8(ptr: i32, value: i64, offset: i32);
    fn instrumented_base_store_i64_16(ptr: i32, value: i64, offset: i32);
    fn instrumented_base_store_i64_32(ptr: i32, value: i64, offset: i32);
    // Base memory grow
    fn instrumented_memory_grow(amount: i32, idx: i32) -> i32;
    fn instrumented_memory_size(idx: i32) -> i32;
}

#[link(wasm_import_module = "wastrumentation_stack")]
extern "C" {
    fn wastrumentation_stack_load_type(ptr: i32, offset: i32) -> i32;
    fn wastrumentation_stack_load_i32(ptr: i32, offset: i32) -> i32;
    fn wastrumentation_stack_load_f32(ptr: i32, offset: i32) -> f32;
    fn wastrumentation_stack_load_i64(ptr: i32, offset: i32) -> i64;
    fn wastrumentation_stack_load_f64(ptr: i32, offset: i32) -> f64;
    fn wastrumentation_stack_store_i32(ptr: i32, value: i32, offset: i32);
    fn wastrumentation_stack_store_f32(ptr: i32, value: f32, offset: i32);
    fn wastrumentation_stack_store_i64(ptr: i32, value: i64, offset: i32);
    fn wastrumentation_stack_store_f64(ptr: i32, value: f64, offset: i32);
}

#[macro_export]
macro_rules! generate_wrapper {
    ($name:ident wrapping $type:ident accessed-using .$accessor:ident()) => {
        #[derive(Debug, PartialEq)]
        pub struct $name(pub $type);

        impl $name {
            pub fn $accessor(&self) -> $type {
                let Self(value) = self;
                *value
            }
        }
    };
}

/// Where an instruction trap was invoked
#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Location {
    instr_index: i64,
    funct_index: i64,
}

impl Location {
    pub fn new(instr_index: i64, funct_index: i64) -> Self {
        Self {
            instr_index,
            funct_index,
        }
    }
    pub fn instruction_index(&self) -> i64 {
        self.instr_index
    }

    pub fn function_index(&self) -> i64 {
        self.funct_index
    }
}

const TYPE_I32: i32 = 0;
const TYPE_F32: i32 = 1;
const TYPE_I64: i32 = 2;
const TYPE_F64: i32 = 3;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum WasmType {
    I32,
    F32,
    I64,
    F64,
}

// TODO: can I wrap this i32 into MaterializedWasmType to ensure type safety?
impl From<&i32> for WasmType {
    fn from(serialized_type: &i32) -> Self {
        match *serialized_type {
            TYPE_I32 => Self::I32,
            TYPE_F32 => Self::F32,
            TYPE_I64 => Self::I64,
            TYPE_F64 => Self::F64,
            _ => panic!(),
        }
    }
}

impl WasmType {
    pub fn size(&self) -> usize {
        match self {
            WasmType::I32 => size_of::<i32>(),
            WasmType::F32 => size_of::<f32>(),
            WasmType::I64 => size_of::<i64>(),
            WasmType::F64 => size_of::<f64>(),
        }
    }

    pub fn size_i32(&self) -> i32 {
        self.size().try_into().unwrap()
    }

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
}

#[derive(Debug, Clone)]
pub enum WasmValue {
    I32(i32),
    F32(f32),
    I64(i64),
    F64(f64),
}

impl PartialEq for WasmValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::I32(l), Self::I32(r)) => l == r,
            (Self::F32(l), Self::F32(r)) => l.to_be_bytes() == r.to_be_bytes(),
            (Self::I64(l), Self::I64(r)) => l == r,
            (Self::F64(l), Self::F64(r)) => l.to_be_bytes() == r.to_be_bytes(),
            _ => false,
        }
    }
}

impl From<i32> for WasmValue {
    fn from(value: i32) -> Self {
        WasmValue::I32(value)
    }
}

impl From<f32> for WasmValue {
    fn from(value: f32) -> Self {
        WasmValue::F32(value)
    }
}

impl From<i64> for WasmValue {
    fn from(value: i64) -> Self {
        WasmValue::I64(value)
    }
}

impl From<f64> for WasmValue {
    fn from(value: f64) -> Self {
        WasmValue::F64(value)
    }
}

impl WasmValue {
    fn store(&self, ptr: i32, offset: usize) {
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

    pub fn as_i32(&self) -> i32 {
        match self {
            Self::I32(v) => *v,
            _ => panic!("Attempt to convert {self:?} to i32"),
        }
    }

    pub fn as_f32(&self) -> f32 {
        match self {
            Self::F32(v) => *v,
            _ => panic!("Attempt to convert {self:?} to f32"),
        }
    }

    pub fn as_i64(&self) -> i64 {
        match self {
            Self::I64(v) => *v,
            _ => panic!("Attempt to convert {self:?} to i64"),
        }
    }

    pub fn as_f64(&self) -> f64 {
        match self {
            Self::F64(v) => *v,
            _ => panic!("Attempt to convert {self:?} to f64"),
        }
    }

    pub fn type_(&self) -> WasmType {
        match self {
            WasmValue::I32(_) => WasmType::I32,
            WasmValue::F32(_) => WasmType::F32,
            WasmValue::I64(_) => WasmType::I64,
            WasmValue::F64(_) => WasmType::F64,
        }
    }

    pub fn i32_from_bool(b: bool) -> Self {
        if b {
            Self::I32(1)
        } else {
            Self::I32(0)
        }
    }

    // Cfr. https://webassembly.github.io/spec/core/exec/runtime.html#default-val
    pub fn default_for(type_: &WasmType) -> Self {
        match type_ {
            WasmType::I32 => Self::I32(0),
            WasmType::I64 => Self::I64(0),
            WasmType::F32 => Self::F32(0.0),
            WasmType::F64 => Self::F64(0.0),
        }
    }

    #[cfg(feature = "std")]
    pub fn bytes_string(&self) -> String {
        let bytes = match self {
            WasmValue::I32(v) => v.to_le_bytes().to_vec(),
            WasmValue::F32(v) => v.to_le_bytes().to_vec(),
            WasmValue::I64(v) => v.to_le_bytes().to_vec(),
            WasmValue::F64(v) => v.to_le_bytes().to_vec(),
        }
        .into_iter()
        .map(|v| format!("{v}"))
        .collect::<Vec<String>>()
        .join(", ");
        let _ = format!("{type_:?}[{bytes}]", type_ = self.type_());
        format!("{self:?}")
    }

    #[must_use]
    pub fn as_wasm_bool(&self) -> bool {
        self.as_i32() != 0
    }
}

pub struct WasmFunction {
    pub f_apply: i32,
    pub instr_f_idx: i32,
    pub sigv: i32,
    pub code_present_serialized: i32,
}

#[cfg(feature = "std")]
impl std::fmt::Debug for WasmFunction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("WasmFunction")
            .field("uninstr_idx", &self.instr_f_idx)
            .field("sig_pointer", &self.sigv)
            .finish()
    }
}

#[derive(Debug)]
pub struct RuntimeValues {
    pub argc: i32,
    pub resc: i32,
    pub sigv: i32,
    pub signature_types: Vec<WasmType>,
}

pub const CODE_IS_PRESENT: i32 = 0;
pub const CODE_IS_IMPORT: i32 = 1;

impl WasmFunction {
    pub fn new(f_apply: i32, instr_f_idx: i32, sigv: i32, code_present_serialized: i32) -> Self {
        WasmFunction {
            f_apply,
            instr_f_idx,
            sigv,
            code_present_serialized,
        }
    }

    pub fn apply(&self) {
        unsafe { call_base(self.f_apply, self.sigv) };
    }

    pub fn instr_f_idx(&self) -> FunctionIndex {
        FunctionIndex(self.instr_f_idx)
    }

    pub fn is_imported(&self) -> bool {
        self.code_present_serialized == CODE_IS_IMPORT
    }

    pub fn is_present(&self) -> bool {
        self.code_present_serialized == CODE_IS_PRESENT
    }
}

pub type MutDynResults = RuntimeValues;
pub type MutDynArgs = RuntimeValues;

impl RuntimeValues {
    pub fn new(argc: i32, resc: i32, sigv: i32, sigtypv: i32) -> Self {
        let total_values = argc + resc;
        let mut signature_types: Vec<WasmType> = Vec::with_capacity(total_values as usize);

        for index in 0..total_values {
            let serialized_type = unsafe { wastrumentation_stack_load_type(sigtypv, index) };
            let wasm_type: WasmType = (&serialized_type).into();
            signature_types.push(wasm_type);
        }
        Self {
            argc,
            resc,
            sigv,
            signature_types,
        }
    }

    fn check_bounds(&self, count: i32, index: i32) {
        if index < 0 {
            panic!()
        };
        if index >= count {
            panic!()
        };
    }

    fn get_value(&self, index: i32) -> WasmValue {
        let index = index as usize;
        self.signature_types[index].load(self.sigv, index)
    }

    fn set_value(&mut self, index: i32, value: WasmValue) {
        let index = index as usize;
        value.store(self.sigv, index);
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

    pub fn set_arg(&mut self, index: i32, value: WasmValue) {
        self.check_bounds(self.argc, index);
        self.set_value(self.arg_base_offset() + index, value);
    }

    pub fn set_res(&mut self, index: i32, value: WasmValue) {
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

    pub fn args_iter(&self) -> RuntimeValuesIterator<'_> {
        RuntimeValuesIterator::new(self, RuntimeValuesIteratorTarget::Arguments)
    }

    pub fn ress_iter(&self) -> RuntimeValuesIterator<'_> {
        RuntimeValuesIterator::new(self, RuntimeValuesIteratorTarget::Results)
    }

    pub fn update_each_arg(&mut self, f: fn(i32, WasmValue) -> WasmValue) {
        for index in 0..self.argc {
            let value = self.get_arg(index);
            let updated_value = f(index, value);
            self.set_arg(index, updated_value);
        }
    }

    pub fn update_each_res(&mut self, f: fn(i32, WasmValue) -> WasmValue) {
        for index in 0..self.resc {
            let value = self.get_res(index);
            let updated_value = f(index, value);
            self.set_res(index, updated_value);
        }
    }
}

pub struct RuntimeValuesIterator<'a> {
    runtime_values: &'a RuntimeValues,
    state: i32,
    limit: i32,
}

enum RuntimeValuesIteratorTarget {
    Arguments,
    Results,
}

impl<'a> RuntimeValuesIterator<'a> {
    fn new(runtime_values: &'a RuntimeValues, target: RuntimeValuesIteratorTarget) -> Self {
        // ie. begin state
        let state = match target {
            RuntimeValuesIteratorTarget::Arguments => runtime_values.arg_base_offset(),
            RuntimeValuesIteratorTarget::Results => runtime_values.res_base_offset(),
        };

        let limit = match target {
            RuntimeValuesIteratorTarget::Arguments => state + runtime_values.argc,
            RuntimeValuesIteratorTarget::Results => state + runtime_values.resc,
        };

        Self {
            runtime_values,
            state,
            limit,
        }
    }
}

impl core::iter::Iterator for RuntimeValuesIterator<'_> {
    type Item = WasmValue;

    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            runtime_values,
            state,
            limit,
        } = self;
        let next = (state < limit).then(|| runtime_values.get_value(*state));
        *state += 1;
        next
    }
}

const THEN_KONTN: i32 = 1;
const ELSE_KONTN: i32 = 0;
const SKIP_KONTN: i32 = ELSE_KONTN;
const BRANCH_FAIL: i32 = 0;

pub trait SerializedContinuation
where
    Self: Sized,
{
    fn low_level_continuation(&self) -> &i32;
    fn with(continuation: i32) -> Self;

    fn is_then(&self) -> bool {
        let continuation = self.low_level_continuation();
        *continuation != BRANCH_FAIL
    }
    fn is_else(&self) -> bool {
        let continuation = self.low_level_continuation();
        *continuation == BRANCH_FAIL
    }

    fn continue_then() -> Self {
        Self::with(THEN_KONTN)
    }

    fn continue_skip() -> Self {
        Self::with(SKIP_KONTN)
    }

    fn continue_else() -> Self {
        Self::with(ELSE_KONTN)
    }
}

generate_wrapper!(FunctionIndex          wrapping i32 accessed-using .value());
generate_wrapper!(FunctionTableIndex     wrapping i32 accessed-using .value());
generate_wrapper!(FunctionTable          wrapping i32 accessed-using .value());
generate_wrapper!(PathContinuation       wrapping i32 accessed-using .value());
generate_wrapper!(IfThenElseInputCount   wrapping i32 accessed-using .value());
generate_wrapper!(IfThenElseArity        wrapping i32 accessed-using .value());
generate_wrapper!(IfThenInputCount       wrapping i32 accessed-using .value());
generate_wrapper!(IfThenArity            wrapping i32 accessed-using .value());
generate_wrapper!(ParameterBrIfCondition wrapping i32 accessed-using .value());
generate_wrapper!(BranchTargetLabel      wrapping i64 accessed-using .label());
generate_wrapper!(ParameterBrIfLabel     wrapping i32 accessed-using .label());
generate_wrapper!(BranchTableTarget      wrapping i32 accessed-using .target());
generate_wrapper!(BranchTableEffective   wrapping i32 accessed-using .label());
generate_wrapper!(BranchTableDefault     wrapping i32 accessed-using .value());
generate_wrapper!(LocalIndex             wrapping i64 accessed-using .value());
generate_wrapper!(BlockArity             wrapping i32 accessed-using .value());
generate_wrapper!(BlockInputCount        wrapping i32 accessed-using .value());
generate_wrapper!(LoopArity              wrapping i32 accessed-using .value());
generate_wrapper!(LoopInputCount         wrapping i32 accessed-using .value());
generate_wrapper!(GlobalIndex            wrapping i64 accessed-using .value());
generate_wrapper!(TableIndex             wrapping i32 accessed-using .value());
generate_wrapper!(ElementIndex           wrapping i32 accessed-using .value());

impl SerializedContinuation for PathContinuation {
    fn low_level_continuation(&self) -> &i32 {
        let Self(continuation) = self;
        continuation
    }

    fn with(continuation: i32) -> Self {
        Self(continuation)
    }
}

impl SerializedContinuation for ParameterBrIfCondition {
    fn low_level_continuation(&self) -> &i32 {
        let Self(continuation) = self;
        continuation
    }

    fn with(continuation: i32) -> Self {
        Self(continuation)
    }
}

#[derive(Debug)]
pub enum LocalOp {
    Get,
    Set,
    Tee,
}

#[derive(Debug)]
pub enum GlobalOp {
    Get,
    Set,
}

#[macro_export]
macro_rules! advice {
    (call pre
        (
            $func_ident: ident: FunctionIndex,
            $location_ident: ident: Location $(,)?) $body:block
    ) => {
        #[no_mangle]
        pub extern "C"
        fn specialized_call_pre (
            func_ident: i32,
            funct_index: i64,
            instr_index: i64,
        ) {
            let $func_ident = FunctionIndex(func_ident);
            let $location_ident = Location::new(funct_index, instr_index);
            $body
        }
    };
    (call post
        (
            $func_ident: ident: FunctionIndex,
            $location_ident: ident: Location $(,)?
        ) $body:block
    ) => {
        #[no_mangle]
        pub extern "C"
        fn specialized_call_post (
            func_ident: i32,
            funct_index: i64,
            instr_index: i64,
        ) {
            let $func_ident = FunctionIndex(func_ident);
            let $location_ident = Location::new(funct_index, instr_index);
            $body
        }
    };
    (call_indirect pre
        (
            $func_table_index_ident: ident: FunctionTableIndex,
            $func_table_ident: ident: FunctionTable,
            $location_ident: ident: Location $(,)?
        ) $body:block
    ) => {
        #[no_mangle]
        pub extern "C"
        fn specialized_call_indirect_pre (
            function_table_index: i32,
            function_table: i32,
            funct_index: i64,
            instr_index: i64,
        ) -> i32 {
            let $func_table_index_ident = FunctionTableIndex(function_table_index);
            let $func_table_ident = FunctionTable(function_table);
            let $location_ident = Location::new(funct_index, instr_index);
            let FunctionTableIndex(final_index) = $body;
            final_index
        }
    };
    (call_indirect post
        (
            $func_table_ident: ident: FunctionTable,
            $location_ident: ident: Location $(,)?
        ) $body:block
    ) => {
        #[no_mangle]
        pub extern "C"
        fn specialized_call_indirect_post (
            function_table: i32,
            funct_index: i64,
            instr_index: i64,
        ) {
            let $func_table_ident = FunctionTable(function_table);
            let $location_ident = Location::new(funct_index, instr_index);
            $body
        }
    };
    (apply
        (
            $func_ident: ident: WasmFunction,
            $args_ident: ident: MutDynArgs,
            $ress_ident: ident: MutDynResults $(,)?
        ) $body:block
    ) => {
        #[no_mangle]
        pub extern "C"
        fn generic_apply (
            f_apply: i32,
            instr_f_idx: i32,
            argc: i32,
            resc: i32,
            sigv: i32,
            sigtypv: i32,
            code_present_serialized: i32,
        ) {
            let $func_ident = WasmFunction::new(f_apply, instr_f_idx, sigv, code_present_serialized);
            let mut $args_ident = MutDynResults::new(argc, resc, sigv, sigtypv);
            let mut $ress_ident = MutDynArgs::new(argc, resc, sigv, sigtypv);
            $body
        }
    };
    (br
        (
            $target_label: ident: BranchTargetLabel,
            $location_ident: ident: Location $(,)?
        ) $body:block
    ) => {
        #[no_mangle]
        pub extern "C"
        fn specialized_br (
            low_level_label: i64,
            funct_index: i64,
            instr_index: i64,
        ) {
            let $target_label = BranchTargetLabel(low_level_label);
            let $location_ident = Location::new(funct_index, instr_index);
            $body
        }
    };
    (if_then_else
        (
            $path_continuation: ident: PathContinuation,
            $if_then_else_input_c: ident: IfThenElseInputCount,
            $if_then_else_arity: ident: IfThenElseArity,
            $location_ident: ident: Location $(,)?
        ) $body:block
    ) => {
        #[no_mangle]
        pub extern "C"
        fn specialized_if_then_else_k (
            path_continuation: i32,
            if_then_else_input_c: i32,
            if_then_else_arity: i32,
            funct_index: i64,
            instr_index: i64,
        ) -> i32 {
            let $path_continuation = PathContinuation(path_continuation);
            let $if_then_else_input_c = IfThenElseInputCount(if_then_else_input_c);
            let $if_then_else_arity = IfThenElseArity(if_then_else_arity);
            let $location_ident = Location::new(funct_index, instr_index);
            let PathContinuation(path_continuation) = $body;
            path_continuation
        }
    };
    (if_then
        (
            $path_continuation: ident: PathContinuation,
            $if_then_input_c: ident: IfThenInputCount,
            $if_then_arity: ident: IfThenArity,
            $location_ident: ident: Location $(,)?
        ) $body:block) => {
        #[no_mangle]
        pub extern "C"
        fn specialized_if_then_k (
            path_continuation: i32,
            if_then_input_c: i32,
            if_then_arity: i32,
            funct_index: i64,
            instr_index: i64,
        ) -> i32 {
            let $path_continuation = PathContinuation(path_continuation);
            let $if_then_input_c = IfThenInputCount(if_then_input_c);
            let $if_then_arity = IfThenArity(if_then_arity);
            let $location_ident = Location::new(funct_index, instr_index);
            let PathContinuation(path_continuation) = $body;
            path_continuation
        }
    };
    (if_then_else_post (
        $location_ident: ident: Location $(,)?
    ) $body:block) => {
        #[no_mangle]
        extern "C" fn trap_if_then_else_post(
            funct_index: i64,
            instr_index: i64,
        ) {
            let $location_ident = Location::new(funct_index, instr_index);
            $body
        }
    };
    (if_then_post (
        $location_ident: ident: Location $(,)?
    ) $body:block) => {
        #[no_mangle]
        extern "C" fn trap_if_then_post(
            funct_index: i64,
            instr_index: i64,
        ) {
            let $location_ident = Location::new(funct_index, instr_index);
            $body
        }
    };
    (br_if
        (
            $path_continuation: ident: ParameterBrIfCondition,
            $target_label: ident: ParameterBrIfLabel,
            $location_ident: ident: Location $(,)?
        ) $body:block
    ) => {
        #[no_mangle]
        pub extern "C"
        fn specialized_br_if (
            path_continuation: i32,
            low_level_label: i32,
            funct_index: i64,
            instr_index: i64,
        ) -> i32 {
            let $path_continuation = ParameterBrIfCondition(path_continuation);
            let $target_label = ParameterBrIfLabel(low_level_label);
            let $location_ident = Location::new(funct_index, instr_index);
            let ParameterBrIfCondition(path_continuation) = $body;
            path_continuation
        }
    };
    (br_table
        (
            $branch_table_target: ident: BranchTableTarget,
            $branch_table_effective: ident: BranchTableEffective,
            $branch_table_default: ident: BranchTableDefault,
            $location_ident: ident: Location $(,)?
        ) $body:block
    ) => {
        #[no_mangle]
        pub extern "C"
        fn specialized_br_table (
            br_table_target: i32,
            effective_label: i32,
            br_table_default: i32,
            funct_index: i64,
            instr_index: i64,
        ) -> i32 {
            let $branch_table_target = BranchTableTarget(br_table_target);
            let $branch_table_effective = BranchTableEffective(effective_label);
            let $branch_table_default = BranchTableDefault(br_table_default);
            let $location_ident = Location::new(funct_index, instr_index);
            let BranchTableTarget(br_table_target) = $body;
            br_table_target
        }
    };
    (select
        (
            $path_continuation: ident: PathContinuation,
            $location_ident: ident: Location $(,)?
        ) $body:block
     ) => {
        #[no_mangle]
        pub extern "C"
        fn specialized_select (
            path_continuation: i32,
            funct_index: i64,
            instr_index: i64,
        ) -> i32 {
            let $path_continuation = PathContinuation(path_continuation);
            let $location_ident = Location::new(funct_index, instr_index);
            let PathContinuation(path_continuation) = $body;
            path_continuation
        }
    };
    ///////////
    // UNARY //
    ///////////
    (unary
        (
            $operator: ident: UnaryOperator,
            $operand: ident: WasmValue,
            $location_ident: ident: Location $(,)?
        ) $body:block
    ) => {
        fn generic_unary_trap(
            operator: UnaryOperator,
            operand: WasmValue,
            location: Location,
        ) -> WasmValue {
            let $operator = operator;
            let $operand = operand;
            let $location_ident = location;
            $body
        }
        advice!(unary generic @genererate-specific generic_unary_trap);
    };
    (unary generic @genererate-specific $generic_unary_trap:ident) => {
        advice!(unary specific @for $generic_unary_trap unary_i32_to_i32 i32 I32 i32 I32);
        advice!(unary specific @for $generic_unary_trap unary_i64_to_i32 i64 I64 i32 I32);
        advice!(unary specific @for $generic_unary_trap unary_i64_to_i64 i64 I64 i64 I64);
        advice!(unary specific @for $generic_unary_trap unary_f32_to_f32 f32 F32 f32 F32);
        advice!(unary specific @for $generic_unary_trap unary_f64_to_f64 f64 F64 f64 F64);
        advice!(unary specific @for $generic_unary_trap unary_f32_to_i32 f32 F32 i32 I32);
        advice!(unary specific @for $generic_unary_trap unary_f64_to_i32 f64 F64 i32 I32);
        advice!(unary specific @for $generic_unary_trap unary_i32_to_i64 i32 I32 i64 I64);
        advice!(unary specific @for $generic_unary_trap unary_f32_to_i64 f32 F32 i64 I64);
        advice!(unary specific @for $generic_unary_trap unary_f64_to_i64 f64 F64 i64 I64);
        advice!(unary specific @for $generic_unary_trap unary_i32_to_f32 i32 I32 f32 F32);
        advice!(unary specific @for $generic_unary_trap unary_i64_to_f32 i64 I64 f32 F32);
        advice!(unary specific @for $generic_unary_trap unary_f64_to_f32 f64 F64 f32 F32);
        advice!(unary specific @for $generic_unary_trap unary_i32_to_f64 i32 I32 f64 F64);
        advice!(unary specific @for $generic_unary_trap unary_i64_to_f64 i64 I64 f64 F64);
        advice!(unary specific @for $generic_unary_trap unary_f32_to_f64 f32 F32 f64 F64);
    };
    (
        unary specific @for $generic_unary_trap:ident
        $function_name:ident
        $operand_type:ident
        $operand_type_wasm_value:ident
        $outcome_type:ident
        $outcome_type_wasm_value:ident
    ) => {
        #[no_mangle]
        extern "C" fn $function_name(
            operand: $operand_type,
            operator: i32,
            funct_index: i64,
            instr_index: i64,
        ) -> $outcome_type {
            let operator = UnaryOperator::from(operator);
            let operand = WasmValue::$operand_type_wasm_value(operand);
            let location = Location::new(funct_index, instr_index);
            let outcome = $generic_unary_trap(operator, operand, location);
            let WasmValue::$outcome_type_wasm_value(outcome) = outcome else {
                panic!(concat!("Attempted to convert {:?} to ", stringify!($outcome_type_wasm_value)), outcome);
            };
            outcome
        }
    };
    ///////////
    // BINARY //
    ///////////
    (binary
        (
            $operator: ident: BinaryOperator,
            $l: ident: WasmValue,
            $r: ident: WasmValue,
            $location_ident: ident: Location $(,)?
        ) $body:block
    ) => {
        fn generic_binary_trap(
            operator: BinaryOperator,
            l: WasmValue,
            r: WasmValue,
            location: Location,
        ) -> WasmValue {
            let $operator = operator;
            let $l = l;
            let $r = r;
            let $location_ident = location;
            $body
        }
        advice!(binary generic @genererate-specific generic_binary_trap);
    };
    (binary generic @genererate-specific $generic_binary_trap:ident) => {
        advice!(binary specific @for $generic_binary_trap binary_i32_i32_to_i32 i32 (I32) i32 (I32) => i32 (I32));
        advice!(binary specific @for $generic_binary_trap binary_i64_i64_to_i32 i64 (I64) i64 (I64) => i32 (I32));
        advice!(binary specific @for $generic_binary_trap binary_f32_f32_to_i32 f32 (F32) f32 (F32) => i32 (I32));
        advice!(binary specific @for $generic_binary_trap binary_f64_f64_to_i32 f64 (F64) f64 (F64) => i32 (I32));
        advice!(binary specific @for $generic_binary_trap binary_i64_i64_to_i64 i64 (I64) i64 (I64) => i64 (I64));
        advice!(binary specific @for $generic_binary_trap binary_f32_f32_to_f32 f32 (F32) f32 (F32) => f32 (F32));
        advice!(binary specific @for $generic_binary_trap binary_f64_f64_to_f64 f64 (F64) f64 (F64) => f64 (F64));
    };
    (
        binary specific @for $generic_binary_trap:ident
        $function_name:ident $l_type:ident ($l_type_wasm_value:ident)
                             $r_type:ident ($r_type_wasm_value:ident)
                             => $outcome_type:ident ($outcome_type_wasm_value:ident)
    ) => {
        #[no_mangle]
        extern "C" fn $function_name(
            l_op: $l_type,
            r_op: $r_type,
            operator: i32,
            funct_index: i64,
            instr_index: i64,
        ) -> $outcome_type {
            let operator = BinaryOperator::from(operator);
            let l_op = WasmValue::$l_type_wasm_value(l_op);
            let r_op = WasmValue::$r_type_wasm_value(r_op);
            let location = Location::new(funct_index, instr_index);
            let outcome = $generic_binary_trap(operator, l_op, r_op, location);
            let WasmValue::$outcome_type_wasm_value(outcome) = outcome else {
                panic!(concat!("Attempted to convert {:?} to ", stringify!($outcome_type_wasm_value)), outcome);
            };
            outcome
        }
    };
    (drop (
        $location_ident: ident: Location $(,)?
    ) $body:block) => {
        #[no_mangle]
        extern "C" fn drop_trap(
            funct_index: i64,
            instr_index: i64,
        ) {
            let $location_ident = Location::new(funct_index, instr_index);
            $body
        }
    };
    (return_ (
        $location_ident: ident: Location $(,)?
    ) $body:block) => {
        #[no_mangle]
        extern "C" fn return_trap(
            funct_index: i64,
            instr_index: i64,
        ) {
            let $location_ident = Location::new(funct_index, instr_index);
            $body
        }
    };
    (const_
        (
            $value: ident: WasmValue,
            $location_ident: ident: Location $(,)?
        ) $body:block
    ) => {
        fn generic_const_trap(
            value: WasmValue,
            location: Location,
        ) -> WasmValue {
            let $value = value;
            let $location_ident = location;
            $body
        }
        advice!(const_ generic @genererate-specific generic_const_trap);
    };
    (const_ generic @genererate-specific $generic_const_trap:ident) => {
        advice!(const_ specific @for $generic_const_trap trap_const_i32 i32 I32);
        advice!(const_ specific @for $generic_const_trap trap_const_f32 f32 F32);
        advice!(const_ specific @for $generic_const_trap trap_const_i64 i64 I64);
        advice!(const_ specific @for $generic_const_trap trap_const_f64 f64 F64);
    };
    (
        const_ specific @for $generic_const_trap:ident
        $function_name:ident
        $const_type:ident
        $const_type_wasm_value:ident
    ) => {
        #[no_mangle]
        extern "C" fn $function_name(
            const_: $const_type,
            funct_index: i64,
            instr_index: i64,
        ) -> $const_type {
            let const_ = WasmValue::$const_type_wasm_value(const_);
            let location = Location::new(funct_index, instr_index);
            let outcome = $generic_const_trap(const_, location);
            let WasmValue::$const_type_wasm_value(outcome) = outcome else {
                panic!(concat!("Attempted to convert {:?} to ", stringify!($const_type_wasm_value)), outcome);
            };
            outcome
        }
    };
    (local (
        $value: ident: WasmValue,
        $index: ident: LocalIndex,
        $local_op: ident: LocalOp,
        $location_ident: ident: Location $(,)?
    ) $body:block) => {
        fn generic_local_trap(
            value: WasmValue,
            index: LocalIndex,
            local_op: LocalOp,
            location: Location,
        ) -> WasmValue {
            let $value = value;
            let $index = index;
            let $local_op = local_op;
            let $location_ident = location;
            $body
        }
        advice!(local generic @genererate-specific generic_local_trap);
    };
    (local generic @genererate-specific $generic_local_trap:ident) => {
        advice!(local specific @for $generic_local_trap trap_local_get_i32 i32 I32 Get);
        advice!(local specific @for $generic_local_trap trap_local_set_i32 i32 I32 Set);
        advice!(local specific @for $generic_local_trap trap_local_tee_i32 i32 I32 Tee);
        advice!(local specific @for $generic_local_trap trap_local_get_f32 f32 F32 Get);
        advice!(local specific @for $generic_local_trap trap_local_set_f32 f32 F32 Set);
        advice!(local specific @for $generic_local_trap trap_local_tee_f32 f32 F32 Tee);
        advice!(local specific @for $generic_local_trap trap_local_get_i64 i64 I64 Get);
        advice!(local specific @for $generic_local_trap trap_local_set_i64 i64 I64 Set);
        advice!(local specific @for $generic_local_trap trap_local_tee_i64 i64 I64 Tee);
        advice!(local specific @for $generic_local_trap trap_local_get_f64 f64 F64 Get);
        advice!(local specific @for $generic_local_trap trap_local_set_f64 f64 F64 Set);
        advice!(local specific @for $generic_local_trap trap_local_tee_f64 f64 F64 Tee);
    };
    (
        local specific @for $generic_local_trap:ident
        $function_name:ident
        $value_type:ident
        $value_type_wasm_value:ident
        $op:ident
    ) => {
        #[no_mangle]
        extern "C" fn $function_name(
            operand: $value_type,
            index: i64,
            funct_index: i64,
            instr_index: i64,
        ) -> $value_type {
            let operand = WasmValue::$value_type_wasm_value(operand);
            let index = LocalIndex(index);
            let local_op = LocalOp::$op;
            let location = Location::new(funct_index, instr_index);
            let outcome = $generic_local_trap(operand, index, local_op, location);
            let WasmValue::$value_type_wasm_value(outcome) = outcome else {
                panic!(concat!("Attempted to convert {:?} to ", stringify!($value_type_wasm_value)), outcome);
            };
            outcome
        }
    };
    (global (
        $value: ident: WasmValue,
        $index: ident: GlobalIndex,
        $global_op: ident: GlobalOp,
        $location_ident: ident: Location $(,)?
    ) $body:block) => {
        fn generic_global_trap(
            value: WasmValue,
            index: GlobalIndex,
            global_op: GlobalOp,
            location: Location,
        ) -> WasmValue {
            let $value = value;
            let $index = index;
            let $global_op = global_op;
            let $location_ident = location;
            $body
        }
        advice!(global generic @genererate-specific generic_global_trap);
    };
    (global generic @genererate-specific $generic_global_trap:ident) => {
        advice!(global specific @for $generic_global_trap trap_global_get_i32 i32 I32 Get);
        advice!(global specific @for $generic_global_trap trap_global_set_i32 i32 I32 Set);
        advice!(global specific @for $generic_global_trap trap_global_get_f32 f32 F32 Get);
        advice!(global specific @for $generic_global_trap trap_global_set_f32 f32 F32 Set);
        advice!(global specific @for $generic_global_trap trap_global_get_i64 i64 I64 Get);
        advice!(global specific @for $generic_global_trap trap_global_set_i64 i64 I64 Set);
        advice!(global specific @for $generic_global_trap trap_global_get_f64 f64 F64 Get);
        advice!(global specific @for $generic_global_trap trap_global_set_f64 f64 F64 Set);
    };
    (
        global specific @for $generic_global_trap:ident
        $function_name:ident
        $value_type:ident
        $value_type_wasm_value:ident
        $op:ident) => {
        #[no_mangle]
        extern "C" fn $function_name(
            operand: $value_type,
            index: i64,
            funct_index: i64,
            instr_index: i64,
        ) -> $value_type {
            let operand = WasmValue::$value_type_wasm_value(operand);
            let index = GlobalIndex(index);
            let global_op = GlobalOp::$op;
            let location = Location::new(funct_index, instr_index);
            let outcome = $generic_global_trap(operand, index, global_op, location);
            let WasmValue::$value_type_wasm_value(outcome) = outcome else {
                panic!(concat!("Attempted to convert {:?} to ", stringify!($value_type_wasm_value)), outcome);
            };
            outcome
        }
    };
    // LOAD
    (load (
        $load_index: ident: LoadIndex,
        $offset: ident: LoadOffset,
        $operation: ident: LoadOperation,
        $location_ident: ident: Location $(,)?
    ) $body:block) => {
        fn generic_load_trap(
            load_index: LoadIndex,
            offset: LoadOffset,
            operation: LoadOperation,
            location: Location,
        ) -> WasmValue {
            let $load_index = load_index;
            let $offset = offset;
            let $operation = operation;
            let $location_ident = location;
            $body
        }
        advice!(load generic @genererate-specific generic_load_trap);
    };
    (load generic @genererate-specific $generic_load_trap:ident) => {
        advice!(load specific @for $generic_load_trap trap_f32_load f32 F32);
        advice!(load specific @for $generic_load_trap trap_f64_load f64 F64);
        advice!(load specific @for $generic_load_trap trap_i32_load i32 I32);
        advice!(load specific @for $generic_load_trap trap_i64_load i64 I64);
    };
    (
        load specific @for $generic_load_trap:ident
        $function_name:ident
        $load_type:ident
        $load_type_wasm_value:ident) => {
        #[no_mangle]
        extern "C" fn $function_name(
            load_idx: i32,
            offset: i64,
            operation: i32,
            funct_index: i64,
            instr_index: i64,
        ) -> $load_type {
            let load_index = LoadIndex(load_idx);
            let offset = LoadOffset(offset);
            let operation = LoadOperation::deserialize(&operation);
            let location = Location::new(funct_index, instr_index);
            let outcome = $generic_load_trap(load_index, offset, operation, location);
            let WasmValue::$load_type_wasm_value(outcome) = outcome else {
                panic!(concat!("Attempted to convert {:?} to ", stringify!($value_type_wasm_value)), outcome);
            };
            outcome
        }
    };
    // STORE
    (store (
        $store_index: ident: StoreIndex,
        $value: ident: WasmValue,
        $offset: ident: StoreOffset,
        $operation: ident: StoreOperation,
        $location_ident: ident: Location $(,)?
    ) $body:block) => {
        fn generic_store_trap(
            store_index: StoreIndex,
            value: WasmValue,
            offset: StoreOffset,
            operation: StoreOperation,
            location: Location,
        ) {
            let $store_index = store_index;
            let $value = value;
            let $offset = offset;
            let $operation = operation;
            let $location_ident = location;
            $body
        }
        advice!(store generic @genererate-specific generic_store_trap);
    };
    (store generic @genererate-specific $generic_store_trap:ident) => {
        advice!(store specific @for $generic_store_trap trap_f32_store f32 F32);
        advice!(store specific @for $generic_store_trap trap_f64_store f64 F64);
        advice!(store specific @for $generic_store_trap trap_i32_store i32 I32);
        advice!(store specific @for $generic_store_trap trap_i64_store i64 I64);
    };
    (
        store specific @for $generic_store_trap:ident
        $function_name:ident
        $store_type:ident
        $store_type_wasm_value:ident
    ) => {
        #[no_mangle]
        extern "C" fn $function_name(
            store_idx: i32,
            value: $store_type,
            offset: i64,
            operation: i32,
            funct_index: i64,
            instr_index: i64,
        ) {
            let store_index = StoreIndex(store_idx);
            let value = WasmValue::$store_type_wasm_value(value);
            let offset = StoreOffset(offset);
            let operation = StoreOperation::deserialize(&operation);
            let location = Location::new(funct_index, instr_index);
            $generic_store_trap(store_index, value, offset, operation, location);
        }
    };
    (memory_size
        (
            $size: ident: WasmValue,
            $index: ident: MemoryIndex,
            $location_ident: ident: Location $(,)?
        )
        $body:block
    ) => {
        #[no_mangle]
        extern "C" fn trap_memory_size(
            size: i32,
            idx: i64,
            funct_index: i64,
            instr_index: i64,
        ) -> i32 {
            let $size = WasmValue::I32(size);
            let $index = MemoryIndex(idx);
            let $location_ident = Location::new(funct_index, instr_index);
            let size: WasmValue = $body;
            size.as_i32()
        }
    };
    (memory_grow
        (
            $amount: ident: WasmValue,
            $index: ident: MemoryIndex,
            $location_ident: ident: Location $(,)?
        )
        $body:block
    ) => {
        #[no_mangle]
        extern "C" fn trap_memory_grow(
            amount: i32,
            idx: i64,
            funct_index: i64,
            instr_index: i64,
        ) -> i32 {
            let $amount = WasmValue::I32(amount);
            let $index = MemoryIndex(idx);
            let $location_ident = Location::new(funct_index, instr_index);
            let delta_or_neg_1: WasmValue = $body;
            delta_or_neg_1.as_i32()
        }
    };
    (block pre (
        $block_input_c: ident: BlockInputCount,
        $block_arity: ident: BlockArity,
        $location_ident: ident: Location $(,)?
    ) $body:block) => {
        #[no_mangle]
        extern "C" fn trap_block_pre(
            block_input_c: i32,
            block_arity: i32,
            funct_index: i64,
            instr_index: i64,
        ) {
            let $block_input_c = BlockInputCount(block_input_c);
            let $block_arity = BlockArity(block_arity);
            let $location_ident = Location::new(funct_index, instr_index);
            $body;
        }
    };
    (block post (
        $location_ident: ident: Location $(,)?
    ) $body:block) => {
        #[no_mangle]
        extern "C" fn trap_block_post(
            funct_index: i64,
            instr_index: i64,
        ) {
            let $location_ident = Location::new(funct_index, instr_index);
            $body
        }
    };
    (loop_ pre (
        $loop_input_c: ident: LoopInputCount,
        $loop_arity: ident: LoopArity,
        $location_ident: ident: Location $(,)?
    ) $body:block) => {
        #[no_mangle]
        extern "C" fn trap_loop_pre(
            loop_input_c: i32,
            loop_arity: i32,
            funct_index: i64,
            instr_index: i64,
        ) {
            let $loop_input_c = LoopInputCount(loop_input_c);
            let $loop_arity = LoopArity(loop_arity);
            let $location_ident = Location::new(funct_index, instr_index);
            $body;
        }
    };
    (loop_ post (
        $location_ident: ident: Location $(,)?
    ) $body:block) => {
        #[no_mangle]
        extern "C" fn trap_loop_post(
            funct_index: i64,
            instr_index: i64,
        ) {
            let $location_ident = Location::new(funct_index, instr_index);
            $body
        }
    };
    /////////////////////
    // REFERENCE TYPES //
    /////////////////////
    (ref_func (
        $func_index: ident: WasmValue,
        $location_ident: ident: Location $(,)?
    ) $body:block) => {
        #[no_mangle]
        extern "C" fn trap_ref_func(
            func_index: i32,
            funct_index: i64,
            instr_index: i64,
        ) {
            let $func_index = func_index;
            let $location_ident = Location::new(funct_index, instr_index);
            $body;
        }
    };
    (ref_null (
        $location_ident: ident: Location $(,)?
    ) $body:block) => {
        #[no_mangle]
        extern "C" fn trap_ref_null(
            funct_index: i64,
            instr_index: i64,
        ) {
            let $location_ident = Location::new(funct_index, instr_index);
            $body;
        }
    };
    (ref_is_null (
        $res: ident: WasmValue,
        $location_ident: ident: Location $(,)?
    ) $body:block) => {
        #[no_mangle]
        extern "C" fn trap_ref_is_null(
            res: i32,
            funct_index: i64,
            instr_index: i64,
        ) -> i32 {
            let $res = WasmValue::I32(res);
            let $location_ident = Location::new(funct_index, instr_index);
            let new_res: WasmValue = $body;
            new_res.as_i32()
        }
    };
    ////////////
    // TABLES //
    ////////////
    (table_get (
        $element_index: ident: WasmValue,
        $table_index: ident: FunctionTableIndex,
        $location_ident: ident: Location $(,)?
    ) $body:block) => {
        #[no_mangle]
        extern "C" fn trap_table_get(
            element_index: i32,
            table_index: i32,
            funct_index: i64,
            instr_index: i64,
        ) -> i32 {
            let $table_index = FunctionTableIndex(table_index);
            let $element_index = element_index;
            let $location_ident = Location::new(funct_index, instr_index);
            let new_element_index: i32 = $body;
            new_element_index
        }
    };
    (table_set (
        $element_index: ident: WasmValue,
        $table_index: ident: FunctionTableIndex,
        $location_ident: ident: Location $(,)?
    ) $body:block) => {
        #[no_mangle]
        extern "C" fn trap_table_set(
            element_index: i32,
            table_index: i32,
            funct_index: i64,
            instr_index: i64,
        ) -> i32 {
            let $table_index = FunctionTableIndex(table_index);
            let $element_index = element_index;
            let $location_ident = Location::new(funct_index, instr_index);
            let new_element_index: i32 = $body;
            new_element_index
        }
    };
    (table_size (
        $table_size: ident: WasmValue,
        $table_index: ident: FunctionTableIndex,
        $location_ident: ident: Location $(,)?
    ) $body:block) => {
        #[no_mangle]
        extern "C" fn trap_table_size(
            table_size: i32,
            table_index: i32,
            funct_index: i64,
            instr_index: i64,
        ) -> i32 {
            let $table_size = table_size;
            let $table_index = FunctionTableIndex(table_index);
            let $location_ident = Location::new(funct_index, instr_index);
            let new_table_size: i32 = $body;
            new_table_size
        }
    };
    (table_grow (
        $grow_size: ident: WasmValue,
        $table_index: ident: FunctionTableIndex,
        $location_ident: ident: Location $(,)?
    ) $body:block) => {
        #[no_mangle]
        extern "C" fn trap_table_grow(
            grow_size: i32,
            table_index: i32,
            funct_index: i64,
            instr_index: i64,
        ) -> i32 {
            let $grow_size = grow_size;
            let $table_index = FunctionTableIndex(table_index);
            let $location_ident = Location::new(funct_index, instr_index);
            let new_grow_size: i32 = $body;
            new_grow_size
        }
    };
    (table_fill (
        $index: ident: WasmValue,
        $fill_size: ident: WasmValue,
        $table_index: ident: FunctionTableIndex,
        $location_ident: ident: Location $(,)?
    ) $body:block) => {
        #[no_mangle]
        extern "C" fn trap_table_fill(
            index: i32,
            fill_size: i32,
            table_index: i32,
            funct_index: i64,
            instr_index: i64,
        ) -> i32 {
            let $table_index = FunctionTableIndex(table_index);
            let $index = index;
            let $fill_size = fill_size;
            let $location_ident = Location::new(funct_index, instr_index);
            let new_index: i32 = $body;
            new_index
        }
    };
    (table_copy (
        $dst_element_index: ident: WasmValue,
        $src_element_index: ident: WasmValue,
        $copy_size: ident: WasmValue,
        $dst_table_index: ident: FunctionTableIndex,
        $src_table_index: ident: FunctionTableIndex,
        $location_ident: ident: Location $(,)?
    ) $body:block) => {

        static mut RETURN_DST: i32 = 0;
        static mut RETURN_SRC: i32 = 0;
        static mut RETURN_SIZE: i32 = 0;

        #[no_mangle]
        extern "C" fn trap_table_copy(
            dst_element_index: i32,
            src_element_index: i32,
            copy_size: i32,
            dst_table_index: i32,
            src_table_index: i32,
            funct_index: i64,
            instr_index: i64,
        ) {
            let $dst_table_index = FunctionTableIndex(dst_table_index);
            let $src_table_index = FunctionTableIndex(src_table_index);
            let $dst_element_index = dst_element_index;
            let $src_element_index = src_element_index;
            let $copy_size = copy_size;
            let $location_ident = Location::new(funct_index, instr_index);
            let (new_dst, new_src, new_size): (i32, i32, i32) = $body;
            unsafe {
                RETURN_DST = new_dst;
                RETURN_SRC = new_src;
                RETURN_SIZE = new_size;
            }
        }

        #[no_mangle]
        extern "C" fn trap_table_copy_get_dst() -> i32 {
            unsafe { RETURN_DST }
        }

        #[no_mangle]
        extern "C" fn trap_table_copy_get_src() -> i32 {
            unsafe { RETURN_SRC }
        }

        #[no_mangle]
        extern "C" fn trap_table_copy_get_size() -> i32 {
            unsafe { RETURN_SIZE }
        }
    };
    (table_init (
        $destination_table_offset: ident: WasmValue,
        $source_element_offset: ident: WasmValue,
        $init_size: ident: WasmValue,
        $table_index: ident: FunctionTableIndex,
        $element_index: ident: ElementIndex,
        $location_ident: ident: Location $(,)?
    ) $body:block) => {
        static mut RETURN_DESTINATION_TABLE_OFFSET: i32 = 0;
        static mut RETURN_SOURCE_ELEMENT_OFFSET: i32 = 0;
        static mut RETURN_INIT_SIZE: i32 = 0;
        #[no_mangle]
        extern "C" fn trap_table_init(
            destination_table_offset: i32,
            source_element_offset: i32,
            init_size: i32,
            table_index: i32,
            element_index: i32,
            funct_index: i64,
            instr_index: i64,
        ) {
            let $table_index = FunctionTableIndex(table_index);
            let $element_index = element_index;
            let $destination_table_offset = destination_table_offset;
            let $source_element_offset = source_element_offset;
            let $init_size = init_size;
            let $location_ident = Location::new(funct_index, instr_index);
            let (new_destination_table_offset, new_source_element_offset, new_init_size): (i32, i32, i32) = $body;
            unsafe {
                RETURN_DESTINATION_TABLE_OFFSET = new_destination_table_offset;
                RETURN_SOURCE_ELEMENT_OFFSET = new_source_element_offset;
                RETURN_INIT_SIZE = new_init_size;
            }
        }

        #[no_mangle]
        extern "C" fn trap_table_init_get_destination_table_offset() -> i32 {
            unsafe { RETURN_DESTINATION_TABLE_OFFSET }
        }

        #[no_mangle]
        extern "C" fn trap_table_init_get_source_element_offset() -> i32 {
            unsafe { RETURN_SOURCE_ELEMENT_OFFSET }
        }

        #[no_mangle]
        extern "C" fn trap_table_init_get_size() -> i32 {
            unsafe { RETURN_INIT_SIZE }
        }
    };
    (elem_drop (
        $element_index: ident: ElementIndex,
        $location_ident: ident: Location $(,)?
    ) $body:block) => {
        #[no_mangle]
        extern "C" fn trap_elem_drop(
            element_index: i32,
            funct_index: i64,
            instr_index: i64,
        ) {
            let $element_index = ElementIndex(element_index);
            let $location_ident = Location::new(funct_index, instr_index);
            $body;
        }
    };

    // General pattern to allow multiple advices in a single `advice! {...}`
    ($(
        $($advice_keyword:ident)+ ($($formal_arg:ident : $formal_type:ident),* $(,)?) $body:block
    )+) => {
        $(advice! { $($advice_keyword)+ ($($formal_arg:$formal_type),*) $body })+
    };
}
