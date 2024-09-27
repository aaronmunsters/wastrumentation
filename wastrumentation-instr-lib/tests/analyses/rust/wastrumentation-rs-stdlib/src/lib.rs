#![cfg_attr(not(feature = "std"), no_std)]
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
    Deserialize, LoadIndex, LoadOffset, LoadOperation, MemoryIndex, StoreIndex, StoreOffset,
    StoreOperation,
};

// Optionally use primitives from core::arch::wasm
// https://doc.rust-lang.org/stable/core/arch/wasm/index.html
#[cfg(not(feature = "std"))]
#[cfg(not(test))]
#[cfg(target_arch = "wasm32")]
#[panic_handler]
fn panic(_panic: &core::panic::PanicInfo<'_>) -> ! {
    core::arch::wasm32::unreachable()
}

#[link(wasm_import_module = "instrumented_input")]
extern "C" {
    fn call_base(f_apply: i32, sigv: i32) -> ();
    fn instrumented_base_load_i32(ptr: i32, offset: i32) -> i32;
    fn instrumented_base_load_f32(ptr: i32, offset: i32) -> f32;
    fn instrumented_base_load_i64(ptr: i32, offset: i32) -> i64;
    fn instrumented_base_load_f64(ptr: i32, offset: i32) -> f64;
    fn instrumented_base_store_i32(ptr: i32, value: i32, offset: i32) -> ();
    fn instrumented_base_store_f32(ptr: i32, value: f32, offset: i32) -> ();
    fn instrumented_base_store_i64(ptr: i32, value: i64, offset: i32) -> ();
    fn instrumented_base_store_f64(ptr: i32, value: f64, offset: i32) -> ();
    fn instrumented_memory_grow(amount: i32, idx: i32) -> i32;
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

#[derive(PartialEq, Debug)]
pub enum WasmType {
    I32,
    F32,
    I64,
    F64,
}

impl From<&i32> for WasmType {
    fn from(serialized_type: &i32) -> Self {
        match serialized_type {
            &TYPE_I32 => Self::I32,
            &TYPE_F32 => Self::F32,
            &TYPE_I64 => Self::I64,
            &TYPE_F64 => Self::F64,
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

#[derive(Debug, Clone, PartialEq)]
pub enum WasmValue {
    I32(i32),
    F32(f32),
    I64(i64),
    F64(f64),
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

    pub fn i32_from_bool(b: bool) -> Self {
        if b {
            Self::I32(0)
        } else {
            Self::I32(1)
        }
    }
}

#[derive(Debug)]
pub struct FunctionIndex(pub i32);

pub struct WasmFunction {
    pub f_apply: i32,
    pub instr_f_idx: i32,
    pub sigv: i32,
}

#[cfg(feature = "std")]
impl std::fmt::Debug for WasmFunction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("WasmFunction")
            .field("base_apply_idx", &self.f_apply)
            .field("uninstr_idx", &self.instr_f_idx)
            .field("sig_pointer", &self.sigv)
            .finish()
    }
}

#[derive(Debug)]
pub struct FunctionTableIndex(pub i32);

#[derive(Debug)]
pub struct FunctionTable(pub i32);

#[derive(Debug)]
pub struct RuntimeValues {
    pub argc: i32,
    pub resc: i32,
    pub sigv: i32,
    pub signature_types: Vec<WasmType>,
    pub signature_offsets: Vec<usize>,
}

impl WasmFunction {
    pub fn new(f_apply: i32, instr_f_idx: i32, sigv: i32) -> Self {
        WasmFunction {
            f_apply,
            instr_f_idx,
            sigv,
        }
    }

    pub fn apply(&self) -> () {
        unsafe { call_base(self.f_apply, self.sigv) };
    }

    pub fn instr_f_idx(&self) -> FunctionIndex {
        FunctionIndex(self.instr_f_idx)
    }
}

pub type MutDynResults = RuntimeValues;
pub type MutDynArgs = RuntimeValues;

extern crate alloc;
use alloc::vec::Vec;

impl RuntimeValues {
    pub fn new(argc: i32, resc: i32, sigv: i32, sigtypv: i32) -> Self {
        let total_values = argc + resc;
        let mut signature_types: Vec<WasmType> = Vec::with_capacity(total_values as usize);
        let mut signature_offsets: Vec<usize> = Vec::with_capacity(total_values as usize);

        let mut offset = 0;
        for value_index in 0..total_values {
            let serialized_type =
                unsafe { wastrumentation_stack_load_i32(sigtypv, value_index * 4) };
            let wasm_type: WasmType = (&serialized_type).into();

            signature_offsets.push(offset);
            offset += wasm_type.size();
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

#[derive(Debug)]
pub struct PathContinuation(pub i32);
impl SerializedContinuation for PathContinuation {
    fn low_level_continuation(&self) -> &i32 {
        let Self(continuation) = self;
        continuation
    }

    fn with(continuation: i32) -> Self {
        Self(continuation)
    }
}

#[derive(Debug)]
pub struct ParameterBrIfCondition(pub i32);
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
pub struct BranchTargetLabel(pub i64);
impl BranchTargetLabel {
    pub fn label(&self) -> i64 {
        let Self(label) = self;
        *label
    }
}

#[derive(Debug)]
pub struct ParameterBrIfLabel(pub i32);
impl ParameterBrIfLabel {
    pub fn label(&self) -> i32 {
        let Self(label) = self;
        *label
    }
}

#[derive(Debug)]
pub struct BranchTableTarget(pub i32);
impl BranchTableTarget {
    pub fn target(&self) -> &i32 {
        let Self(target) = self;
        target
    }
}

#[derive(Debug)]
pub struct BranchTableDefault(pub i32);
impl BranchTableDefault {
    pub fn value(&self) -> &i32 {
        let Self(value) = self;
        value
    }
}

#[derive(Debug)]
pub struct LocalIndex(pub i64);
impl LocalIndex {
    pub fn value(&self) -> &i64 {
        let Self(value) = self;
        value
    }
}

#[derive(Debug)]
pub enum LocalOp {
    Get,
    Set,
    Tee,
}

#[derive(Debug)]
pub struct GlobalIndex(pub i64);
impl GlobalIndex {
    pub fn value(&self) -> &i64 {
        let Self(value) = self;
        value
    }
}

#[derive(Debug)]
pub enum GlobalOp {
    Get,
    Set,
}

#[macro_export]
macro_rules! advice {
    (call pre
        ($func_ident: ident : FunctionIndex $(,)?) $body:block
    ) => {
        #[no_mangle]
        pub extern "C"
        fn specialized_call_pre ($func_ident:FunctionIndex) -> ()
        $body
    };
    (call post
        ($func_ident: ident : FunctionIndex $(,)?) $body:block
    ) => {
        #[no_mangle]
        pub extern "C"
        fn specialized_call_post ($func_ident:FunctionIndex) -> ()
        $body
    };
    (call_indirect pre
        ($func_table_index_ident: ident : FunctionTableIndex, $func_table_ident: ident : FunctionTable $(,)?) $body:block
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
    (call_indirect post
        ($func_table_ident: ident : FunctionTable $(,)?) $body:block
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
    (apply
        (
            $func_ident: ident : WasmFunction,
            $args_ident: ident : MutDynArgs,
            $ress_ident: ident : MutDynResults $(,)?
        ) $body:block
    ) => {
        #[no_mangle]
        pub extern "C"
        fn generic_apply (f_apply: i32, instr_f_idx: i32, argc: i32, resc: i32, sigv: i32, sigtypv: i32) -> () {
            let $func_ident = WasmFunction::new(f_apply, instr_f_idx, sigv);
            let mut $args_ident = MutDynResults::new(argc, resc, sigv, sigtypv);
            let mut $ress_ident = MutDynArgs::new(argc, resc, sigv, sigtypv);
            $body
        }
    };
    (br
        (
            $target_label: ident : BranchTargetLabel $(,)?
        ) $body:block
    ) => {
        #[no_mangle]
        pub extern "C"
        fn specialized_br (
            low_level_label: i64,
        ) {
            let $target_label = BranchTargetLabel(low_level_label);
            $body
        }
    };
    (if_
        (
            $path_continuation: ident : PathContinuation $(,)?
        ) $body:block
    ) => {
        #[no_mangle]
        pub extern "C"
        fn specialized_if_then_k (
            path_continuation: i32,
        ) -> i32 {
            let $path_continuation = PathContinuation(path_continuation);
            let PathContinuation(path_continuation) = $body;
            path_continuation
        }
    };
    (if_then
        (
            $path_continuation: ident : PathContinuation $(,)?
        ) $body:block) => {
        #[no_mangle]
        pub extern "C"
        fn specialized_if_then_else_k (
            path_continuation: i32,
        ) -> i32 {
            let $path_continuation = PathContinuation(path_continuation);
            let PathContinuation(path_continuation) = $body;
            path_continuation
        }
    };
    (br_if
        (
            $path_continuation: ident : ParameterBrIfCondition,
            $target_label: ident : ParameterBrIfLabel $(,)?
        ) $body:block
    ) => {
        #[no_mangle]
        pub extern "C"
        fn specialized_br_if (
            path_continuation: i32,
            low_level_label: i32,
        ) -> i32 {
            let $path_continuation = ParameterBrIfCondition(path_continuation);
            let $target_label = ParameterBrIfLabel(low_level_label);
            let ParameterBrIfCondition(path_continuation) = $body;
            path_continuation
        }
    };
    (br_table
        (
            $branch_table_target: ident : BranchTableTarget,
            $branch_table_default: ident : BranchTableDefault $(,)?
        ) $body:block
    ) => {
        #[no_mangle]
        pub extern "C"
        fn specialized_br_table (
            br_table_target: i32,
            br_table_default: i32,
        ) -> i32 {
            let $branch_table_target = BranchTableTarget(br_table_target);
            let $branch_table_default = BranchTableDefault(br_table_default);
            let BranchTableTarget(br_table_target) = $body;
            br_table_target
        }
    };
    (select
        (
            $path_continuation: ident : PathContinuation $(,)?
        ) $body:block
     ) => {
        #[no_mangle]
        pub extern "C"
        fn specialized_select (
            path_continuation: i32,
        ) -> i32 {
            let $path_continuation = PathContinuation(path_continuation);
            let PathContinuation(path_continuation) = $body;
            path_continuation
        }
    };
    ///////////
    // UNARY //
    ///////////
    (unary generic
        (
            $operator: ident: UnaryOperator,
            $operand: ident: WasmValue $(,)?
        ) $body:block
    ) => {
        fn generic_unary_trap(
            operator: UnaryOperator,
            operand: WasmValue,
        ) -> WasmValue {
            let $operator = operator;
            let $operand = operand;
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
        extern "C" fn $function_name(operand: $operand_type, operator: i32) -> $outcome_type {
            let operator = UnaryOperator::from(operator);
            let operand = WasmValue::$operand_type_wasm_value(operand);
            let outcome = $generic_unary_trap(operator, operand);
            let WasmValue::$outcome_type_wasm_value(outcome) = outcome else {
                panic!("Attempted to convert {outcome:?} to $outcome_type_wasm_value"); // TODO: stringify!
            };
            outcome
        }
    };
    ///////////
    // BINARY //
    ///////////
    (binary generic
        (
            $operator: ident: BinaryOperator,
            $l: ident: WasmValue,
            $r: ident: WasmValue $(,)?
        ) $body:block
    ) => {
        fn generic_binary_trap(
            operator: BinaryOperator,
            l: WasmValue,
            r: WasmValue,
        ) -> WasmValue {
            let $operator = operator;
            let $l = l;
            let $r = r;
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
        extern "C" fn $function_name(l_op: $l_type, r_op: $r_type, operator: i32) -> $outcome_type {
            let operator = BinaryOperator::from(operator);
            let l_op = WasmValue::$l_type_wasm_value(l_op);
            let r_op = WasmValue::$r_type_wasm_value(r_op);
            let outcome = $generic_binary_trap(operator, l_op, r_op);
            let WasmValue::$outcome_type_wasm_value(outcome) = outcome else {
                panic!("Attempted to convert {outcome:?} to $outcome_type_wasm_value"); // TODO: stringify!
            };
            outcome
        }
    };
    (drop () $body:block) => {
        #[no_mangle]
        extern "C" fn drop_trap() {
            $body
        }
    };
    (return_ () $body:block) => {
        #[no_mangle]
        extern "C" fn return_trap() {
            $body
        }
    };
    (const_ generic
        (
            $value: ident: WasmValue $(,)?
        ) $body:block
    ) => {
        fn generic_const_trap(value: WasmValue) -> WasmValue {
            let $value = value;
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
        extern "C" fn $function_name(const_: $const_type) -> $const_type {
            let const_ = WasmValue::$const_type_wasm_value(const_);
            let outcome = $generic_const_trap(const_);
            let WasmValue::$const_type_wasm_value(outcome) = outcome else {
                panic!("Attempted to convert {outcome:?} to $const_type_wasm_value"); // TODO: stringify!
            };
            outcome
        }
    };
    (local generic (
        $value: ident: WasmValue,
        $index: ident: LocalIndex,
        $local_op: ident: LocalOp $(,)?
    ) $body:block) => {
        fn generic_local_trap(
            value: WasmValue,
            index: LocalIndex,
            local_op: LocalOp,
        ) -> WasmValue {
            let $value = value;
            let $index = index;
            let $local_op = local_op;
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
        extern "C" fn $function_name(operand: $value_type, index: i64) -> $value_type {
            let operand = WasmValue::$value_type_wasm_value(operand);
            let index = LocalIndex(index);
            let local_op = LocalOp::$op;
            let outcome = $generic_local_trap(operand, index, local_op);
            let WasmValue::$value_type_wasm_value(outcome) = outcome else {
                panic!("Attempted to convert {outcome:?} to $value_type_wasm_value"); // TODO: stringify!
            };
            outcome
        }
    };
    (global generic (
        $value: ident: WasmValue,
        $index: ident: GlobalIndex,
        $global_op: ident: GlobalOp $(,)?
    ) $body:block) => {
        fn generic_global_trap(
            value: WasmValue,
            index: GlobalIndex,
            global_op: GlobalOp,
        ) -> WasmValue {
            let $value = value;
            let $index = index;
            let $global_op = global_op;
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
        extern "C" fn $function_name(operand: $value_type, index: i64) -> $value_type {
            let operand = WasmValue::$value_type_wasm_value(operand);
            let index = GlobalIndex(index);
            let global_op = GlobalOp::$op;
            let outcome = $generic_global_trap(operand, index, global_op);
            let WasmValue::$value_type_wasm_value(outcome) = outcome else {
                panic!("Attempted to convert {outcome:?} to $value_type_wasm_value"); // TODO: stringify!
            };
            outcome
        }
    };
    // LOAD
    (load generic (
        $load_index: ident: LoadIndex,
        $offset: ident: LoadOffset,
        $operation: ident: LoadOperation $(,)?
    ) $body:block) => {
        fn generic_load_trap(
            load_index: LoadIndex,
            offset: LoadOffset,
            operation: LoadOperation,
        ) -> WasmValue {
            let $load_index = load_index;
            let $offset = offset;
            let $operation = operation;
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
        extern "C" fn $function_name(load_idx: i32, offset: i64, operation: i32) -> $load_type {
            let load_index = LoadIndex(load_idx);
            let offset = LoadOffset(offset);
            let operation = LoadOperation::deserialize(&operation);
            let outcome = $generic_load_trap(load_index, offset, operation);
            let WasmValue::$load_type_wasm_value(outcome) = outcome else {
                panic!("Attempted to convert {outcome:?} to $value_type_wasm_value"); // TODO: stringify!
            };
            outcome
        }
    };
    // STORE
    (store generic (
        $store_index: ident: StoreIndex,
        $value: ident: WasmValue,
        $offset: ident: StoreOffset,
        $operation: ident: StoreOperation $(,)?
    ) $body:block) => {
        fn generic_store_trap(
            store_index: StoreIndex,
            value: WasmValue,
            offset: StoreOffset,
            operation: StoreOperation,
        ) {
            let $store_index = store_index;
            let $value = value;
            let $offset = offset;
            let $operation = operation;
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
        $store_type_wasm_value:ident) => {
        #[no_mangle]
        extern "C" fn $function_name(store_idx: i32, value: $store_type, offset: i64, operation: i32) {
            let store_index = StoreIndex(store_idx);
            let value = WasmValue::$store_type_wasm_value(value);
            let offset = StoreOffset(offset);
            let operation = StoreOperation::deserialize(&operation);
            $generic_store_trap(store_index, value, offset, operation);
        }
    };
    (memory_size
        ($size: ident: WasmValue, $index: ident: MemoryIndex $(,)?)
        $body:block
    ) => {
        #[no_mangle]
        extern "C" fn trap_memory_size(size: i32, idx: i64) -> i32 {
            let $size = WasmValue::I32(size);
            let $index = MemoryIndex(idx);
            let size: WasmValue = $body;
            size.as_i32()
        }
    };
    (memory_grow
        ($amount: ident: WasmValue, $index: ident: MemoryIndex $(,)?)
        $body:block
    ) => {
        #[no_mangle]
        extern "C" fn trap_memory_grow(amount: i32, idx: i64) -> i32 {
            let $amount = WasmValue::I32(amount);
            let $index = MemoryIndex(idx);
            let delta_or_neg_1: WasmValue = $body;
            delta_or_neg_1.as_i32()
        }
    };
}

use core::mem::size_of;
