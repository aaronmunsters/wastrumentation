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
        println!("[ANALYSIS:] apply (pre) {function:#?}({args:#?})");
        function.apply();
        println!("[ANALYSIS:] apply (post) {function:#?}({args:#?}) = {ress:#?}");
    }
}

advice! { if_ (path_continuation: PathContinuation) {
        println!("[ANALYSIS:] if_ {path_continuation:#?}");
        path_continuation
    }
}

advice! { if_then (path_continuation: PathContinuation) {
        println!("[ANALYSIS:] if_then {path_continuation:#?}");
        path_continuation
    }
}

advice! { br (branch_target_label: BranchTargetLabel) {
        println!("[ANALYSIS:] br {branch_target_label:#?}");
    }
}

advice! { br_if (path_continuation : ParameterBrIfCondition, target_label : ParameterBrIfLabel) {
        println!("[ANALYSIS:] br_if {path_continuation:#?} to {target_label:#?}");
        path_continuation
    }
}

advice! { br_table (branch_table_target: BranchTableTarget, branch_table_default: BranchTableDefault) {
        println!("[ANALYSIS:] br_table {branch_table_target:#?} (default: {branch_table_default:#?})");
        branch_table_target
    }
}

advice! { select (path_continuation: PathContinuation) {
        println!("[ANALYSIS:] select {path_continuation:#?}");
        path_continuation
    }
}

advice! { call pre (target_func : FunctionIndex) {
        println!("[ANALYSIS:] call pre {target_func:#?}");
    }
}

advice! { call post (target_func : FunctionIndex) {
        println!("[ANALYSIS:] call post {target_func:#?}");
    }
}

advice! { call_indirect pre (target_func: FunctionTableIndex, func_table_ident: FunctionTable) {
        println!("[ANALYSIS:] call_indirect pre {target_func:#?} {func_table_ident:#?}");
        target_func
    }
}

advice! { call_indirect post (target_func: FunctionTable) {
        println!("[ANALYSIS:] call_indirect post {target_func:#?}");
    }
}

advice! { unary generic (operator: UnaryOperator, operand: WasmValue) {
        println!("[ANALYSIS:] unary generic {operator:#?} {operand:#?}");
        operator.apply(operand)
    }
}

advice! { binary generic ( operator: BinaryOperator, l_operand: WasmValue, r_operand: WasmValue) {
        println!("[ANALYSIS:] binary generic {operator:#?} {l_operand:#?} {r_operand:#?}");
        operator.apply(l_operand, r_operand)
    }
}

advice! { drop () {
        println!("[ANALYSIS:] Drop called!");
    }
}

advice! { return_ () {
        println!("[ANALYSIS:] Return called!");
    }
}

advice! { const_ generic (value: WasmValue) {
        println!("[ANALYSIS:] const_ generic {value:#?}");
        value
    }
}

advice! { local generic (value: WasmValue, index: LocalIndex, local_op: LocalOp) {
        println!("[ANALYSIS:] local generic {value:#?} @ {index:#?} : {local_op:#?}");
        value
    }
}

advice! { global generic (value: WasmValue, index: GlobalIndex, global_op: GlobalOp) {
        println!("[ANALYSIS:] global generic {value:#?} @ {index:#?} : {global_op:#?}");
        value
    }
}

advice! { load generic (store_index: LoadIndex, offset: LoadOffset, operation: LoadOperation) {
        let value = operation.perform(&store_index, &offset);
        println!("[ANALYSIS:] load generic {operation:#?} @ (CONST {offset:#?} + {store_index:#?}) -> {value:#?}");
        value
    }
}

advice! { store generic (store_index: StoreIndex, value: WasmValue, offset: StoreOffset, operation: StoreOperation) {
        println!("[ANALYSIS:] store generic {operation:#?} @ (CONST {offset:#?} + {store_index:#?}) <- {value:#?}");
        operation.perform(&store_index, &value, &offset);
    }
}

advice! { memory_size (size: WasmValue, index: MemoryIndex) {
        println!("[ANALYSIS:] memory_size {size:#?} @ {index:#?}");
        size
    }
}

advice! { memory_grow (amount: WasmValue, index: MemoryIndex) {
        println!("[ANALYSIS:] memory_grow {amount:#?} @ {index:#?}");
        index.grow(amount)
    }
}
