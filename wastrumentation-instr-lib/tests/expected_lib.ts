
@inline
function allocate_ret_0_arg_0(): usize {
    const to_allocate = 0; // constant folded
    const stack_begin = stack_allocate(to_allocate); // inlined
    return stack_begin;
}
@inline
function free_values_ret_0_arg_0(ptr: usize): void {
    const to_deallocate = 0; // constant folded
    stack_deallocate(ptr, to_deallocate); // inlined
    return;
}
@inline
function store_rets_ret_0_arg_0(stack_ptr: usize): void {
    return;
}
@inline
function allocate_signature_types_buffer_ret_0_arg_0(): usize {
    const to_allocate = sizeof<i32>() * 0;; // constant folded
    const stack_begin = stack_allocate(to_allocate); // inlined
    return stack_begin;
}
@inline
function free_types_ret_0_arg_0(ptr: usize): void {
    const to_deallocate = sizeof<i32>() * 0;; // constant folded
    stack_deallocate(ptr, to_deallocate); // inlined
    return;
}
export function allocate_ret__arg_(): usize {
    return allocate_ret_0_arg_0();
};

export function free_values_ret__arg_(ptr: usize): void {
    return free_values_ret_0_arg_0(ptr);
};
export function store_rets_ret__arg_(stack_ptr: usize): void {
    return store_rets_ret_0_arg_0(stack_ptr);
};
export function allocate_types_ret__arg_(): usize {
    const types_buffer = allocate_signature_types_buffer_ret_0_arg_0();
    return types_buffer;
}
export function free_types_ret__arg_(ptr: usize): void {
    return free_types_ret_0_arg_0(ptr);
};
@inline
function allocate_ret_2_arg_2<R0, R1, T0, T1>(a0: T0, a1: T1): usize {
    const to_allocate = sizeof<R0>() + sizeof<R1>() + sizeof<T0>() + sizeof<T1>(); // constant folded
    const stack_begin = stack_allocate(to_allocate); // inlined
    // store a0
    const a0_offset = sizeof<R0>() + sizeof<R1>(); // constant folded
    wastrumentation_memory_store<T0>(stack_begin, a0, a0_offset); // inlined
    // store a1
    const a1_offset = sizeof<R0>() + sizeof<R1>() + sizeof<T0>(); // constant folded
    wastrumentation_memory_store<T1>(stack_begin, a1, a1_offset); // inlined
    return stack_begin;
}
@inline
function load_arg0_ret_2_arg_2<R0, R1, T0, T1>(stack_ptr: usize): T0 {
    const a0_offset = sizeof<R0>() + sizeof<R1>(); // constant folded
    return wastrumentation_memory_load<T0>(stack_ptr, a0_offset); // inlined
}

@inline
function load_arg1_ret_2_arg_2<R0, R1, T0, T1>(stack_ptr: usize): T1 {
    const a1_offset = sizeof<R0>() + sizeof<R1>() + sizeof<T0>(); // constant folded
    return wastrumentation_memory_load<T1>(stack_ptr, a1_offset); // inlined
}

@inline
function load_ret0_ret_2_arg_2<R0, R1, T0, T1>(stack_ptr: usize): R0 {
    const r0_offset = 0; // constant folded
    return wastrumentation_memory_load<R0>(stack_ptr, r0_offset); // inlined
}

@inline
function load_ret1_ret_2_arg_2<R0, R1, T0, T1>(stack_ptr: usize): R1 {
    const r1_offset = sizeof<R0>(); // constant folded
    return wastrumentation_memory_load<R1>(stack_ptr, r1_offset); // inlined
}
@inline
function store_arg0_ret_2_arg_2<R0, R1, T0, T1>(stack_ptr: usize, a0: T0): void {
    const a0_offset = sizeof<R0>() + sizeof<R1>(); // constant folded
    return wastrumentation_memory_store<T0>(stack_ptr, a0, a0_offset); // inlined
}

@inline
function store_arg1_ret_2_arg_2<R0, R1, T0, T1>(stack_ptr: usize, a1: T1): void {
    const a1_offset = sizeof<R0>() + sizeof<R1>() + sizeof<T0>(); // constant folded
    return wastrumentation_memory_store<T1>(stack_ptr, a1, a1_offset); // inlined
}

