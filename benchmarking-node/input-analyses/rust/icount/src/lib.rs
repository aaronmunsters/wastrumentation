use enum_collections::{EnumMap, Enumerated};
use std::{
    cell::RefCell,
    collections::HashMap,
    ptr::{addr_of, addr_of_mut},
    sync::LazyLock,
};
use wastrumentation_rs_stdlib::*;

type CountsMapping = HashMap<i64, EnumMap<Instruction, i32, { Instruction::SIZE }>>;
static mut COUNTS: LazyLock<RefCell<CountsMapping>> =
    LazyLock::new(|| RefCell::new(HashMap::new()));

fn inc_instr(loc: Location, instruction_kind: Instruction) {
    let counts = unsafe { addr_of_mut!(COUNTS).as_ref().unwrap() };
    counts
        .borrow_mut()
        .entry(loc.function_index())
        .and_modify(|map| map[instruction_kind] += 1)
        .or_insert(EnumMap::new_default());
}

#[rustfmt::skip]
#[no_mangle]
pub fn total_instrs() -> usize { Instruction::SIZE }

#[no_mangle]
pub fn total_counted_fs() -> usize {
    unsafe { addr_of!(COUNTS).as_ref().unwrap().borrow().len() }
}

const ERROR_INSTRUCTION_INDEX_DOES_NOT_EXIST: i32 = -1;
const ERROR_FUNCTION_INDEX_DOES_NOT_EXIST: i32 = -1;

#[no_mangle]
pub fn get_count_for(f_idx: i64, serialized_index: i32) -> i32 {
    let counts = unsafe { addr_of!(COUNTS).as_ref().unwrap() };
    counts
        .borrow()
        .get(&f_idx)
        .map(|map| {
            *map.iter()
                .step_by(serialized_index as usize)
                .next()
                .unwrap_or(&ERROR_INSTRUCTION_INDEX_DOES_NOT_EXIST)
        })
        .unwrap_or(ERROR_FUNCTION_INDEX_DOES_NOT_EXIST)
}

#[derive(Enumerated, Debug, Copy, Clone)]
enum Instruction {
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

advice! {
    if_then_else       (c: PathContinuation, _ic: IfThenElseInputCount, _ia: IfThenElseArity   , loc: Location) { inc_instr(loc, Instruction::If);               c                         }
    if_then_else_post  (                                                                         loc: Location) { inc_instr(loc, Instruction::IfPost);                                     }
    if_then            (c: PathContinuation, _ic: IfThenInputCount, _ia: IfThenArity           , loc: Location) { inc_instr(loc, Instruction::IfThen);           c                         }
    if_then_post       (                                                                         loc: Location) { inc_instr(loc, Instruction::IfThenPost);                                 }
    br                 (_l: BranchTargetLabel                                                  , loc: Location) { inc_instr(loc, Instruction::Br);                                         }
    br_if              (c : ParameterBrIfCondition, _l : ParameterBrIfLabel                    , loc: Location) { inc_instr(loc, Instruction::BrIf);             c                         }
    br_table           (bt: BranchTableTarget, _e: BranchTableEffective, _d: BranchTableDefault, loc: Location) { inc_instr(loc, Instruction::BrIfTable);        bt                        }
    select             (c: PathContinuation                                                    , loc: Location) { inc_instr(loc, Instruction::Select);           c                         }
    call pre           (_t : FunctionIndex                                                     , loc: Location) { inc_instr(loc, Instruction::CallPre);                                    }
    call post          (_t : FunctionIndex                                                     , loc: Location) { inc_instr(loc, Instruction::CallPost);                                   }
    call_indirect pre  (t: FunctionTableIndex, _f: FunctionTable                               , loc: Location) { inc_instr(loc, Instruction::CallIndirectPre);  t                         }
    call_indirect post (_t: FunctionTable                                                      , loc: Location) { inc_instr(loc, Instruction::CallIndirectPost);                           }
    unary              (opt: UnaryOperator, opnd: WasmValue                                    , loc: Location) { inc_instr(loc, Instruction::Unary);            opt.apply(opnd)           }
    binary             ( opt: BinaryOperator, l_opnd: WasmValue, r_opnd: WasmValue             , loc: Location) { inc_instr(loc, Instruction::Binary);           opt.apply(l_opnd, r_opnd) }
    drop               (                                                                         loc: Location) { inc_instr(loc, Instruction::Drop);                                       }
    return_            (                                                                         loc: Location) { inc_instr(loc, Instruction::Return);                                     }
    const_             (v: WasmValue                                                           , loc: Location) { inc_instr(loc, Instruction::Const);            v                         }
    local              (v: WasmValue, _i: LocalIndex, _l: LocalOp                              , loc: Location) { inc_instr(loc, Instruction::Local);            v                         }
    global             (v: WasmValue, _i: GlobalIndex, _g: GlobalOp                            , loc: Location) { inc_instr(loc, Instruction::Global);           v                         }
    load               (i: LoadIndex, o: LoadOffset, op: LoadOperation                         , loc: Location) { inc_instr(loc, Instruction::Load);             op.perform(&i, &o)        }
    store              (i: StoreIndex, v: WasmValue, o: StoreOffset, op: StoreOperation        , loc: Location) { inc_instr(loc, Instruction::Store);            op.perform(&i, &v, &o);   }
    memory_size        (s: WasmValue, _i: MemoryIndex                                          , loc: Location) { inc_instr(loc, Instruction::MemorySize);       s                         }
    memory_grow        (a: WasmValue, i: MemoryIndex                                           , loc: Location) { inc_instr(loc, Instruction::MemoryGrow);       i.grow(a)                 }
    block pre          (_bi: BlockInputCount, _ba: BlockArity                                  , loc: Location) { inc_instr(loc, Instruction::BlockPre);                                   }
    block post         (                                                                         loc: Location) { inc_instr(loc, Instruction::BlockPost);                                  }
    loop_ pre          (_li: LoopInputCount, _la: LoopArity                                    , loc: Location) { inc_instr(loc, Instruction::LoopPre);                                    }
    loop_ post         (                                                                         loc: Location) { inc_instr(loc, Instruction::LoopPost);                                   }
}
