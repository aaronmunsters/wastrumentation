extern crate wastrumentation_rs_stdlib;

use std::{cell::RefCell, collections::HashMap, ptr::addr_of, sync::LazyLock};

use wastrumentation_rs_stdlib::*;

static mut COVERAGE: LazyLock<RefCell<HashMap<(i64, i64, i32), i64>>> =
    std::sync::LazyLock::new(|| RefCell::new(HashMap::new()));

fn add_block_enter(location: Location, runtime_target: i32) {
    let function_index = location.function_index();
    let instruction_index = location.instruction_index();

    let mut coverage = unsafe { addr_of!(COVERAGE).as_ref().unwrap() }.borrow_mut();
    coverage
        .entry((function_index, instruction_index, runtime_target))
        .and_modify(|c| *c += 1)
        .or_insert(0);
}

#[no_mangle]
pub fn get_count(function_index: i64, instruction_index: i64, target: i32) -> i64 {
    let coverage = unsafe { addr_of!(COVERAGE).as_ref().unwrap().borrow() };
    let function_index_coverage = coverage.get(&(function_index, instruction_index, target));
    *function_index_coverage.unwrap_or(&0)
}

advice! { if_                (c: PathContinuation, _ic: IfThenElseInputCount, _ia: IfThenElseArity  , l: Location) { add_block_enter(l, c.value()); c  } }
advice! { if_then            (c: PathContinuation, _ic: IfThenInputCount, _ia: IfThenArity          , l: Location) { add_block_enter(l, c.value()); c  } }
advice! { call pre           (_t : FunctionIndex                                                    , l: Location) { add_block_enter(l, 0        );    } }
advice! { call_indirect pre  (t: FunctionTableIndex, _f: FunctionTable                              , l: Location) { add_block_enter(l, t.value()); t  } }
advice! { block pre          (_bi: BlockInputCount, _ba: BlockArity                                 , l: Location) { add_block_enter(l, 0        );    } }
advice! { loop_ pre          (_li: LoopInputCount, _la: LoopArity                                   , l: Location) { add_block_enter(l, 0        );    } }