@inline
function store_ret0_ret_2_arg_2<R0, R1, T0, T1>(stack_ptr: usize, r0: R0): void {
    const r0_offset = 0; // constant folded
    return wastrumentation_memory_store<R0>(stack_ptr, r0, r0_offset); // inlined
}

@inline
function store_ret1_ret_2_arg_2<R0, R1, T0, T1>(stack_ptr: usize, r1: R1): void {
    const r1_offset = sizeof<R0>(); // constant folded
    return wastrumentation_memory_store<R1>(stack_ptr, r1, r1_offset); // inlined
}
@inline
function free_values_ret_2_arg_2<R0, R1, T0, T1>(ptr: usize): void {
    const to_deallocate = sizeof<R0>() + sizeof<R1>() + sizeof<T0>() + sizeof<T1>(); // constant folded
    stack_deallocate(ptr, to_deallocate); // inlined
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
    const to_allocate = sizeof<i32>() * 4;; // constant folded
    const stack_begin = stack_allocate(to_allocate); // inlined
    return stack_begin;
}
@inline
function free_types_ret_2_arg_2(ptr: usize): void {
    const to_deallocate = sizeof<i32>() * 4;; // constant folded
    stack_deallocate(ptr, to_deallocate); // inlined
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
    wastrumentation_memory_store<i32>(types_buffer, 1, (sizeof<i32>()*0));
    wastrumentation_memory_store<i32>(types_buffer, 3, (sizeof<i32>()*1));
    wastrumentation_memory_store<i32>(types_buffer, 0, (sizeof<i32>()*2));
    wastrumentation_memory_store<i32>(types_buffer, 2, (sizeof<i32>()*3));
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
    wastrumentation_memory_store<i32>(types_buffer, 3, (sizeof<i32>()*0));
    wastrumentation_memory_store<i32>(types_buffer, 1, (sizeof<i32>()*1));
    wastrumentation_memory_store<i32>(types_buffer, 0, (sizeof<i32>()*2));
    wastrumentation_memory_store<i32>(types_buffer, 2, (sizeof<i32>()*3));
    return types_buffer;
}
export function free_types_ret_f64_f32_arg_i32_i64(ptr: usize): void {
    return free_types_ret_2_arg_2(ptr);
};
@inline
function allocate_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(a0: T0, a1: T1, a2: T2, a3: T3): usize {
    const to_allocate = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<T0>() + sizeof<T1>() + sizeof<T2>() + sizeof<T3>(); // constant folded
    const stack_begin = stack_allocate(to_allocate); // inlined
    // store a0
    const a0_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>(); // constant folded
    wastrumentation_memory_store<T0>(stack_begin, a0, a0_offset); // inlined
    // store a1
    const a1_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<T0>(); // constant folded
    wastrumentation_memory_store<T1>(stack_begin, a1, a1_offset); // inlined
    // store a2
    const a2_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<T0>() + sizeof<T1>(); // constant folded
    wastrumentation_memory_store<T2>(stack_begin, a2, a2_offset); // inlined
    // store a3
    const a3_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<T0>() + sizeof<T1>() + sizeof<T2>(); // constant folded
    wastrumentation_memory_store<T3>(stack_begin, a3, a3_offset); // inlined
    return stack_begin;
}
@inline
function load_arg0_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize): T0 {
    const a0_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>(); // constant folded
    return wastrumentation_memory_load<T0>(stack_ptr, a0_offset); // inlined
}

@inline
function load_arg1_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize): T1 {
    const a1_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<T0>(); // constant folded
    return wastrumentation_memory_load<T1>(stack_ptr, a1_offset); // inlined
}

@inline
function load_arg2_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize): T2 {
    const a2_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<T0>() + sizeof<T1>(); // constant folded
    return wastrumentation_memory_load<T2>(stack_ptr, a2_offset); // inlined
}

@inline
function load_arg3_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize): T3 {
    const a3_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<T0>() + sizeof<T1>() + sizeof<T2>(); // constant folded
    return wastrumentation_memory_load<T3>(stack_ptr, a3_offset); // inlined
}

@inline
function load_ret0_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize): R0 {
    const r0_offset = 0; // constant folded
    return wastrumentation_memory_load<R0>(stack_ptr, r0_offset); // inlined
}

