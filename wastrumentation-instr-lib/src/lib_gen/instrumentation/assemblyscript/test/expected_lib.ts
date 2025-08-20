
@inline
function allocate_ret_0_arg_0(): usize {
    const stack_begin = stack_allocate_values(0); // inlined
    return stack_begin;
}
@inline
function free_values_ret_0_arg_0(ptr: usize): void {
    stack_deallocate_values(ptr, 0); // inlined
    return;
}
@inline
function store_rets_ret_0_arg_0(stack_ptr: usize): void {
    return;
}
@inline
function allocate_signature_types_buffer_ret_0_arg_0(): usize {
    const stack_begin = stack_allocate_types(0); // inlined
    return stack_begin;
}
@inline
function free_types_ret_0_arg_0(ptr: usize): void {
    stack_deallocate_types(ptr, 0); // inlined
    return;
}
export function allocate_ret_arg(): usize {
    return allocate_ret_0_arg_0();
};

export function free_values_ret_arg(ptr: usize): void {
    return free_values_ret_0_arg_0(ptr);
};
export function store_rets_ret_arg(stack_ptr: usize): void {
    return store_rets_ret_0_arg_0(stack_ptr);
};
export function allocate_types_ret_arg(): usize {
    const types_buffer = allocate_signature_types_buffer_ret_0_arg_0();
    return types_buffer;
}
export function free_types_ret_arg(ptr: usize): void {
    return free_types_ret_0_arg_0(ptr);
};
@inline
function allocate_ret_2_arg_2<R0, R1, T0, T1>(a0: T0, a1: T1): usize {
    const stack_begin = stack_allocate_values(4); // inlined
    // store a0
    wastrumentation_memory_store<T0>(stack_begin, a0, 2);
    // store a1
    wastrumentation_memory_store<T1>(stack_begin, a1, 3);
    return stack_begin;
}
@inline
function load_arg0_ret_2_arg_2<R0, R1, T0, T1>(stack_ptr: usize): T0 {
    return wastrumentation_memory_load<T0>(stack_ptr, 2);
}

@inline
function load_arg1_ret_2_arg_2<R0, R1, T0, T1>(stack_ptr: usize): T1 {
    return wastrumentation_memory_load<T1>(stack_ptr, 3);
}

@inline
function load_ret0_ret_2_arg_2<R0, R1, T0, T1>(stack_ptr: usize): R0 {
    return wastrumentation_memory_load<R0>(stack_ptr, 0);
}

@inline
function load_ret1_ret_2_arg_2<R0, R1, T0, T1>(stack_ptr: usize): R1 {
    return wastrumentation_memory_load<R1>(stack_ptr, 1);
}
@inline
function store_arg0_ret_2_arg_2<R0, R1, T0, T1>(stack_ptr: usize, a0: T0): void {
    return wastrumentation_memory_store<T0>(stack_ptr, a0, 2);
}

@inline
function store_arg1_ret_2_arg_2<R0, R1, T0, T1>(stack_ptr: usize, a1: T1): void {
    return wastrumentation_memory_store<T1>(stack_ptr, a1, 3);
}

@inline
function store_ret0_ret_2_arg_2<R0, R1, T0, T1>(stack_ptr: usize, r0: R0): void {
    return wastrumentation_memory_store<R0>(stack_ptr, r0, 0);
}

@inline
function store_ret1_ret_2_arg_2<R0, R1, T0, T1>(stack_ptr: usize, r1: R1): void {
    return wastrumentation_memory_store<R1>(stack_ptr, r1, 1);
}
@inline
function free_values_ret_2_arg_2<R0, R1, T0, T1>(ptr: usize): void {
    stack_deallocate_values(ptr, 4); // inlined
    return;
}
@inline
function store_rets_ret_2_arg_2<R0, R1, T0, T1>(stack_ptr: usize, a0: R0, a1: R1): void {
    // store a0
    store_ret0_ret_2_arg_2<R0, R1, T0, T1>(stack_ptr, a0);
    // store a1
    store_ret1_ret_2_arg_2<R0, R1, T0, T1>(stack_ptr, a1);
    return;
}
@inline
function allocate_signature_types_buffer_ret_2_arg_2(): usize {
    const stack_begin = stack_allocate_types(4); // inlined
    return stack_begin;
}
@inline
function free_types_ret_2_arg_2(ptr: usize): void {
    stack_deallocate_types(ptr, 4); // inlined
    return;
}
export function allocate_ret_f32_f64_arg_i32_i64(a0: i32, a1: i64): usize {
    return allocate_ret_2_arg_2<f32, f64, i32, i64>(a0, a1);
};

