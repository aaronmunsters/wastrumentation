use wastrumentation_rs_stdlib::*;

/////
// START ADVICE SPECIFICATION //
//                         /////

advice! { apply (function : WasmFunction, args : MutDynArgs, ress : MutDynResults) {
        println!("[ANALYSIS:] apply (pre) {function:#?}({args:#?})");
        function.apply();
        println!("[ANALYSIS:] apply (post) {function:#?}({args:#?}) = {ress:#?}");
    }
}

advice! { if_then_else (
        path_continuation: PathContinuation,
        if_then_else_input_c: IfThenElseInputCount,
        if_then_else_arity: IfThenElseArity,
        location: Location,
    ) {
        println!("[ANALYSIS:] if_ {path_continuation:#?} [if_then_else_input_c: {if_then_else_input_c:?}, if_then_else_arity: {if_then_else_arity:?}, location: {location:?}]");
        path_continuation
    }
}

advice! { if_then_else_post (
        location: Location,
    ) {
        println!("if_post (location: {location:?})");
    }
}

advice! { if_then (
    path_continuation: PathContinuation,
    if_then_input_c: IfThenInputCount,
    if_then_arity: IfThenArity,
    location: Location,
    ) {
        println!("[ANALYSIS:] if_then {path_continuation:#?} [if_then_input_c: {if_then_input_c:?}, if_then_arity: {if_then_arity:?}, location: {location:?}]");
        path_continuation
    }
}

advice! { if_then_post (
        location: Location,
    ) {
        println!("if_then_post (location: {location:?})");
    }
}

advice! { br (
        branch_target_label: BranchTargetLabel,
        location: Location,
    ) {
        println!("[ANALYSIS:] br {branch_target_label:#?}, location: {location:?}");
    }
}

advice! { br_if (
        path_continuation : ParameterBrIfCondition,
        target_label : ParameterBrIfLabel,
        location: Location,
    ) {
        println!("[ANALYSIS:] br_if {path_continuation:#?} to {target_label:#?}, location: {location:?}");
        path_continuation
    }
}

advice! { br_table (
        branch_table_target: BranchTableTarget,
        branch_table_effective: BranchTableEffective,
        branch_table_default: BranchTableDefault,
        location: Location,
    ) {
        println!("[ANALYSIS:] br_table {branch_table_target:#?} (effective: {branch_table_effective:#?}) (default: {branch_table_default:#?}), location: {location:?}");
        branch_table_target
    }
}

advice! { select (
        path_continuation: PathContinuation,
        location: Location,
    ) {
        println!("[ANALYSIS:] select {path_continuation:#?}, location: {location:?}");
        path_continuation
    }
}

advice! { call pre (
        target_func : FunctionIndex,
        location: Location,
    ) {
        println!("[ANALYSIS:] call pre {target_func:#?}, location: {location:?}");
    }
}

advice! { call post (
        target_func : FunctionIndex,
        location: Location,
    ) {
        println!("[ANALYSIS:] call post {target_func:#?}, location: {location:?}");
    }
}

advice! { call_indirect pre (
        target_func: FunctionTableIndex,
        func_table_ident: FunctionTable,
        location: Location,
    ) {
        println!("[ANALYSIS:] call_indirect pre {target_func:#?} {func_table_ident:#?}, location: {location:?}");
        target_func
    }
}

advice! { call_indirect post (
        target_func: FunctionTable,
        location: Location,
    ) {
        println!("[ANALYSIS:] call_indirect post {target_func:#?}, location: {location:?}");
    }
}

advice! { unary (
        operator: UnaryOperator,
        operand: WasmValue,
        location: Location,
    ) {
        println!("[ANALYSIS:] unary generic {operator:#?} {operand:#?}, location: {location:?}");
        operator.apply(operand)
    }
}

advice! { binary (
        operator: BinaryOperator,
        l_operand: WasmValue,
        r_operand: WasmValue,
        location: Location,
    ) {
        println!("[ANALYSIS:] binary generic {operator:#?} {l_operand:#?} {r_operand:#?}, location: {location:?}");
        operator.apply(l_operand, r_operand)
    }
}

advice! { drop (
        location: Location,
    ) {
        println!("[ANALYSIS:] Drop called! location: {location:?}");
    }
}

advice! { return_ (
        location: Location,
    ) {
        println!("[ANALYSIS:] Return called! location: {location:?}");
    }
}

advice! { const_ (
        value: WasmValue,
        location: Location,
    ) {
        println!("[ANALYSIS:] const_ generic {value:#?}, location: {location:?}");
        value
    }
}

advice! { local (
        value: WasmValue,
        index: LocalIndex,
        local_op: LocalOp,
        location: Location,
    ) {
        println!("[ANALYSIS:] local generic {value:#?} @ {index:#?} : {local_op:#?}, location: {location:?}");
        value
    }
}

