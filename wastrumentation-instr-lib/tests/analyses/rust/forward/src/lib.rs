use wastrumentation_rs_stdlib::*;

/////
// START ADVICE SPECIFICATION //
//                         /////

advice! { apply (function : WasmFunction, args : MutDynArgs, ress : MutDynResults) {
        let _ = args;
        let _ = ress;
        function.apply();
    }
}

advice! { if_then_else (
        path_continuation: PathContinuation,
        if_then_else_input_c: IfThenElseInputCount,
        if_then_else_arity: IfThenElseArity,
        location: Location,
    ) {
        let _ = if_then_else_input_c;
        let _ = if_then_else_arity;
        let _ = location;
        path_continuation
    }
}

advice! { if_then_else_post (
        location: Location,
    ) {
        let _ = location;
    }
}

advice! { if_then (
        path_continuation: PathContinuation,
        if_then_input_c: IfThenInputCount,
        if_then_arity: IfThenArity,
        location: Location,
    ) {
        let _ = if_then_input_c;
        let _ = if_then_arity;
        let _ = location;
        path_continuation
    }
}

advice! { if_then_post (
        location: Location,
    ) {
        let _ = location;
    }
}

advice! { br (
        branch_target_label: BranchTargetLabel,
        location: Location,
    ) {
        let _ = branch_target_label;
        let _ = location;
    }
}

advice! { br_if (
        path_continuation : ParameterBrIfCondition,
        target_label : ParameterBrIfLabel,
        location: Location,
    ) {
        let _ = target_label;
        let _ = location;
        path_continuation
    }
}

advice! { br_table (
        branch_table_target: BranchTableTarget,
        branch_table_effective: BranchTableEffective,
        branch_table_default: BranchTableDefault,
        location: Location,
    ) {
        let _ = branch_table_effective;
        let _ = branch_table_default;
        let _ = location;
        branch_table_target
    }
}

advice! { select (
        path_continuation: PathContinuation,
        location: Location,
    ) {
        let _ = location;
        path_continuation
    }
}

advice! { call pre (
        target_func : FunctionIndex,
        location: Location,
    ) {
        let _ = target_func;
        let _ = location;
    }
}

advice! { call post (
        target_func: FunctionIndex,
        location: Location,
    ) {
        let _ = target_func;
        let _ = location;
    }
}

advice! { call_indirect pre (
        target_func: FunctionTableIndex,
        func_table_ident: FunctionTable,
        location: Location,
    ) {
        let _ = func_table_ident;
        let _ = location;
        target_func
    }
}

advice! { call_indirect post (
        target_func: FunctionTable,
        location: Location,
    ) {
        let _ = target_func;
        let _ = location;
    }
}

advice! { unary(
        operator: UnaryOperator,
        operand: WasmValue,
        location: Location,
    ) {
        let _ = location;
        operator.apply(operand)
    }
}

advice! { binary(
        operator: BinaryOperator,
        l_operand: WasmValue,
        r_operand: WasmValue,
        location: Location,
    ) {
        let _ = location;
        operator.apply(l_operand, r_operand)
    }
}

advice! { drop (
        location: Location,
    ) {
        let _ = location;
    }
}

advice! { return_ (
        location: Location,
    ) {
        let _ = location;
    }
}

advice! { const_(
        value: WasmValue,
        location: Location,
    ) {
        let _ = location;
        value
    }
}

advice! { local(
        value: WasmValue,
        index: LocalIndex,
        local_op: LocalOp,
        location: Location,
    ) {
        let _ = index;
        let _ = local_op;
        let _ = location;
        value
    }
}

advice! { global(
        value: WasmValue,
        index: GlobalIndex,
        global_op: GlobalOp,
        location: Location,
    ) {
        let _ = index;
        let _ = global_op;
        let _ = location;
        value
    }
}

advice! { load(
        store_index: LoadIndex,
        offset: LoadOffset,
        operation: LoadOperation,
        location: Location,
    ) {
        let _ = location;
        operation.perform(&store_index, &offset)
    }
}

advice! { store(
        store_index: StoreIndex,
        value: WasmValue,
        offset: StoreOffset,
        operation: StoreOperation,
        location: Location,
    ) {
        let _ = location;
        operation.perform(&store_index, &value, &offset);
    }
}

advice! { memory_size (
        size: WasmValue,
        index: MemoryIndex,
        location: Location,
    ) {
        let _ = index;
        let _ = location;
        size
    }
}

advice! { memory_grow (
        amount: WasmValue,
        index: MemoryIndex,
        location: Location,
    ) {
        let _ = location;
        index.grow(amount)
    }
}

advice! { block pre (
        block_input_count: BlockInputCount,
        block_arity: BlockArity,
        location: Location,
    ) {
        let _ = block_input_count;
        let _ = block_arity;
        let _ = location;
    }
}

advice! { block post (
        location: Location,
    ) {
        let _ = location;
    }
}

advice! { loop_ pre (
        loop_input_count: LoopInputCount,
        loop_arity: LoopArity,
        location: Location,
    ) {
        let _ = loop_input_count;
        let _ = loop_arity;
        let _ = location;
    }
}

advice! { loop_ post (
        location: Location,
    ) {
        let _ = location;
    }
}

advice! { ref_func (
        func_index: WasmValue,
        location: Location,
    ) {
        let _ = func_index;
        let _ = location;
    }
}

advice! { ref_null (
        location: Location,
    ) {
        let _ = location;
    }
}

advice! { ref_is_null (
        res: WasmValue,
        location: Location,
    ) {
        let _ = location;
        res
    }
}

advice! { table_set (
    element_index: WasmValue,
    table_index: FunctionTableIndex,
    location: Location,
    ) {
        let _ = table_index;
        let _ = location;
        element_index
    }
}

advice! { table_get (
        element_index: WasmValue,
        table_index: FunctionTableIndex,
        location: Location,
    ) {
        let _ = table_index;
        let _ = location;
        element_index
    }
}

advice! { table_size (
        table_size: WasmValue,
        table_index: FunctionTableIndex,
        location: Location,
    ) {
        let _ = table_index;
        let _ = location;
        table_size
    }
}

advice! { table_grow (
        grow_size: WasmValue,
        table_index: FunctionTableIndex,
        location: Location,
    ) {
        let _ = table_index;
        let _ = location;
        grow_size
    }
}

advice! { table_fill (
        index: WasmValue,
        fill_size: WasmValue,
        table_index: FunctionTableIndex,
        location: Location,
    ) {
        let _ = fill_size;
        let _ = table_index;
        let _ = location;
        index
    }
}

advice! { table_copy (
        dst_element_index: WasmValue,
        src_element_index: WasmValue,
        copy_size: WasmValue,
        dst_table_index: FunctionTableIndex,
        src_table_index: FunctionTableIndex,
        location: Location,
    ) {
        let _ = dst_table_index;
        let _ = src_table_index;
        let _ = location;
        (dst_element_index, src_element_index, copy_size)
    }
}

advice! { table_init (
        destination_table_offset: WasmValue,
        source_element_offset: WasmValue,
        init_size: WasmValue,
        table_index: FunctionTableIndex,
        element_index: ElementIndex,
        location: Location,
    ) {
        let _ = table_index;
        let _ = element_index;
        let _ = location;
        (destination_table_offset, source_element_offset, init_size)
    }
}

advice! { elem_drop (
        element_index: ElementIndex,
        location: Location,
    ) {
        let _ = element_index;
        let _ = location;
    }
}
