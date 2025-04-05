const counts = {};

const incInstr = (f_idx, i_idx) => {
    counts[f_idx] = counts[f_idx] || {};
    counts[f_idx][i_idx] = (counts[f_idx][i_idx] || 0) + 1;
};

Wasabi.analysis = {
    nop: ({func, instr}) => incInstr(func, instr),
    unreachable: ({func, instr}) => incInstr(func, instr),
    if_: ({func, instr}, _c) => incInstr(func, instr),
    br: ({func, instr}, _t) => incInstr(func, instr),
    br_if: ({func, instr}, _ct, _c) => incInstr(func, instr),
    br_table: ({func, instr}, _t, _d, _ti) => incInstr(func, instr),
    begin: ({func, instr}, _t, _e) => incInstr(func, instr),
    drop: ({func, instr}, _v) => incInstr(func, instr),
    select: ({func, instr}, _f, _s, _c) => incInstr(func, instr),
    call_pre: ({func, instr}, _t, _a, _i) => incInstr(func, instr),
    return_: ({func, instr}, _vs) => incInstr(func, instr),
    const_: ({func, instr}, _o, _v) => incInstr(func, instr),
    end: ({func, instr}, _t, _e) => incInstr(func, instr),
    call_post: ({func, instr}, _vs) => incInstr(func, instr),
    unary: ({func, instr}, _o, _i, _r) => incInstr(func, instr),
    binary: ({func, instr}, _o, _f, _s, _r) => incInstr(func, instr),
    load: ({func, instr}, _o, _m, _v) => incInstr(func, instr),
    store: ({func, instr}, _o, _m, _v) => incInstr(func, instr),
    memory_size: ({func, instr}, _c) => incInstr(func, instr),
    memory_grow: ({func, instr}, _p, previousSizePages) => incInstr(func, instr),
    local: ({func, instr}, _o, _i, _v) => incInstr(func, instr),
    global: ({func, instr}, _o, _i, _v) => incInstr(func, instr),
};

Wasabi.analysisResult = () => counts;
