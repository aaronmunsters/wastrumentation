use std::{cell::RefCell, collections::HashMap, ptr::addr_of, sync::LazyLock};
use wastrumentation_rs_stdlib::*;

static mut GLOBALS: LazyLock<RefCell<HashMap<i64, AccessCount>>> =
    LazyLock::new(|| RefCell::new(HashMap::new()));

struct AccessCount {
    pub read_count: i32,
    pub write_count: i32,
}

fn inc_count(index: GlobalIndex, op: GlobalOp) {
    let globals = unsafe { addr_of!(GLOBALS).as_ref().unwrap() };
    globals
        .borrow_mut()
        .entry(index.value())
        .and_modify(|mana| match op {
            GlobalOp::Get => mana.read_count += 1,
            GlobalOp::Set => mana.write_count += 1,
        })
        .or_insert(AccessCount {
            read_count: 0,
            write_count: 0,
        });
}

const READ_WRITE_SERIALIZATION_FAILED: i32 = -1;
const GLOBAL_INDEX_NO_READ_NO_WRITE: i32 = -2;

#[no_mangle]
pub fn get_inc(global_index: i64, read_or_write: i32) -> i32 {
    let globals = unsafe { addr_of!(GLOBALS).as_ref().unwrap().borrow() };
    globals
        .get(&global_index)
        .map(|v| match read_or_write {
            0 => v.read_count,
            1 => v.write_count,
            _ => READ_WRITE_SERIALIZATION_FAILED,
        })
        .unwrap_or(GLOBAL_INDEX_NO_READ_NO_WRITE)
}

advice! { global (v: WasmValue, index: GlobalIndex, op: GlobalOp, _l: Location) {
    inc_count(index, op); /* return */ v
} }