export function load_arg0_ret_f32_f64_arg_i32_i64(stack_ptr: usize): i32 {
    return load_arg0_ret_2_arg_2<f32, f64, i32, i64>(stack_ptr);
};

export function load_arg1_ret_f32_f64_arg_i32_i64(stack_ptr: usize): i64 {
    return load_arg1_ret_2_arg_2<f32, f64, i32, i64>(stack_ptr);
};

export function load_ret0_ret_f32_f64_arg_i32_i64(stack_ptr: usize): f32 {
    return load_ret0_ret_2_arg_2<f32, f64, i32, i64>(stack_ptr);
};

export function load_ret1_ret_f32_f64_arg_i32_i64(stack_ptr: usize): f64 {
    return load_ret1_ret_2_arg_2<f32, f64, i32, i64>(stack_ptr);
};
export function store_arg0_ret_f32_f64_arg_i32_i64(stack_ptr: usize, a0: i32): void {
    return store_arg0_ret_2_arg_2<f32, f64, i32, i64>(stack_ptr, a0);
};

export function store_arg1_ret_f32_f64_arg_i32_i64(stack_ptr: usize, a1: i64): void {
    return store_arg1_ret_2_arg_2<f32, f64, i32, i64>(stack_ptr, a1);
};

export function store_ret0_ret_f32_f64_arg_i32_i64(stack_ptr: usize, a0: f32): void {
    return store_ret0_ret_2_arg_2<f32, f64, i32, i64>(stack_ptr, a0);
};

export function store_ret1_ret_f32_f64_arg_i32_i64(stack_ptr: usize, a1: f64): void {
    return store_ret1_ret_2_arg_2<f32, f64, i32, i64>(stack_ptr, a1);
};
export function free_values_ret_f32_f64_arg_i32_i64(ptr: usize): void {
    return free_values_ret_2_arg_2<f32, f64, i32, i64>(ptr);
};
export function store_rets_ret_f32_f64_arg_i32_i64(stack_ptr: usize, a0: f32, a1: f64): void {
    return store_rets_ret_2_arg_2<f32, f64, i32, i64>(stack_ptr, a0, a1);
};
export function allocate_types_ret_f32_f64_arg_i32_i64(): usize {
    const types_buffer = allocate_signature_types_buffer_ret_2_arg_2();
    wastrumentation_stack_store_type(types_buffer, 0, 1);
    wastrumentation_stack_store_type(types_buffer, 1, 3);
    wastrumentation_stack_store_type(types_buffer, 2, 0);
    wastrumentation_stack_store_type(types_buffer, 3, 2);
    return types_buffer;
}
export function free_types_ret_f32_f64_arg_i32_i64(ptr: usize): void {
    return free_types_ret_2_arg_2(ptr);
};
export function allocate_ret_f64_f32_arg_i32_i64(a0: i32, a1: i64): usize {
    return allocate_ret_2_arg_2<f64, f32, i32, i64>(a0, a1);
};

export function load_arg0_ret_f64_f32_arg_i32_i64(stack_ptr: usize): i32 {
    return load_arg0_ret_2_arg_2<f64, f32, i32, i64>(stack_ptr);
};

export function load_arg1_ret_f64_f32_arg_i32_i64(stack_ptr: usize): i64 {
    return load_arg1_ret_2_arg_2<f64, f32, i32, i64>(stack_ptr);
};

export function load_ret0_ret_f64_f32_arg_i32_i64(stack_ptr: usize): f64 {
    return load_ret0_ret_2_arg_2<f64, f32, i32, i64>(stack_ptr);
};

