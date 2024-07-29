#![no_std]

extern crate wastrumentation_rs_stdlib;
use wastrumentation_rs_stdlib::{advice, FunctionIndex, FunctionTable, FunctionTableIndex};

#[no_mangle]
pub extern "C" fn get_number_of_calls() -> i32 {
    unsafe { NUMBER_OF_CALLS }
}

#[no_mangle]
pub extern "C" fn get_max_call_depth() -> i32 {
    unsafe { MAX_CALL_DEPTH }
}

#[no_mangle]
pub extern "C" fn get_call_stack() -> i32 {
    unsafe { CALL_STACK }
}

static mut NUMBER_OF_CALLS: i32 = 0;
static mut MAX_CALL_DEPTH: i32 = 0;
static mut CALL_STACK: i32 = 0;

advice! {
    advice call before
    (f: FunctionIndex) {
        let _ = f;
        unsafe {
            /* [1] */
            CALL_STACK += 1
        };
        unsafe {
            /* [2] */
            MAX_CALL_DEPTH = i32::max(MAX_CALL_DEPTH, CALL_STACK);
        };
        unsafe {
            /* [3] */
            NUMBER_OF_CALLS += 1;
        };
    }
}

advice! {
    advice call after
    (f: FunctionIndex) {
        let _ = f;
        unsafe {
            CALL_STACK -= 1;
        };
    }
}

advice! {
    advice call-indirect before
    (table_f_idx: FunctionTableIndex, table: FunctionTable) {
        let _ = table;
        unsafe {
            /* [1] */
            CALL_STACK += 1
        };
        unsafe {
            /* [2] */
            MAX_CALL_DEPTH = i32::max(MAX_CALL_DEPTH, CALL_STACK);
        };
        unsafe {
            /* [3] */
            NUMBER_OF_CALLS += 1;
        };
        table_f_idx
    }
}

advice! {
    advice call-indirect after
    (table: FunctionTable) {
        let _ = table;
        unsafe {
            CALL_STACK -= 1;
        };
    }
}
