const allocations: Array<ArrayBuffer> = new Array<ArrayBuffer>();

function stack_allocate(bytes: usize): usize {
    let allocation = new ArrayBuffer(bytes as i32);
    let pointer = (allocations.push(allocation) - 1);
    console.log(`stack_allocate[*frame=${pointer}, len(frame)=${bytes}]`);
    return pointer;
}

function stack_deallocate(bytes: usize): void {
    // todo... get 'free' pointer and dealloc!
}

export function wastrumentation_stack_load_i32(ptr: usize, offset: usize): i32 {
    console.log(`load_i32[*frame=${ptr}, offset=${offset}, len(frame)=${allocations[ptr as i32].byteLength}]`);
    return Int32Array.wrap(allocations[ptr as i32], offset as i32, 1)[0];
};
export function wastrumentation_stack_load_f32(ptr: usize, offset: usize): f32 {
    console.log(`load_f32[*frame=${ptr}, offset=${offset}, len(frame)=${allocations[ptr as i32].byteLength}]`);
    return Float32Array.wrap(allocations[ptr as i32], offset as i32, 1)[0];
};
export function wastrumentation_stack_load_i64(ptr: usize, offset: usize): i64 {
    console.log(`load_i64[*frame=${ptr}, offset=${offset}, len(frame)=${allocations[ptr as i32].byteLength}]`);
    return Int64Array.wrap(allocations[ptr as i32], offset as i32, 1)[0];
};
export function wastrumentation_stack_load_f64(ptr: usize, offset: usize): f64 {
    console.log(`load_f64[*frame=${ptr}, offset=${offset}, len(frame)=${allocations[ptr as i32].byteLength}]`);
    return Float64Array.wrap(allocations[ptr as i32], offset as i32, 1)[0];
};

export function wastrumentation_stack_store_i32(ptr: usize, value: i32, offset: usize): void {
    console.log(`store_i32[*frame=${ptr}, offset=${offset}, value=${value}, len(frame)=${allocations[ptr as i32].byteLength}]`);
    Int32Array.wrap(allocations[ptr as i32], offset as i32, 1)[0] = value;
};
export function wastrumentation_stack_store_f32(ptr: usize, value: f32, offset: usize): void {
    console.log(`store_f32[*frame=${ptr}, offset=${offset}, value=${value}, len(frame)=${allocations[ptr as i32].byteLength}]`);
    Float64Array.wrap(allocations[ptr as i32], offset as i32, 1)[0] = value;
};
export function wastrumentation_stack_store_i64(ptr: usize, value: i64, offset: usize): void {
    console.log(`store_i64[*frame=${ptr}, offset=${offset}, value=${value}, len(frame)=${allocations[ptr as i32].byteLength}]`);
    Int64Array.wrap(allocations[ptr as i32], offset as i32, 1)[0] = value;
};
export function wastrumentation_stack_store_f64(ptr: usize, value: f64, offset: usize): void {
    console.log(`store_f64[*frame=${ptr}, offset=${offset}, value=${value}, len(frame)=${allocations[ptr as i32].byteLength}]`);
    Float64Array.wrap(allocations[ptr as i32], offset as i32, 1)[0] = value;
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