advice! { global (
        value: WasmValue,
        index: GlobalIndex,
        global_op: GlobalOp,
        location: Location,
    ) {
        println!("[ANALYSIS:] global generic {value:#?} @ {index:#?} : {global_op:#?}, location: {location:?}");
        value
    }
}

advice! { load (
        store_index: LoadIndex,
        offset: LoadOffset,
        operation: LoadOperation,
        location: Location,
    ) {
        let value = operation.perform(&store_index, &offset);
        println!("[ANALYSIS:] load generic {operation:#?} @ (CONST {offset:#?} + {store_index:#?}) -> {value:#?}, location: {location:?}");
        value
    }
}

advice! { store (
        store_index: StoreIndex,
        value: WasmValue,
        offset: StoreOffset,
        operation: StoreOperation,
        location: Location,
    ) {
        println!("[ANALYSIS:] store generic {operation:#?} @ (CONST {offset:#?} + {store_index:#?}) <- {value:#?}, location: {location:?}");
        operation.perform(&store_index, &value, &offset);
    }
}

advice! { memory_size (
        size: WasmValue,
        index: MemoryIndex,
        location: Location,
    ) {
        println!("[ANALYSIS:] memory_size {size:#?} @ {index:#?}, location: {location:?}");
        size
    }
}

advice! { memory_grow (
        amount: WasmValue,
        index: MemoryIndex,
        location: Location,
    ) {
        println!("[ANALYSIS:] memory_grow {amount:#?} @ {index:#?}, location: {location:?}");
        index.grow(amount)
    }
}

advice! { block pre (
        block_input_count: BlockInputCount,
        block_arity: BlockArity,
        location: Location,
    ) {
        println!("[ANALYSIS:] block pre [block_input_count: {block_input_count:?}, block_arity: {block_arity:?}], location: {location:?}");
    }
}

advice! { block post (
        location: Location,
    ) {
        println!("[ANALYSIS:] block post, location: {location:?}");
    }
}

advice! { loop_ pre (
        loop_input_count: LoopInputCount,
        loop_arity: LoopArity,
        location: Location,
    ) {
        println!("[ANALYSIS:] loop_ pre [loop_input_count: {loop_input_count:?}, loop_arity: {loop_arity:?}], location: {location:?}");
    }
}

advice! { loop_ post (
        location: Location,
    ) {
        println!("[ANALYSIS:] loop_ post, location: {location:?}");
    }
}

advice! { ref_func (
        func_index: WasmValue,
        location: Location,
    ) {
        println!("[ANALYSIS:] ref_func {func_index:#?}, location: {location:?}");
    }
}

advice! { ref_null (
        location: Location,
    ) {
        println!("[ANALYSIS:] ref_null, location: {location:?}");
    }
}

advice! { ref_is_null (
        res: WasmValue,
        location: Location,
    ) {
        println!("[ANALYSIS:] ref_is_null {res:#?}, location: {location:?}");
        res
    }
}

advice! { table_set (
    element_index: WasmValue,
    table_index: FunctionTableIndex,
    location: Location,
    ) {
        println!("[ANALYSIS:] table_set @ {table_index:#?}[{element_index:#?}], location: {location:?}");
        element_index
    }
}

advice! { table_get (
        element_index: WasmValue,
        table_index: FunctionTableIndex,
        location: Location,
    ) {
        println!("[ANALYSIS:] table_get @ {table_index:#?}[{element_index:#?}], location: {location:?}");
        element_index
    }
}

advice! { table_size (
        table_size: WasmValue,
        table_index: FunctionTableIndex,
        location: Location,
    ) {
        println!("[ANALYSIS:] table_size {table_size:#?} @ {table_index:#?}, location: {location:?}");
        table_size
    }
}

advice! { table_grow (
        grow_size: WasmValue,
        table_index: FunctionTableIndex,
        location: Location,
    ) {
        println!("[ANALYSIS:] table_grow {grow_size:#?} @ {table_index:#?}, location: {location:?}");
        grow_size
    }
}

advice! { table_fill (
        index: WasmValue,
        fill_size: WasmValue,
        table_index: FunctionTableIndex,
        location: Location,
    ) {
        println!("[ANALYSIS:] table_fill starting at {index:#?} with size {fill_size:#?} @ {table_index:#?}, location: {location:?}");
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
        println!("[ANALYSIS:] table_copy {copy_size:#?} elements from {src_table_index:#?}[{src_element_index:#?}] to {dst_table_index:#?}[{dst_element_index:#?}], location: {location:?}");
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
        println!("[ANALYSIS:] table_init {init_size:#?} elements from segment {source_element_offset:#?} at offset {destination_table_offset:#?} to {table_index:#?}[{element_index:#?}], location: {location:?}");
        (destination_table_offset, source_element_offset, init_size)
    }
}

advice! { elem_drop (
        element_index: ElementIndex,
        location: Location,
    ) {
        println!("[ANALYSIS:] elem_drop {element_index:#?}, location: {location:?}");
    }
}
