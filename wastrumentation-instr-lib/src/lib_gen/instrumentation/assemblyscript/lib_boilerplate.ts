let $STACK_FREE_PTR: usize = 0;
let $TOTAL_MEMORY: usize = 0;
const $MEMORY_GROWTH_SIZE: usize = 1;
const SIZE_WASM_VALUE: usize = sizeof<i64>();
const SIZE_WASM_TYPE: usize = sizeof<i32>();

@inline
function grow_memory(): void {
    // Grow by constant size
    if (memory.grow(($MEMORY_GROWTH_SIZE as i32)) == -1) {
        // If growth failed, trap
        unreachable()
    } else {
        // If growth did not fail, adjust house keeping
        $TOTAL_MEMORY += $MEMORY_GROWTH_SIZE;
    }
}

@inline
function total_used_memory_in_bytes(): usize {
    const $PAGE_BYTE_SIZE: usize = 65536;
    return $TOTAL_MEMORY * $PAGE_BYTE_SIZE;
}

@inline
function stack_allocate(bytes: usize): usize {
    const stack_free_ptr_before_alloc = $STACK_FREE_PTR;
    const stack_free_ptr_after_alloc = $STACK_FREE_PTR + bytes;
    while (stack_free_ptr_after_alloc > total_used_memory_in_bytes()) grow_memory();
    $STACK_FREE_PTR = stack_free_ptr_after_alloc;
    return stack_free_ptr_before_alloc;
}

@inline
function stack_deallocate(_ptr: usize, bytes: usize): void {
    $STACK_FREE_PTR -= bytes;
}

@inline
function stack_allocate_values(count: usize): usize {
    return stack_allocate(count * SIZE_WASM_VALUE);
}

@inline
function stack_deallocate_values(_ptr: usize, count: usize): void {
    stack_deallocate(_ptr, count * SIZE_WASM_VALUE);
}

@inline
function stack_allocate_types(count: usize): usize {
    return stack_allocate(count * SIZE_WASM_TYPE);
}

@inline
function stack_deallocate_types(_ptr: usize, count: usize): void {
    stack_deallocate(_ptr, count * SIZE_WASM_TYPE);
}

@inline
export function wastrumentation_stack_load_type(ptr: usize, offset: usize): i32 {
    return load<i32>(ptr + (offset * SIZE_WASM_TYPE), 0);
};

@inline
export function wastrumentation_stack_store_type(ptr: usize, offset: usize, ty: i32): void {
    store<i32>(ptr + (offset * SIZE_WASM_TYPE), ty);
};

@inline
export function wastrumentation_stack_load_i32(ptr: usize, offset: usize): i32 {
    return load<i32>(ptr + (offset * SIZE_WASM_VALUE), 0);
};
@inline
export function wastrumentation_stack_load_f32(ptr: usize, offset: usize): f32 {
    return load<f32>(ptr + (offset * SIZE_WASM_VALUE), 0);
};
@inline
export function wastrumentation_stack_load_i64(ptr: usize, offset: usize): i64 {
    return load<i64>(ptr + (offset * SIZE_WASM_VALUE), 0);
};
@inline
export function wastrumentation_stack_load_f64(ptr: usize, offset: usize): f64 {
    return load<f64>(ptr + (offset * SIZE_WASM_VALUE), 0);
};

@inline
export function wastrumentation_stack_store_i32(ptr: usize, value: i32, offset: usize): void {
    return store<i32>(ptr + (offset * SIZE_WASM_VALUE), value);
};
@inline
export function wastrumentation_stack_store_f32(ptr: usize, value: f32, offset: usize): void {
    return store<f32>(ptr + (offset * SIZE_WASM_VALUE), value);
};
@inline
export function wastrumentation_stack_store_i64(ptr: usize, value: i64, offset: usize): void {
    return store<i64>(ptr + (offset * SIZE_WASM_VALUE), value);
};
@inline
export function wastrumentation_stack_store_f64(ptr: usize, value: f64, offset: usize): void {
    return store<f64>(ptr + (offset * SIZE_WASM_VALUE), value);
};

function wastrumentation_memory_load<T>(ptr: usize, offset: usize): T {
    if (false) { unreachable(); }
    else if (sizeof<T>() == 4 && isInteger<T>())
        return wastrumentation_stack_load_i32(ptr, offset);
    else if (sizeof<T>() == 4 && isFloat<T>())
        return wastrumentation_stack_load_f32(ptr, offset);
    else if (sizeof<T>() == 8 && isInteger<T>())
        return wastrumentation_stack_load_i64(ptr, offset);
    else if (sizeof<T>() == 8 && isFloat<T>())
        return wastrumentation_stack_load_f64(ptr, offset);
    unreachable();
}

function wastrumentation_memory_store<T>(ptr: usize, value: T, offset: usize): void {
    if (false) { unreachable(); }
    else if (sizeof<T>() == 4 && isInteger<T>())
        return wastrumentation_stack_store_i32(ptr, value, offset);
    else if (sizeof<T>() == 4 && isFloat<T>())
        return wastrumentation_stack_store_f32(ptr, value, offset);
    else if (sizeof<T>() == 8 && isInteger<T>())
        return wastrumentation_stack_store_i64(ptr, value, offset);
    else if (sizeof<T>() == 8 && isFloat<T>())
        return wastrumentation_stack_store_f64(ptr, value, offset);
    unreachable();
}
