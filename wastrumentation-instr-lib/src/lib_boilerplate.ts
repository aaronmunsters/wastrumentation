
let $STACK_FREE_PTR: usize = 0;
let $TOTAL_MEMORY: usize = 0;
const $PAGE_BYTE_SIZE: usize = 65536;
const $MEMORY_GROWTH_SIZE: usize = 1;

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
function stack_allocate(bytes: usize): usize {
    const stack_free_ptr_before_alloc = $STACK_FREE_PTR;
    const stack_free_ptr_after_alloc = $STACK_FREE_PTR + bytes;
    let total_memory_in_bytes = $TOTAL_MEMORY * $PAGE_BYTE_SIZE;

    while (stack_free_ptr_after_alloc > total_memory_in_bytes) {
        grow_memory();
        total_memory_in_bytes = $TOTAL_MEMORY * $PAGE_BYTE_SIZE;
    }

    $STACK_FREE_PTR = stack_free_ptr_after_alloc;
    return stack_free_ptr_before_alloc;
}

@inline
function stack_deallocate(bytes: usize): void {
    $STACK_FREE_PTR =- bytes;
}

@inline
export function wastrumentation_stack_load_i32(ptr: usize): i32 {
    return load<i32>(ptr);
};
@inline
export function wastrumentation_stack_load_f32(ptr: usize): f32 {
    return load<f32>(ptr);
};
@inline
export function wastrumentation_stack_load_i64(ptr: usize): i64 {
    return load<i64>(ptr);
};
@inline
export function wastrumentation_stack_load_f64(ptr: usize): f64 {
    return load<f64>(ptr);
};

@inline
export function wastrumentation_stack_store_i32(ptr: usize, value: i32): void {
    return store<i32>(ptr, value);
};
@inline
export function wastrumentation_stack_store_f32(ptr: usize, value: f32): void {
    return store<f32>(ptr, value);
};
@inline
export function wastrumentation_stack_store_i64(ptr: usize, value: i64): void {
    return store<i64>(ptr, value);
};
@inline
export function wastrumentation_stack_store_f64(ptr: usize, value: f64): void {
    return store<f64>(ptr, value);
};

function wastrumentation_memory_load<T>(ptr: usize, offset: usize): T {
    if (false) { unreachable(); }
    else if (sizeof<T>() == 4 && isInteger<T>())
        return wastrumentation_stack_load_i32(ptr + offset);
    else if (sizeof<T>() == 4 && isFloat<T>())
        return wastrumentation_stack_load_f32(ptr + offset);
    else if (sizeof<T>() == 8 && isInteger<T>())
        return wastrumentation_stack_load_i64(ptr + offset);
    else if (sizeof<T>() == 8 && isFloat<T>())
        return wastrumentation_stack_load_f64(ptr + offset);
    unreachable();
}

function wastrumentation_memory_store<T>(ptr: usize, value: T, offset: usize): void {
    if (false) { unreachable(); }
    else if (sizeof<T>() == 4 && isInteger<T>())
        return wastrumentation_stack_store_i32(ptr + offset, value);
    else if (sizeof<T>() == 4 && isFloat<T>())
        return wastrumentation_stack_store_f32(ptr + offset, value);
    else if (sizeof<T>() == 8 && isInteger<T>())
        return wastrumentation_stack_store_i64(ptr + offset, value);
    else if (sizeof<T>() == 8 && isFloat<T>())
        return wastrumentation_stack_store_f64(ptr + offset, value);
    unreachable();
}
