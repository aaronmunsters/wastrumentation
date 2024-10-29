#![no_std]

/// WARNING: This program is still a template.
///          This  program  contains  template
///          snippets  like  <TO_CODE_GEN {x}>
///          that   will  be  filled  in  with
///          results from the  static analysis
///          that  comes before  this  dynamic
///          analysis execution.
extern crate wastrumentation_rs_stdlib;

use wastrumentation_rs_stdlib::*;

const MAP_SIZE: usize = 0; // <TO_CODE_GEN {MAP_SIZE}>
static mut MAP: &mut [i32] = &mut [0; MAP_SIZE]; // Maps [FunctionIndex -> CallCount]

#[no_mangle]
pub extern "C" fn get_calls_for(index: i32) -> i32 {
    unsafe { MAP[index as usize] }
}

advice! { apply (func: WasmFunction, _args: MutDynArgs, _results: MutDynResults) {
        let map = unsafe { MAP.as_mut() };

        match func.instr_f_idx {
            // <TO_CODE_GEN {MAP_INCREMENT}>
            _ => (), // core::panic!(),
        }

        func.apply();
    }
}
