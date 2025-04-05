const counts = {};

const incInstr = (instr) => counts[instr] = (counts[instr] || 0) + 1;

Wasabi.analysis = {
    nop(location) {
        incInstr("nop");
    },
    unreachable(location) {
        incInstr("unreachable");
    },
    if_(location, condition) {
        incInstr("if");
    },
    br(location, target) {
        incInstr("br");
    },
    br_if(location, conditionalTarget, condition) {
        incInstr("br_if");
    },
    br_table(location, table, defaultTarget, tableIdx) {
        incInstr("br_table");
    },
    begin(location, type, end) {
        // if and else are already counted by if_ hook
        if (type !== "if" && type !== "function") incInstr(type);
    },
    drop(location, value) {
        incInstr("drop");
    },
    select(location, first, second, cond) {
        incInstr("select");
    },
    call_pre(location, targetFunc, args, indirectTableIdx) {
        incInstr((indirectTableIdx === undefined) ? "call" : "call_indirect");
    },
    return_(location, values) {
        incInstr("return");
    },
    const_(location, op, value) {
        incInstr(op);
    },
    end(location, type, end) {
        if (type !== "if" && type !== "function") incInstr(type);
    },
    call_post(location, values) {
        incInstr("call_post");
    },
    unary(location, op, input, result) {
        incInstr(op);
    },
    binary(location, op, first, second, result) {
        incInstr(op);
    },
    load(location, op, memarg, value) {
        incInstr(op);
    },
    store(location, op, memarg, value) {
        incInstr(op);
    },
    memory_size(location, currentSizePages) {
        incInstr("memory_size");
    },
    memory_grow(location, byPages, previousSizePages) {
        incInstr("memory_grow");
    },
    local(location, op, localIndex, value) {
        incInstr(op);
    },
    global(location, op, globalIndex, value) {
        incInstr(op);
    },
};

Wasabi.analysisResult = () => counts;
