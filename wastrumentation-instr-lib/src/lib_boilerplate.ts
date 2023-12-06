
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

export function wastrumentation_stack_load_i32(ptr: i32): i32 {
    return load<i32>(ptr);
};
export function wastrumentation_stack_load_f32(ptr: i32): f32 {
    return load<f32>(ptr);
};
export function wastrumentation_stack_load_i64(ptr: i32): i64 {
    return load<i64>(ptr);
};
export function wastrumentation_stack_load_f64(ptr: i32): f64 {
    return load<f64>(ptr);
};

export function wastrumentation_stack_store_i32(ptr: i32, value: i32): void {
    return store<i32>(ptr, value);
};
export function wastrumentation_stack_store_f32(ptr: i32, value: f32): void {
    return store<f32>(ptr, value);
};
export function wastrumentation_stack_store_i64(ptr: i32, value: i64): void {
    return store<i64>(ptr, value);
};
export function wastrumentation_stack_store_f64(ptr: i32, value: f64): void {
    return store<f64>(ptr, value);
};

