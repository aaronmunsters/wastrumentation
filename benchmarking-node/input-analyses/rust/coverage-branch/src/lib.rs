extern crate wastrumentation_rs_stdlib;

use std::{cell::RefCell, collections::HashMap, ptr::addr_of, sync::LazyLock};

use wastrumentation_rs_stdlib::*;

static mut COVERAGE: LazyLock<RefCell<HashMap<(i64, i64), i32>>> =
    std::sync::LazyLock::new(|| RefCell::new(HashMap::new()));

fn add_location(location: Location, condition: i32) {
    let function_index = location.function_index();
    let instruction_index = location.instruction_index();
    let location = (function_index, instruction_index);

    let coverage = unsafe { addr_of!(COVERAGE).as_ref().unwrap() };
    coverage.borrow_mut().insert(location, condition);
}

#[no_mangle]
pub fn get_coverage(function_index: i64, instruction_index: i64) -> i32 {
    let coverage = unsafe { addr_of!(COVERAGE).as_ref().unwrap().borrow() };
    let function_index_coverage = coverage.get(&(function_index, instruction_index));
    function_index_coverage.map(|v| *v).unwrap_or(-1)
}

advice! { if_then_else (c: PathContinuation, _ic: IfThenElseInputCount, _ia: IfThenElseArity, l: Location) { add_location(l, c.value()); c } }
advice! { if_then (c: PathContinuation, _ic: IfThenInputCount, _ia: IfThenArity, l: Location)              { add_location(l, c.value()); c } }
advice! { br_if (c : ParameterBrIfCondition, _l : ParameterBrIfLabel, l: Location)                         { add_location(l, c.value()); c } }
advice! { br_table (t: BranchTableTarget, e: BranchTableEffective, _d: BranchTableDefault, l: Location)    { add_location(l, e.label()); t } }
advice! { select (c: PathContinuation, l: Location)                                                        { add_location(l, c.value()); c } }
