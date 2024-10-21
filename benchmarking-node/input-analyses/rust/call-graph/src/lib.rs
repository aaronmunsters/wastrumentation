extern crate wastrumentation_rs_stdlib;

use std::{cell::RefCell, collections::HashSet, ptr::addr_of, sync::LazyLock};

use wastrumentation_rs_stdlib::*;

static mut CALL_GRAPH: LazyLock<RefCell<HashSet<(i64, i64)>>> =
    std::sync::LazyLock::new(|| RefCell::new(HashSet::new()));

fn add_edge(from: i64, to: i64) {
    let call_graph = unsafe { addr_of!(CALL_GRAPH).as_ref().unwrap() };
    call_graph.borrow_mut().insert((from, to));
}

fn get_edge_at(index: i32) -> Option<(i64, i64)> {
    let call_graph = unsafe { addr_of!(CALL_GRAPH).as_ref().unwrap() };
    call_graph
        .borrow()
        .iter()
        .nth(index.try_into().unwrap())
        .cloned()
}

#[no_mangle]
pub fn get_total_edges() -> i32 {
    let call_graph = unsafe { addr_of!(CALL_GRAPH).as_ref().unwrap() };
    call_graph.borrow().len().try_into().unwrap()
}

#[no_mangle]
pub fn get_from_at(index: i32) -> i64 {
    match get_edge_at(index) {
        Some((from, _to)) => from,
        None => panic!(),
    }
}

#[no_mangle]
pub fn get_to_at(index: i32) -> i64 {
    match get_edge_at(index) {
        Some((_from, to)) => to,
        None => panic!(),
    }
}

advice! { call pre (t : FunctionIndex, l: Location) {
        let _ = l;
        let caller = l.function_index();
        let callee = t.value().into();
        add_edge(caller, callee);
    }
}
