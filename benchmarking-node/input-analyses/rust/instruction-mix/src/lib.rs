#![no_std]

extern crate wastrumentation_rs_stdlib;

use strum::EnumCount;
use wastrumentation_rs_stdlib::*;

static mut COUNTS: [usize; InstructionIndex::COUNT] = [0; InstructionIndex::COUNT];

fn inc_instr(instruction_kind: InstructionIndex) {
    unsafe { COUNTS[instruction_kind as usize] += 1 };
}

#[no_mangle]
pub fn total_instrs() -> usize {
    InstructionIndex::COUNT
}

#[no_mangle]
pub fn get_count_instr(index: i32) -> usize {
    unsafe { COUNTS[index as usize] }
}

#[derive(Debug, EnumCount, Copy, Clone)]
#[repr(usize)] // Make the enum variants usize
enum InstructionIndex {
    If,
    IfPost,
    IfThen,
    IfThenPost,
    Br,
    BrIf,
    BrIfTable,
    Select,
    CallPre,
    CallPost,
    CallIndirectPre,
    CallIndirectPost,
    Unary,
    Binary,
    Drop,
    Return,
    Const,
    Local,
    Global,
    Load,
    Store,
    MemorySize,
    MemoryGrow,
    BlockPre,
    BlockPost,
    LoopPre,
    LoopPost,
}

advice! { if_                (c: PathContinuation, _ic: IfThenElseInputCount, _ia: IfThenElseArity   ) { inc_instr(InstructionIndex::If);               c                         } }
advice! { if_post            (                                                                       ) { inc_instr(InstructionIndex::IfPost);                                     } }
advice! { if_then            (c: PathContinuation, _ic: IfThenInputCount, _ia: IfThenArity           ) { inc_instr(InstructionIndex::IfThen);           c                         } }
advice! { if_then_post       (                                                                       ) { inc_instr(InstructionIndex::IfThenPost);                                 } }
advice! { br                 (_l: BranchTargetLabel                                                  ) { inc_instr(InstructionIndex::Br);                                         } }
advice! { br_if              (c : ParameterBrIfCondition, _l : ParameterBrIfLabel                    ) { inc_instr(InstructionIndex::BrIf);             c                         } }
advice! { br_table           (bt: BranchTableTarget, _e: BranchTableEffective, _d: BranchTableDefault) { inc_instr(InstructionIndex::BrIfTable);        bt                        } }
advice! { select             (c: PathContinuation                                                    ) { inc_instr(InstructionIndex::Select);           c                         } }
advice! { call pre           (_t : FunctionIndex                                                     ) { inc_instr(InstructionIndex::CallPre);                                    } }
advice! { call post          (_t : FunctionIndex                                                     ) { inc_instr(InstructionIndex::CallPost);                                   } }
advice! { call_indirect pre  (t: FunctionTableIndex, _f: FunctionTable                               ) { inc_instr(InstructionIndex::CallIndirectPre);  t                         } }
advice! { call_indirect post (_t: FunctionTable                                                      ) { inc_instr(InstructionIndex::CallIndirectPost);                           } }
advice! { unary generic      (opt: UnaryOperator, opnd: WasmValue                                    ) { inc_instr(InstructionIndex::Unary);            opt.apply(opnd)           } }
advice! { binary generic     ( opt: BinaryOperator, l_opnd: WasmValue, r_opnd: WasmValue             ) { inc_instr(InstructionIndex::Binary);           opt.apply(l_opnd, r_opnd) } }
advice! { drop               (                                                                       ) { inc_instr(InstructionIndex::Drop);                                       } }
advice! { return_            (                                                                       ) { inc_instr(InstructionIndex::Return);                                     } }
advice! { const_ generic     (v: WasmValue                                                           ) { inc_instr(InstructionIndex::Const);            v                         } }
advice! { local generic      (v: WasmValue, _i: LocalIndex, _l: LocalOp                              ) { inc_instr(InstructionIndex::Local);            v                         } }
advice! { global generic     (v: WasmValue, _i: GlobalIndex, _g: GlobalOp                            ) { inc_instr(InstructionIndex::Global);           v                         } }
advice! { load generic       (i: LoadIndex, o: LoadOffset, op: LoadOperation                         ) { inc_instr(InstructionIndex::Load);             op.perform(&i, &o)        } }
advice! { store generic      (i: StoreIndex, v: WasmValue, o: StoreOffset, op: StoreOperation        ) { inc_instr(InstructionIndex::Store);            op.perform(&i, &v, &o);   } }
advice! { memory_size        (s: WasmValue, _i: MemoryIndex                                          ) { inc_instr(InstructionIndex::MemorySize);       s                         } }
advice! { memory_grow        (a: WasmValue, i: MemoryIndex                                           ) { inc_instr(InstructionIndex::MemoryGrow);       i.grow(a)                 } }
advice! { block pre          (_bi: BlockInputCount, _ba: BlockArity                                  ) { inc_instr(InstructionIndex::BlockPre);                                   } }
advice! { block post         (                                                                       ) { inc_instr(InstructionIndex::BlockPost);                                  } }
advice! { loop_ pre          (_li: LoopInputCount, _la: LoopArity                                    ) { inc_instr(InstructionIndex::LoopPre);                                    } }
advice! { loop_ post         (                                                                       ) { inc_instr(InstructionIndex::LoopPost);                                   } }