@inline
function load_ret1_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize): R1 {
    const r1_offset = sizeof<R0>(); // constant folded
    return wastrumentation_memory_load<R1>(stack_ptr, r1_offset); // inlined
}

@inline
function load_ret2_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize): R2 {
    const r2_offset = sizeof<R0>() + sizeof<R1>(); // constant folded
    return wastrumentation_memory_load<R2>(stack_ptr, r2_offset); // inlined
}

@inline
function load_ret3_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize): R3 {
    const r3_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>(); // constant folded
    return wastrumentation_memory_load<R3>(stack_ptr, r3_offset); // inlined
}
@inline
function store_arg0_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize, a0: T0): void {
    const a0_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>(); // constant folded
    return wastrumentation_memory_store<T0>(stack_ptr, a0, a0_offset); // inlined
}

@inline
function store_arg1_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize, a1: T1): void {
    const a1_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<T0>(); // constant folded
    return wastrumentation_memory_store<T1>(stack_ptr, a1, a1_offset); // inlined
}

@inline
function store_arg2_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize, a2: T2): void {
    const a2_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<T0>() + sizeof<T1>(); // constant folded
    return wastrumentation_memory_store<T2>(stack_ptr, a2, a2_offset); // inlined
}

@inline
function store_arg3_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize, a3: T3): void {
    const a3_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<T0>() + sizeof<T1>() + sizeof<T2>(); // constant folded
    return wastrumentation_memory_store<T3>(stack_ptr, a3, a3_offset); // inlined
}

@inline
function store_ret0_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize, r0: R0): void {
    const r0_offset = 0; // constant folded
    return wastrumentation_memory_store<R0>(stack_ptr, r0, r0_offset); // inlined
}

@inline
function store_ret1_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize, r1: R1): void {
    const r1_offset = sizeof<R0>(); // constant folded
    return wastrumentation_memory_store<R1>(stack_ptr, r1, r1_offset); // inlined
}

@inline
function store_ret2_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize, r2: R2): void {
    const r2_offset = sizeof<R0>() + sizeof<R1>(); // constant folded
    return wastrumentation_memory_store<R2>(stack_ptr, r2, r2_offset); // inlined
}

@inline
function store_ret3_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(stack_ptr: usize, r3: R3): void {
    const r3_offset = sizeof<R0>() + sizeof<R1>() + sizeof<R2>(); // constant folded
    return wastrumentation_memory_store<R3>(stack_ptr, r3, r3_offset); // inlined
}
@inline
function free_values_ret_4_arg_4<R0, R1, R2, R3, T0, T1, T2, T3>(ptr: usize): void {
    const to_deallocate = sizeof<R0>() + sizeof<R1>() + sizeof<R2>() + sizeof<R3>() + sizeof<T0>() + sizeof<T1>() + sizeof<T2>() + sizeof<T3>(); // constant folded
    stack_deallocate(ptr, to_deallocate); // inlined
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
    const to_allocate = sizeof<i32>() * 8;; // constant folded
    const stack_begin = stack_allocate(to_allocate); // inlined
    return stack_begin;
}
@inline
function free_types_ret_4_arg_4(ptr: usize): void {
    const to_deallocate = sizeof<i32>() * 8;; // constant folded
    stack_deallocate(ptr, to_deallocate); // inlined
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
    wastrumentation_memory_store<i32>(types_buffer, 3, (sizeof<i32>()*0));
    wastrumentation_memory_store<i32>(types_buffer, 1, (sizeof<i32>()*1));
    wastrumentation_memory_store<i32>(types_buffer, 0, (sizeof<i32>()*2));
    wastrumentation_memory_store<i32>(types_buffer, 2, (sizeof<i32>()*3));
    wastrumentation_memory_store<i32>(types_buffer, 2, (sizeof<i32>()*4));
    wastrumentation_memory_store<i32>(types_buffer, 0, (sizeof<i32>()*5));
    wastrumentation_memory_store<i32>(types_buffer, 1, (sizeof<i32>()*6));
    wastrumentation_memory_store<i32>(types_buffer, 3, (sizeof<i32>()*7));
    return types_buffer;
}
export function free_types_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(ptr: usize): void {
    return free_types_ret_4_arg_4(ptr);
};