export function load_ret1_ret_f64_f32_arg_i32_i64(stack_ptr: usize): f32 {
    return load_ret1_ret_2_arg_2<f64, f32, i32, i64>(stack_ptr);
};
export function store_arg0_ret_f64_f32_arg_i32_i64(stack_ptr: usize, a0: i32): void {
    return store_arg0_ret_2_arg_2<f64, f32, i32, i64>(stack_ptr, a0);
};

export function store_arg1_ret_f64_f32_arg_i32_i64(stack_ptr: usize, a1: i64): void {
    return store_arg1_ret_2_arg_2<f64, f32, i32, i64>(stack_ptr, a1);
};

export function store_ret0_ret_f64_f32_arg_i32_i64(stack_ptr: usize, a0: f64): void {
    return store_ret0_ret_2_arg_2<f64, f32, i32, i64>(stack_ptr, a0);
};

export function store_ret1_ret_f64_f32_arg_i32_i64(stack_ptr: usize, a1: f32): void {
    return store_ret1_ret_2_arg_2<f64, f32, i32, i64>(stack_ptr, a1);
};
export function free_values_ret_f64_f32_arg_i32_i64(ptr: usize): void {
    return free_values_ret_2_arg_2<f64, f32, i32, i64>(ptr);
};
export function store_rets_ret_f64_f32_arg_i32_i64(stack_ptr: usize, a0: f64, a1: f32): void {
    return store_rets_ret_2_arg_2<f64, f32, i32, i64>(stack_ptr, a0, a1);
};
export function allocate_types_ret_f64_f32_arg_i32_i64(): usize {
    const types_buffer = allocate_signature_types_buffer_ret_2_arg_2();
    wastrumentation_stack_store_type(types_buffer, 0, 3);
    wastrumentation_stack_store_type(types_buffer, 1, 1);
    wastrumentation_stack_store_type(types_buffer, 2, 0);
    wastrumentation_stack_store_type(types_buffer, 3, 2);
    return types_buffer;
}
export function free_types_ret_f64_f32_arg_i32_i64(ptr: usize): void {
    return free_types_ret_2_arg_2(ptr);
};
@inline
function allocate_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(a0: T0, a1: T1, a2: T2, a3: T3): usize {
    const stack_begin = stack_allocate_values(8); // inlined
    // store a0
    wastrumentation_memory_store<T0>(stack_begin, a0, 4);
    // store a1
    wastrumentation_memory_store<T1>(stack_begin, a1, 5);
    // store a2
    wastrumentation_memory_store<T2>(stack_begin, a2, 6);
    // store a3
    wastrumentation_memory_store<T3>(stack_begin, a3, 7);
    return stack_begin;
}
@inline
function load_arg0_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize): T0 {
    return wastrumentation_memory_load<T0>(stack_ptr, 4);
}

@inline
function load_arg1_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize): T1 {
    return wastrumentation_memory_load<T1>(stack_ptr, 5);
}

@inline
function load_arg2_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize): T2 {
    return wastrumentation_memory_load<T2>(stack_ptr, 6);
}

@inline
function load_arg3_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize): T3 {
    return wastrumentation_memory_load<T3>(stack_ptr, 7);
}

@inline
function load_ret0_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize): R0 {
    return wastrumentation_memory_load<R0>(stack_ptr, 0);
}

@inline
function load_ret1_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize): R1 {
    return wastrumentation_memory_load<R1>(stack_ptr, 1);
}

@inline
function load_ret2_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize): R2 {
    return wastrumentation_memory_load<R2>(stack_ptr, 2);
}

@inline
function load_ret3_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize): R3 {
    return wastrumentation_memory_load<R3>(stack_ptr, 3);
}
@inline
function store_arg0_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize, a0: T0): void {
    return wastrumentation_memory_store<T0>(stack_ptr, a0, 4);
}

@inline
function store_arg1_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize, a1: T1): void {
    return wastrumentation_memory_store<T1>(stack_ptr, a1, 5);
}

