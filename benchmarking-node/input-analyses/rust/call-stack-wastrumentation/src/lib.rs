#![no_std]

extern crate wastrumentation_rs_stdlib;
use wastrumentation_rs_stdlib::{
    advice, FunctionIndex, FunctionTable, FunctionTableIndex, MutDynArgs, MutDynResults,
    WasmFunction,
};

// There is no option to make use of WASM Globals here.
// E.g. `pub static mut NUMBER_OF_APPLIES: i32 = 0;`
// As per requested here: https://users.rust-lang.org/t/exposing-globals-to-host-application-in-wasm/57562/2
// The reason being that the Rust compiler intentionally
// outputs the global as a pointer in the module memory,
// as such it would require the host to access memory
// anyway and a function call then facilitates it anyway
// Reference: https://github.com/rust-lang/rust/issues/65987

#[no_mangle]
pub extern "C" fn get_number_of_applies() -> i32 {
    unsafe { NUMBER_OF_APPLIES }
}

#[no_mangle]
pub extern "C" fn get_max_apply_depth() -> i32 {
    unsafe { MAX_APPLY_DEPTH }
}

#[no_mangle]
pub extern "C" fn get_number_of_calls() -> i32 {
    unsafe { NUMBER_OF_CALLS }
}

#[no_mangle]
pub extern "C" fn get_max_call_depth() -> i32 {
    unsafe { MAX_CALL_DEPTH }
}

static mut NUMBER_OF_APPLIES: i32 = 0;
static mut MAX_APPLY_DEPTH: i32 = 0;
static mut NUMBER_OF_CALLS: i32 = 0;
static mut MAX_CALL_DEPTH: i32 = 0;
static mut APPLY_STACK: i32 = 0;
static mut CALL_STACK: i32 = 0;

advice! {
    call pre
    (f: FunctionIndex) {
        let _ = f;
        unsafe {
            /* [1] */
            CALL_STACK += 1
        };
        unsafe {
            /* [2] */
            MAX_CALL_DEPTH = i32::max(MAX_CALL_DEPTH, CALL_STACK);
        }
        unsafe {
            /* [3] */
            NUMBER_OF_CALLS += 1;
        }
    }
}

advice! {
    call post
    (f: FunctionIndex) {
        let _ = f;
        unsafe {
            CALL_STACK -= 1;
        }
    }
}

advice! {
    call_indirect pre
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
    call_indirect post
    (table: FunctionTable) {
        let _ = table;
        unsafe {
            CALL_STACK -= 1;
        };
    }
}

advice! {
    apply
    (func: WasmFunction, args: MutDynArgs, results: MutDynResults) {
        let _ = args;
        let _ = results;

        // Before apply:
        // [1] Increment apply stack size
        // [2] Ensure highest apply stack size is recorded
        // [3] Ensure apply count is incremented
        // After apply:
        // [4] Ensure apply count is decremented

        unsafe {
            /* [1] */
            APPLY_STACK += 1;
        }
        unsafe {
            /* [2] */
            MAX_APPLY_DEPTH = i32::max(MAX_APPLY_DEPTH, APPLY_STACK);
        }
        unsafe {
            /* [3] */
            NUMBER_OF_APPLIES += 1;
        }
        func.apply();
        unsafe {
            /* [4] */
            APPLY_STACK -= 1;
        }
    }
}
