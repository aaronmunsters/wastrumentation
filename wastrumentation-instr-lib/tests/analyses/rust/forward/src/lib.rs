use wastrumentation_rs_stdlib::{
    advice, BinaryOperator, BranchTableDefault, BranchTableTarget, BranchTargetLabel, Deserialize,
    FunctionIndex, FunctionTable, FunctionTableIndex, GlobalIndex, GlobalOp, LoadIndex, LoadOffset,
    LoadOperation, LocalIndex, LocalOp, MemoryIndex, MutDynArgs, MutDynResults,
    ParameterBrIfCondition, ParameterBrIfLabel, PathContinuation, StoreIndex, StoreOffset,
    StoreOperation, UnaryOperator, WasmFunction, WasmValue,
};

/////
// START ADVICE SPECIFICATION //
//                         /////

advice! { apply (function : WasmFunction, args : MutDynArgs, ress : MutDynResults) {
        let _ = args;
        let _ = ress;
        function.apply();
    }
}

advice! { if_ (path_continuation: PathContinuation) {
        path_continuation
    }
}

advice! { if_then (path_continuation: PathContinuation) {
        path_continuation
    }
}

advice! { br (branch_target_label: BranchTargetLabel) {
    let _ = branch_target_label;
    }
}

advice! { br_if (path_continuation : ParameterBrIfCondition, target_label : ParameterBrIfLabel) {
        let _ = target_label;
        path_continuation
    }
}

advice! { br_table (branch_table_target: BranchTableTarget, branch_table_default: BranchTableDefault) {
        let _ = branch_table_default;
        branch_table_target
    }
}

advice! { select (path_continuation: PathContinuation) {
        path_continuation
    }
}

advice! { call pre (target_func : FunctionIndex) {
        let _ = target_func;
    }
}

advice! { call post (target_func : FunctionIndex) {
        let _ = target_func;
    }
}

advice! { call_indirect pre (target_func: FunctionTableIndex, func_table_ident: FunctionTable) {
        let _ = func_table_ident;
        target_func
    }
}

advice! { call_indirect post (target_func: FunctionTable) {
        let _ = target_func;
    }
}

advice! { unary generic (operator: UnaryOperator, operand: WasmValue) {
        operator.apply(operand)
    }
}

advice! { binary generic ( operator: BinaryOperator, l_operand: WasmValue, r_operand: WasmValue) {
        operator.apply(l_operand, r_operand)
    }
}

advice! { drop () {
    }
}

advice! { return_ () {
    }
}

advice! { const_ generic (value: WasmValue) {
        value
    }
}

advice! { local generic (value: WasmValue, index: LocalIndex, local_op: LocalOp) {
        let _ = index;
        let _ = local_op;
        value
    }
}

advice! { global generic (value: WasmValue, index: GlobalIndex, global_op: GlobalOp) {
        let _ = index;
        let _ = global_op;
        value
    }
}

advice! { load generic (store_index: LoadIndex, offset: LoadOffset, operation: LoadOperation) {
        let value = operation.perform(&store_index, &offset);
        value
    }
}

advice! { store generic (store_index: StoreIndex, value: WasmValue, offset: StoreOffset, operation: StoreOperation) {
        operation.perform(&store_index, &value, &offset);
    }
}

advice! { memory_size (size: WasmValue, index: MemoryIndex) {
        let _ = index;
        size
    }
}

advice! { memory_grow (amount: WasmValue, index: MemoryIndex) {
        index.grow(amount)
    }
}
