extern crate wastrumentation_rs_stdlib;

use std::{cell::RefCell, collections::HashMap, ptr::addr_of, sync::LazyLock};

use wastrumentation_rs_stdlib::*;

static mut COVERAGE: LazyLock<RefCell<HashMap<(i64, i64), bool>>> =
    std::sync::LazyLock::new(|| RefCell::new(HashMap::new()));

fn add_location(location: Location) {
    let function_index = location.function_index();
    let instruction_index = location.instruction_index();
    let location = (function_index, instruction_index);

    let coverage = unsafe { addr_of!(COVERAGE).as_ref().unwrap() };
    coverage.borrow_mut().insert(location, true);
}

#[no_mangle]
pub fn get_coverage(function_index: i64, instruction_index: i64) -> i32 {
    let coverage = unsafe { addr_of!(COVERAGE).as_ref().unwrap().borrow() };
    let function_index_coverage = coverage.get(&(function_index, instruction_index));
    function_index_coverage.map(|_| 1).unwrap_or(0)
}

advice! { if_                (c: PathContinuation, _ic: IfThenElseInputCount, _ia: IfThenElseArity   , l: Location) { add_location(l); c                         } }
advice! { if_post            (                                                                         l: Location) { add_location(l);                           } }
advice! { if_then            (c: PathContinuation, _ic: IfThenInputCount, _ia: IfThenArity           , l: Location) { add_location(l); c                         } }
advice! { if_then_post       (                                                                         l: Location) { add_location(l);                           } }
advice! { br                 (_l: BranchTargetLabel                                                  , l: Location) { add_location(l);                           } }
advice! { br_if              (c : ParameterBrIfCondition, _l : ParameterBrIfLabel                    , l: Location) { add_location(l); c                         } }
advice! { br_table           (bt: BranchTableTarget, _e: BranchTableEffective, _d: BranchTableDefault, l: Location) { add_location(l); bt                        } }
advice! { select             (c: PathContinuation                                                    , l: Location) { add_location(l); c                         } }
advice! { call pre           (_t : FunctionIndex                                                     , l: Location) { add_location(l);                           } }
advice! { call post          (_t : FunctionIndex                                                     , l: Location) { add_location(l);                           } }
advice! { call_indirect pre  (t: FunctionTableIndex, _f: FunctionTable                               , l: Location) { add_location(l); t                         } }
advice! { call_indirect post (_t: FunctionTable                                                      , l: Location) { add_location(l);                           } }
advice! { unary generic      (opt: UnaryOperator, opnd: WasmValue                                    , l: Location) { add_location(l); opt.apply(opnd)           } }
advice! { binary generic     ( opt: BinaryOperator, l_opnd: WasmValue, r_opnd: WasmValue             , l: Location) { add_location(l); opt.apply(l_opnd, r_opnd) } }
advice! { drop               (                                                                         l: Location) { add_location(l);                           } }
advice! { return_            (                                                                         l: Location) { add_location(l);                           } }
advice! { const_ generic     (v: WasmValue                                                           , l: Location) { add_location(l); v                         } }
advice! { local generic      (v: WasmValue, _i: LocalIndex, _l: LocalOp                              , l: Location) { add_location(l); v                         } }
advice! { global generic     (v: WasmValue, _i: GlobalIndex, _g: GlobalOp                            , l: Location) { add_location(l); v                         } }
advice! { load generic       (i: LoadIndex, o: LoadOffset, op: LoadOperation                         , l: Location) { add_location(l); op.perform(&i, &o)        } }
advice! { store generic      (i: StoreIndex, v: WasmValue, o: StoreOffset, op: StoreOperation        , l: Location) { add_location(l); op.perform(&i, &v, &o);   } }
advice! { memory_size        (s: WasmValue, _i: MemoryIndex                                          , l: Location) { add_location(l); s                         } }
advice! { memory_grow        (a: WasmValue, i: MemoryIndex                                           , l: Location) { add_location(l); i.grow(a)                 } }
advice! { block pre          (_bi: BlockInputCount, _ba: BlockArity                                  , l: Location) { add_location(l);                           } }
advice! { block post         (                                                                         l: Location) { add_location(l);                           } }
advice! { loop_ pre          (_li: LoopInputCount, _la: LoopArity                                    , l: Location) { add_location(l);                           } }
advice! { loop_ post         (                                                                         l: Location) { add_location(l);                           } }
