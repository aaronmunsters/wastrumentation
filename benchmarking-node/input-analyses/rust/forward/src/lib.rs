#![no_std]

extern crate wastrumentation_rs_stdlib;

use wastrumentation_rs_stdlib::*;

advice! { if_                (c: PathContinuation, _ic: IfThenElseInputCount, _ia: IfThenElseArity   , _l: Location) { c                         } }
advice! { if_post            (                                                                         _l: Location) {                           } }
advice! { if_then            (c: PathContinuation, _ic: IfThenInputCount, _ia: IfThenArity           , _l: Location) { c                         } }
advice! { if_then_post       (                                                                         _l: Location) {                           } }
advice! { br                 (_l: BranchTargetLabel                                                  , _l: Location) {                           } }
advice! { br_if              (c : ParameterBrIfCondition, _l : ParameterBrIfLabel                    , _l: Location) { c                         } }
advice! { br_table           (bt: BranchTableTarget, _e: BranchTableEffective, _d: BranchTableDefault, _l: Location) { bt                        } }
advice! { select             (c: PathContinuation                                                    , _l: Location) { c                         } }
advice! { call pre           (_t : FunctionIndex                                                     , _l: Location) {                           } }
advice! { call post          (_t : FunctionIndex                                                     , _l: Location) {                           } }
advice! { call_indirect pre  (t: FunctionTableIndex, _f: FunctionTable                               , _l: Location) { t                         } }
advice! { call_indirect post (_t: FunctionTable                                                      , _l: Location) {                           } }
advice! { unary generic      (opt: UnaryOperator, opnd: WasmValue                                    , _l: Location) { opt.apply(opnd)           } }
advice! { binary generic     ( opt: BinaryOperator, l_opnd: WasmValue, r_opnd: WasmValue             , _l: Location) { opt.apply(l_opnd, r_opnd) } }
advice! { drop               (                                                                         _l: Location) {                           } }
advice! { return_            (                                                                         _l: Location) {                           } }
advice! { const_ generic     (v: WasmValue                                                           , _l: Location) { v                         } }
advice! { local generic      (v: WasmValue, _i: LocalIndex, _l: LocalOp                              , _l: Location) { v                         } }
advice! { global generic     (v: WasmValue, _i: GlobalIndex, _g: GlobalOp                            , _l: Location) { v                         } }
advice! { load generic       (i: LoadIndex, o: LoadOffset, op: LoadOperation                         , _l: Location) { op.perform(&i, &o)        } }
advice! { store generic      (i: StoreIndex, v: WasmValue, o: StoreOffset, op: StoreOperation        , _l: Location) { op.perform(&i, &v, &o);   } }
advice! { memory_size        (s: WasmValue, _i: MemoryIndex                                          , _l: Location) { s                         } }
advice! { memory_grow        (a: WasmValue, i: MemoryIndex                                           , _l: Location) { i.grow(a)                 } }
advice! { block pre          (_bi: BlockInputCount, _ba: BlockArity                                  , _l: Location) {                           } }
advice! { block post         (                                                                         _l: Location) {                           } }
advice! { loop_ pre          (_li: LoopInputCount, _la: LoopArity                                    , _l: Location) {                           } }
advice! { loop_ post         (                                                                         _l: Location) {                           } }
