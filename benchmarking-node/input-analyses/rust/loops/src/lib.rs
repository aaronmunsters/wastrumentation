use std::{
    cell::RefCell,
    collections::HashMap,
    ptr::{addr_of, addr_of_mut},
    sync::LazyLock,
};
use wastrumentation_rs_stdlib::*;

type CountsMapping = HashMap<(i64, i64), i32>;
static mut COUNTS: LazyLock<RefCell<CountsMapping>> =
    LazyLock::new(|| RefCell::new(HashMap::new()));

fn inc_instr(loc: Location) {
    let counts = unsafe { addr_of_mut!(COUNTS).as_ref().unwrap() };
    counts
        .borrow_mut()
        .entry((loc.function_index(), loc.instruction_index()))
        .and_modify(|c| *c += 1)
        .or_insert(0);
}

#[no_mangle]
pub fn total_counted_loops() -> usize {
    unsafe { addr_of!(COUNTS).as_ref().unwrap() }.borrow().len()
}

const ERROR_INSTRUCTION_INDEX_NO_COUNT: i32 = -1;

#[no_mangle]
pub fn get_count_for(f_idx: i64, i_idx: i64) -> i32 {
    let counts = unsafe { addr_of!(COUNTS).as_ref().unwrap() };
    *counts
        .borrow()
        .get(&(f_idx, i_idx))
        .unwrap_or(&ERROR_INSTRUCTION_INDEX_NO_COUNT)
}

advice! { loop_ pre (_li: LoopInputCount, _la: LoopArity, loc: Location) { inc_instr(loc); } }
