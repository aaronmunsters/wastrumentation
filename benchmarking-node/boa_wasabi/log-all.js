/*
 * User-facing API for dynamic analyses.
 */

globalThis.analysis_results = {
    NUMBER_OF_CALLS: 0,
    MAX_CALL_DEPTH: 0,
    CALL_STACK: 0,
};

Wasabi.analysis = {
    start(location) {},
    nop(location) {},
    unreachable(location) {},
    if_(location, condition) {},
    br(location, target) {},
    br_if(location, conditionalTarget, condition) {},
    br_table(location, table, defaultTarget, tableIdx) {},
    begin(location, type) {},
    end(location, type, beginLocation, ifLocation) {},
    drop(location, value) {},
    select(location, cond, first, second) {},

    call_pre(location, targetFunc, args, indirectTableIdx) {
        /* [1] */
        analysis_results.CALL_STACK += 1;
        /* [2] */
        analysis_results.MAX_CALL_DEPTH = Math.max(analysis_results.MAX_CALL_DEPTH, analysis_results.CALL_STACK);
        /* [3] */
        analysis_results.NUMBER_OF_CALLS += 1;
    },

    call_post(location, values) {
        analysis_results.CALL_STACK -= 1;
    },

    return_(location, values) {},
    const_(location, op, value) {},
    unary(location, op, input, result) {},
    binary(location, op, first, second, result) {},
    load(location, op, memarg, value) {},
    store(location, op, memarg, value) {},
    memory_size(location, currentSizePages) {},
    memory_grow(location, byPages, previousSizePages) {},
    local(location, op, localIndex, value) {},
    global(location, op, globalIndex, value) {},
};