@inline
function store_arg2_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize, a2: T2): void {
    return wastrumentation_memory_store<T2>(stack_ptr, a2, 6);
}

@inline
function store_arg3_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize, a3: T3): void {
    return wastrumentation_memory_store<T3>(stack_ptr, a3, 7);
}

@inline
function store_ret0_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize, r0: R0): void {
    return wastrumentation_memory_store<R0>(stack_ptr, r0, 0);
}

@inline
function store_ret1_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize, r1: R1): void {
    return wastrumentation_memory_store<R1>(stack_ptr, r1, 1);
}

@inline
function store_ret2_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize, r2: R2): void {
    return wastrumentation_memory_store<R2>(stack_ptr, r2, 2);
}

@inline
function store_ret3_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize, r3: R3): void {
    return wastrumentation_memory_store<R3>(stack_ptr, r3, 3);
}
@inline
function free_values_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(ptr: usize): void {
    stack_deallocate_values(ptr, 8); // inlined
    return;
}
@inline
function store_rets_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize, a0: R0, a1: R1, a2: R2, a3: R3): void {
    // store a0
    store_ret0_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr, a0);
    // store a1
    store_ret1_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr, a1);
    // store a2
    store_ret2_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr, a2);
    // store a3
    store_ret3_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr, a3);
    return;
}
@inline
function allocate_signature_types_buffer_ret_4_arg_4(): usize {
    const stack_begin = stack_allocate_types(8); // inlined
    return stack_begin;
}
@inline
function free_types_ret_4_arg_4(ptr: usize): void {
    stack_deallocate_types(ptr, 8); // inlined
    return;
}
export function allocate_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(a0: i64, a1: i32, a2: f32, a3: f64): usize {
    return allocate_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(a0, a1, a2, a3);
};

export function load_arg0_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize): i64 {
    return load_arg0_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
};

export function load_arg1_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize): i32 {
    return load_arg1_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
};

export function load_arg2_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize): f32 {
    return load_arg2_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
};

export function load_arg3_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize): f64 {
    return load_arg3_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
};

export function load_ret0_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize): f64 {
    return load_ret0_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
};

export function load_ret1_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize): f32 {
    return load_ret1_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
};

export function load_ret2_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize): i32 {
    return load_ret2_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
};

export function load_ret3_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize): i64 {
    return load_ret3_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
};
export function store_arg0_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a0: i64): void {
    return store_arg0_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a0);
};

export function store_arg1_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a1: i32): void {
    return store_arg1_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a1);
};

export function store_arg2_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a2: f32): void {
    return store_arg2_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a2);
};

export function store_arg3_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a3: f64): void {
    return store_arg3_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a3);
};

export function store_ret0_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a0: f64): void {
    return store_ret0_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a0);
};

export function store_ret1_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a1: f32): void {
    return store_ret1_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a1);
};

export function store_ret2_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a2: i32): void {
    return store_ret2_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a2);
};

export function store_ret3_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a3: i64): void {
    return store_ret3_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a3);
};
export function free_values_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(ptr: usize): void {
    return free_values_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(ptr);
};
export function store_rets_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a0: f64, a1: f32, a2: i32, a3: i64): void {
    return store_rets_ret_4_arg_4<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a0, a1, a2, a3);
};
export function allocate_types_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(): usize {
    const types_buffer = allocate_signature_types_buffer_ret_4_arg_4();
    wastrumentation_stack_store_type(types_buffer, 0, 3);
    wastrumentation_stack_store_type(types_buffer, 1, 1);
    wastrumentation_stack_store_type(types_buffer, 2, 0);
    wastrumentation_stack_store_type(types_buffer, 3, 2);
    wastrumentation_stack_store_type(types_buffer, 4, 2);
    wastrumentation_stack_store_type(types_buffer, 5, 0);
    wastrumentation_stack_store_type(types_buffer, 6, 1);
    wastrumentation_stack_store_type(types_buffer, 7, 3);
    return types_buffer;
}
export function free_types_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(ptr: usize): void {
    return free_types_ret_4_arg_4(ptr);
};
