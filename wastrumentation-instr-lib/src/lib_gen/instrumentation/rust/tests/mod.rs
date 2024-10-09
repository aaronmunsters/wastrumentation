use wastrumentation::wasm_constructs::RefType;

use super::*;
use rust_to_wasm_compiler::{Profile, RustToWasmCompiler};
mod test_edge_case;

#[test]
fn test_compiles() {
    let mut hash_set: HashSet<Signature> = HashSet::default();
    hash_set.insert(Signature {
        return_types: vec![WasmType::I32],
        argument_types: vec![WasmType::F64],
    });
    hash_set.insert(Signature {
        return_types: vec![],
        argument_types: vec![],
    });
    hash_set.insert(Signature {
        return_types: vec![],
        argument_types: vec![WasmType::I32],
    });
    hash_set.insert(Signature {
        return_types: vec![WasmType::I32],
        argument_types: vec![],
    });
    let signatures: Vec<Signature> = hash_set.into_iter().collect();

    let (ManifestSource(manifest), RustSourceCode(rust_source)) = generate_lib(&signatures);
    let wasm_module = RustToWasmCompiler::new()
        .unwrap()
        .compile_source(
            WasiSupport::Disabled,
            &manifest,
            &rust_source,
            Profile::Release,
        )
        .unwrap();

    // Compile result
    assert!(!wasm_module.is_empty());
}

// Some sample signatures for testing purposes
fn get_ret_f64_f32_arg_i32_i64() -> Signature {
    Signature {
        return_types: vec![WasmType::F64, WasmType::F32],
        argument_types: vec![WasmType::I32, WasmType::I64],
    }
}

fn get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64() -> Signature {
    Signature {
        return_types: vec![WasmType::F64, WasmType::F32, WasmType::I32, WasmType::I64],
        argument_types: vec![WasmType::I64, WasmType::I32, WasmType::F32, WasmType::F64],
    }
}

#[test]
fn test_signature_uniqueness() {
    let mut hash_set: HashSet<Signature> = HashSet::default();
    hash_set.insert(get_ret_f64_f32_arg_i32_i64());
    hash_set.insert(get_ret_f64_f32_arg_i32_i64());
    hash_set.insert(get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64());
    hash_set.insert(get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64());
    hash_set.insert(Signature {
        return_types: vec![WasmType::Ref(RefType::ExternRef)],
        argument_types: vec![WasmType::Ref(RefType::FuncRef)],
    });
    hash_set.insert(Signature {
        return_types: vec![WasmType::Ref(RefType::ExternRef)],
        argument_types: vec![WasmType::Ref(RefType::FuncRef)],
    });
    assert_eq!(hash_set.len(), 3);
}

#[test]
fn compute_memory_offset_generically_works_correctly() {
    let pos_frst = 0;
    let pos_scnd = 1;
    let pos_thrd = 2;
    let pos_frth = 3;
    for ((position, ret_count, arg_offset), expected) in [
        ((pos_frst, 0, 1), "0"),
        ((pos_frst, 0, 2), "0"),
        ((pos_scnd, 0, 2), "size_of::<T0>()"),
        ((pos_frst, 1, 0), "0"),
        ((pos_frst, 1, 1), "0"),
        ((pos_scnd, 1, 1), "size_of::<R0>()"),
        ((pos_frst, 1, 2), "0"),
        ((pos_scnd, 1, 2), "size_of::<R0>()"),
        ((pos_thrd, 1, 2), "size_of::<R0>() + size_of::<T0>()"),
        ((pos_frst, 2, 0), "0"),
        ((pos_scnd, 2, 0), "size_of::<R0>()"),
        ((pos_frst, 2, 1), "0"),
        ((pos_scnd, 2, 1), "size_of::<R0>()"),
        ((pos_thrd, 2, 1), "size_of::<R0>() + size_of::<R1>()"),
        ((pos_frst, 2, 2), "0"),
        ((pos_scnd, 2, 2), "size_of::<R0>()"),
        ((pos_thrd, 2, 2), "size_of::<R0>() + size_of::<R1>()"),
        (
            (pos_frth, 2, 2),
            "size_of::<R0>() + size_of::<R1>() + size_of::<T0>()",
        ),
    ] {
        assert_eq!(generics_offset(position, ret_count, arg_offset), expected);
    }
}

#[test]
fn generating_allocate_generic_instructions() {
    assert_eq!(
        generate_allocate_generic(0, 0),
        "
#[inline(always)]
fn allocate_ret_0_arg_0() -> usize  {
    let align = align_of::<u8>();
    let size_to_allocate: usize = 0;
    let layout = unsafe { Layout::from_size_align_unchecked(size_to_allocate, align) };
    let shadow_frame_ptr = unsafe { GlobalAlloc::alloc(&ALLOC, layout) } as usize;
    shadow_frame_ptr as usize
}"
    );

    assert_eq!(
        generate_allocate_generic(0, 1),
        "
#[inline(always)]
fn allocate_ret_0_arg_1<T0>(a0: T0) -> usize where T0: Copy {
    let align = align_of::<u8>();
    let size_to_allocate: usize = size_of::<T0>();
    let layout = unsafe { Layout::from_size_align_unchecked(size_to_allocate, align) };
    let shadow_frame_ptr = unsafe { GlobalAlloc::alloc(&ALLOC, layout) } as usize;
    // store a0
    let a0_offset = 0; // constant folded
    wastrumentation_memory_store::<T0>(shadow_frame_ptr, a0, a0_offset);

    shadow_frame_ptr as usize
}"
    );

    assert_eq!(
        generate_allocate_generic(5, 5),
    "
#[inline(always)]
fn allocate_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(a0: T0, a1: T1, a2: T2, a3: T3, a4: T4) -> usize where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let align = align_of::<u8>();
    let size_to_allocate: usize = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>() + size_of::<R4>() + size_of::<T0>() + size_of::<T1>() + size_of::<T2>() + size_of::<T3>() + size_of::<T4>();
    let layout = unsafe { Layout::from_size_align_unchecked(size_to_allocate, align) };
    let shadow_frame_ptr = unsafe { GlobalAlloc::alloc(&ALLOC, layout) } as usize;
    // store a0
    let a0_offset = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>() + size_of::<R4>(); // constant folded
    wastrumentation_memory_store::<T0>(shadow_frame_ptr, a0, a0_offset);

    // store a1
    let a1_offset = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>() + size_of::<R4>() + size_of::<T0>(); // constant folded
    wastrumentation_memory_store::<T1>(shadow_frame_ptr, a1, a1_offset);

    // store a2
    let a2_offset = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>() + size_of::<R4>() + size_of::<T0>() + size_of::<T1>(); // constant folded
    wastrumentation_memory_store::<T2>(shadow_frame_ptr, a2, a2_offset);

    // store a3
    let a3_offset = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>() + size_of::<R4>() + size_of::<T0>() + size_of::<T1>() + size_of::<T2>(); // constant folded
    wastrumentation_memory_store::<T3>(shadow_frame_ptr, a3, a3_offset);

    // store a4
    let a4_offset = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>() + size_of::<R4>() + size_of::<T0>() + size_of::<T1>() + size_of::<T2>() + size_of::<T3>(); // constant folded
    wastrumentation_memory_store::<T4>(shadow_frame_ptr, a4, a4_offset);

    shadow_frame_ptr as usize
}"
    );
}

#[test]
fn generating_allocate_specialized_instructions() {
    assert_eq!(
        generate_allocate_specialized(&RustSignature(&get_ret_f64_f32_arg_i32_i64())),
        "
#[no_mangle]
pub fn allocate_ret_f64_f32_arg_i32_i64(a0: i32, a1: i64) -> usize {
    return allocate_ret_2_arg_2::<f64, f32, i32, i64>(a0, a1);
}
"
    );

    assert_eq!(generate_allocate_specialized(&RustSignature(&get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64())), "
#[no_mangle]
pub fn allocate_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(a0: i64, a1: i32, a2: f32, a3: f64) -> usize {
    return allocate_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(a0, a1, a2, a3);
}
");
}

#[test]
fn generating_load_generic_instructions() {
    assert_eq!(
        generate_load_generic(0, 1),
        "
#[inline(always)]
fn load_arg0_ret_0_arg_1<T0>(stack_ptr: usize) -> T0 where T0: Copy {
    let a0_offset = 0; // constant folded
    return wastrumentation_memory_load::<T0>(stack_ptr, a0_offset); // inlined
}",
    );
    assert_eq!(generate_load_generic(5, 5), "
#[inline(always)]
fn load_arg0_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize) -> T0 where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let a0_offset = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>() + size_of::<R4>(); // constant folded
    return wastrumentation_memory_load::<T0>(stack_ptr, a0_offset); // inlined
}

#[inline(always)]
fn load_arg1_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize) -> T1 where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let a1_offset = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>() + size_of::<R4>() + size_of::<T0>(); // constant folded
    return wastrumentation_memory_load::<T1>(stack_ptr, a1_offset); // inlined
}

#[inline(always)]
fn load_arg2_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize) -> T2 where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let a2_offset = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>() + size_of::<R4>() + size_of::<T0>() + size_of::<T1>(); // constant folded
    return wastrumentation_memory_load::<T2>(stack_ptr, a2_offset); // inlined
}

#[inline(always)]
fn load_arg3_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize) -> T3 where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let a3_offset = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>() + size_of::<R4>() + size_of::<T0>() + size_of::<T1>() + size_of::<T2>(); // constant folded
    return wastrumentation_memory_load::<T3>(stack_ptr, a3_offset); // inlined
}

#[inline(always)]
fn load_arg4_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize) -> T4 where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let a4_offset = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>() + size_of::<R4>() + size_of::<T0>() + size_of::<T1>() + size_of::<T2>() + size_of::<T3>(); // constant folded
    return wastrumentation_memory_load::<T4>(stack_ptr, a4_offset); // inlined
}

#[inline(always)]
fn load_ret0_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize) -> R0 where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let r0_offset = 0; // constant folded
    return wastrumentation_memory_load::<R0>(stack_ptr, r0_offset); // inlined
}

#[inline(always)]
fn load_ret1_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize) -> R1 where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let r1_offset = size_of::<R0>(); // constant folded
    return wastrumentation_memory_load::<R1>(stack_ptr, r1_offset); // inlined
}

#[inline(always)]
fn load_ret2_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize) -> R2 where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let r2_offset = size_of::<R0>() + size_of::<R1>(); // constant folded
    return wastrumentation_memory_load::<R2>(stack_ptr, r2_offset); // inlined
}

#[inline(always)]
fn load_ret3_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize) -> R3 where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let r3_offset = size_of::<R0>() + size_of::<R1>() + size_of::<R2>(); // constant folded
    return wastrumentation_memory_load::<R3>(stack_ptr, r3_offset); // inlined
}

#[inline(always)]
fn load_ret4_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize) -> R4 where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let r4_offset = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>(); // constant folded
    return wastrumentation_memory_load::<R4>(stack_ptr, r4_offset); // inlined
}");
}

#[test]
fn generating_load_specialized_instructions() {
    assert_eq!(
        generate_load_specialized(&RustSignature(&get_ret_f64_f32_arg_i32_i64())),
        "
#[no_mangle]
pub fn load_arg0_ret_f64_f32_arg_i32_i64(stack_ptr: usize) -> i32 {
    return load_arg0_ret_2_arg_2::<f64, f32, i32, i64>(stack_ptr);
}

#[no_mangle]
pub fn load_arg1_ret_f64_f32_arg_i32_i64(stack_ptr: usize) -> i64 {
    return load_arg1_ret_2_arg_2::<f64, f32, i32, i64>(stack_ptr);
}

#[no_mangle]
pub fn load_ret0_ret_f64_f32_arg_i32_i64(stack_ptr: usize) -> f64 {
    return load_ret0_ret_2_arg_2::<f64, f32, i32, i64>(stack_ptr);
}

#[no_mangle]
pub fn load_ret1_ret_f64_f32_arg_i32_i64(stack_ptr: usize) -> f32 {
    return load_ret1_ret_2_arg_2::<f64, f32, i32, i64>(stack_ptr);
}"
    );

    assert_eq!(
        generate_load_specialized(&RustSignature(
            &get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64()
        )),
        "
#[no_mangle]
pub fn load_arg0_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize) -> i64 {
    return load_arg0_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
}

#[no_mangle]
pub fn load_arg1_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize) -> i32 {
    return load_arg1_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
}

#[no_mangle]
pub fn load_arg2_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize) -> f32 {
    return load_arg2_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
}

#[no_mangle]
pub fn load_arg3_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize) -> f64 {
    return load_arg3_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
}

#[no_mangle]
pub fn load_ret0_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize) -> f64 {
    return load_ret0_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
}

#[no_mangle]
pub fn load_ret1_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize) -> f32 {
    return load_ret1_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
}

#[no_mangle]
pub fn load_ret2_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize) -> i32 {
    return load_ret2_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
}

#[no_mangle]
pub fn load_ret3_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize) -> i64 {
    return load_ret3_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr);
}"
    );
}

#[test]
fn generating_store_generic_instructions() {
    assert_eq!(
        generate_store_generic(0, 1),
        "
#[inline(always)]
fn store_arg0_ret_0_arg_1<T0>(stack_ptr: usize, a0: T0) where T0: Copy {
    let a0_offset: usize = 0; // constant folded
    return wastrumentation_memory_store::<T0>(stack_ptr, a0, a0_offset); // inlined
}",
    );
    assert_eq!(generate_store_generic(5, 5), "
#[inline(always)]
fn store_arg0_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, a0: T0) where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let a0_offset: usize = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>() + size_of::<R4>(); // constant folded
    return wastrumentation_memory_store::<T0>(stack_ptr, a0, a0_offset); // inlined
}

#[inline(always)]
fn store_arg1_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, a1: T1) where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let a1_offset: usize = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>() + size_of::<R4>() + size_of::<T0>(); // constant folded
    return wastrumentation_memory_store::<T1>(stack_ptr, a1, a1_offset); // inlined
}

#[inline(always)]
fn store_arg2_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, a2: T2) where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let a2_offset: usize = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>() + size_of::<R4>() + size_of::<T0>() + size_of::<T1>(); // constant folded
    return wastrumentation_memory_store::<T2>(stack_ptr, a2, a2_offset); // inlined
}

#[inline(always)]
fn store_arg3_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, a3: T3) where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let a3_offset: usize = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>() + size_of::<R4>() + size_of::<T0>() + size_of::<T1>() + size_of::<T2>(); // constant folded
    return wastrumentation_memory_store::<T3>(stack_ptr, a3, a3_offset); // inlined
}

#[inline(always)]
fn store_arg4_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, a4: T4) where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let a4_offset: usize = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>() + size_of::<R4>() + size_of::<T0>() + size_of::<T1>() + size_of::<T2>() + size_of::<T3>(); // constant folded
    return wastrumentation_memory_store::<T4>(stack_ptr, a4, a4_offset); // inlined
}

#[inline(always)]
fn store_ret0_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, r0: R0) where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let r0_offset: usize = 0; // constant folded
    return wastrumentation_memory_store::<R0>(stack_ptr, r0, r0_offset); // inlined
}

#[inline(always)]
fn store_ret1_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, r1: R1) where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let r1_offset: usize = size_of::<R0>(); // constant folded
    return wastrumentation_memory_store::<R1>(stack_ptr, r1, r1_offset); // inlined
}

#[inline(always)]
fn store_ret2_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, r2: R2) where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let r2_offset: usize = size_of::<R0>() + size_of::<R1>(); // constant folded
    return wastrumentation_memory_store::<R2>(stack_ptr, r2, r2_offset); // inlined
}

#[inline(always)]
fn store_ret3_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, r3: R3) where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let r3_offset: usize = size_of::<R0>() + size_of::<R1>() + size_of::<R2>(); // constant folded
    return wastrumentation_memory_store::<R3>(stack_ptr, r3, r3_offset); // inlined
}

#[inline(always)]
fn store_ret4_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, r4: R4) where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let r4_offset: usize = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>(); // constant folded
    return wastrumentation_memory_store::<R4>(stack_ptr, r4, r4_offset); // inlined
}");
}

#[test]
fn generating_store_specialized_instructions() {
    assert_eq!(
        generate_store_specialized(&RustSignature(&get_ret_f64_f32_arg_i32_i64())),
        "
#[no_mangle]
pub fn store_arg0_ret_f64_f32_arg_i32_i64(stack_ptr: usize, a0: i32) {
    return store_arg0_ret_2_arg_2::<f64, f32, i32, i64>(stack_ptr, a0);
}

#[no_mangle]
pub fn store_arg1_ret_f64_f32_arg_i32_i64(stack_ptr: usize, a1: i64) {
    return store_arg1_ret_2_arg_2::<f64, f32, i32, i64>(stack_ptr, a1);
}

#[no_mangle]
pub fn store_ret0_ret_f64_f32_arg_i32_i64(stack_ptr: usize, a0: f64) {
    return store_ret0_ret_2_arg_2::<f64, f32, i32, i64>(stack_ptr, a0);
}

#[no_mangle]
pub fn store_ret1_ret_f64_f32_arg_i32_i64(stack_ptr: usize, a1: f32) {
    return store_ret1_ret_2_arg_2::<f64, f32, i32, i64>(stack_ptr, a1);
}",
    );

    assert_eq!(
        generate_store_specialized(&RustSignature(
            &get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64()
        )),
        "
#[no_mangle]
pub fn store_arg0_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a0: i64) {
    return store_arg0_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a0);
}

#[no_mangle]
pub fn store_arg1_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a1: i32) {
    return store_arg1_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a1);
}

#[no_mangle]
pub fn store_arg2_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a2: f32) {
    return store_arg2_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a2);
}

#[no_mangle]
pub fn store_arg3_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a3: f64) {
    return store_arg3_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a3);
}

#[no_mangle]
pub fn store_ret0_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a0: f64) {
    return store_ret0_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a0);
}

#[no_mangle]
pub fn store_ret1_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a1: f32) {
    return store_ret1_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a1);
}

#[no_mangle]
pub fn store_ret2_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a2: i32) {
    return store_ret2_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a2);
}

#[no_mangle]
pub fn store_ret3_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a3: i64) {
    return store_ret3_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a3);
}"
    );
}

#[test]
fn generating_free_generic_instruction() {
    assert_eq!(
        generate_free_values_buffer_generic(0, 1),
        "
#[inline(always)]
fn free_values_ret_0_arg_1<T0>(ptr: usize) where T0: Copy {
    let to_deallocate = size_of::<T0>(); // constant folded
    stack_deallocate(ptr, to_deallocate); // inlined
    return;
}"
    );
    assert_eq!(generate_free_values_buffer_generic(5, 5), "
#[inline(always)]
fn free_values_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(ptr: usize) where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    let to_deallocate = size_of::<R0>() + size_of::<R1>() + size_of::<R2>() + size_of::<R3>() + size_of::<R4>() + size_of::<T0>() + size_of::<T1>() + size_of::<T2>() + size_of::<T3>() + size_of::<T4>(); // constant folded
    stack_deallocate(ptr, to_deallocate); // inlined
    return;
}");
}

#[test]
fn generating_free_specialized_instruction() {
    assert_eq!(
        generate_free_values_buffer_specialized(&RustSignature(&get_ret_f64_f32_arg_i32_i64())),
        "
#[no_mangle]
pub fn free_values_ret_f64_f32_arg_i32_i64(ptr: usize) {
    return free_values_ret_2_arg_2::<f64, f32, i32, i64>(ptr);
}",
    );

    assert_eq!(
        generate_free_values_buffer_specialized(&RustSignature(
            &get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64()
        )),
        "
#[no_mangle]
pub fn free_values_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(ptr: usize) {
    return free_values_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(ptr);
}",
    );
}

#[test]
fn generating_free_types_generic_instruction() {
    assert_eq!(
        generate_free_types_buffer_generic(0, 1),
        "
#[inline(always)]
fn free_types_ret_0_arg_1(ptr: usize) {
    let to_deallocate = size_of::<i32>() * 1; // constant folded
    stack_deallocate(ptr, to_deallocate); // inlined
    return;
}"
    );
    assert_eq!(
        generate_free_types_buffer_generic(5, 5),
        "
#[inline(always)]
fn free_types_ret_5_arg_5(ptr: usize) {
    let to_deallocate = size_of::<i32>() * 10; // constant folded
    stack_deallocate(ptr, to_deallocate); // inlined
    return;
}"
    );
}

#[test]
fn generating_free_types_specialized_instruction() {
    assert_eq!(
        generate_free_types_buffer_specialized(&RustSignature(&get_ret_f64_f32_arg_i32_i64())),
        "
#[no_mangle]
pub fn free_types_ret_f64_f32_arg_i32_i64(ptr: usize) {
    return free_types_ret_2_arg_2(ptr);
}"
    );
    assert_eq!(
        generate_free_types_buffer_specialized(&RustSignature(
            &get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64()
        )),
        "
#[no_mangle]
pub fn free_types_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(ptr: usize) {
    return free_types_ret_4_arg_4(ptr);
}"
    );
}

#[test]
fn generating_store_rets_generic_instruction() {
    assert_eq!(
        generate_store_rets_generic(0, 1),
        "
#[inline(always)]
fn store_rets_ret_0_arg_1<T0>(_stack_ptr: usize) where T0: Copy {
    return;
}",
    );
    assert_eq!(generate_store_rets_generic(5, 5), "
#[inline(always)]
fn store_rets_ret_5_arg_5<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr: usize, a0: R0, a1: R1, a2: R2, a3: R3, a4: R4) where R0: Copy, R1: Copy, R2: Copy, R3: Copy, R4: Copy, T0: Copy, T1: Copy, T2: Copy, T3: Copy, T4: Copy {
    // store a0
    store_ret0_ret_5_arg_5::<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr, a0);
    // store a1
    store_ret1_ret_5_arg_5::<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr, a1);
    // store a2
    store_ret2_ret_5_arg_5::<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr, a2);
    // store a3
    store_ret3_ret_5_arg_5::<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr, a3);
    // store a4
    store_ret4_ret_5_arg_5::<R0, R1, R2, R3, R4, T0, T1, T2, T3, T4>(stack_ptr, a4);
    return;
}");
}

#[test]
fn generating_store_rets_specialized_instruction() {
    assert_eq!(
        generate_store_rets_specialized(&RustSignature(&get_ret_f64_f32_arg_i32_i64())),
        "
#[no_mangle]
pub fn store_rets_ret_f64_f32_arg_i32_i64(stack_ptr: usize, a0: f64, a1: f32) {
    return store_rets_ret_2_arg_2::<f64, f32, i32, i64>(stack_ptr, a0, a1);
}"
    );

    assert_eq!(generate_store_rets_specialized(&RustSignature(&get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64())), "
#[no_mangle]
pub fn store_rets_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a0: f64, a1: f32, a2: i32, a3: i64) {
    return store_rets_ret_4_arg_4::<f64, f32, i32, i64, i64, i32, f32, f64>(stack_ptr, a0, a1, a2, a3);
}");
}

#[test]
fn generating_allocate_types_generic_specialized() {
    assert_eq!(
        generate_allocate_types_buffer_generic(0, 1),
        "
#[inline(always)]
fn allocate_signature_types_buffer_ret_0_arg_1() -> usize {
    let to_allocate = size_of::<i32>() * 1; // constant folded
    let stack_begin = stack_allocate(to_allocate); // inlined
    return stack_begin;
}"
    );

    assert_eq!(
        generate_allocate_types_buffer_generic(5, 5),
        "
#[inline(always)]
fn allocate_signature_types_buffer_ret_5_arg_5() -> usize {
    let to_allocate = size_of::<i32>() * 10; // constant folded
    let stack_begin = stack_allocate(to_allocate); // inlined
    return stack_begin;
}"
    )
}

#[test]
fn generating_allocate_types_buffer_specialized_instruction() {
    let signature_empty = Signature {
        return_types: vec![],
        argument_types: vec![],
    };
    assert_eq!(
        generate_allocate_types_buffer_specialized(&RustSignature(&signature_empty)),
        "
#[no_mangle]
pub fn allocate_types_ret_arg() -> usize {
    let types_buffer = allocate_signature_types_buffer_ret_0_arg_0();
    return types_buffer;
}"
    );
    let signature_0 = Signature {
        return_types: vec![WasmType::F64, WasmType::F32],
        argument_types: vec![WasmType::I32, WasmType::I64],
    };
    assert_eq!(
        generate_allocate_types_buffer_specialized(&RustSignature(&signature_0)),
        "
#[no_mangle]
pub fn allocate_types_ret_f64_f32_arg_i32_i64() -> usize {
    let types_buffer = allocate_signature_types_buffer_ret_2_arg_2();
    wastrumentation_memory_store::<i32>(types_buffer, 3, size_of::<i32>()*0);
    wastrumentation_memory_store::<i32>(types_buffer, 1, size_of::<i32>()*1);
    wastrumentation_memory_store::<i32>(types_buffer, 0, size_of::<i32>()*2);
    wastrumentation_memory_store::<i32>(types_buffer, 2, size_of::<i32>()*3);
    return types_buffer;
}"
    );
    let signature_1 = Signature {
        return_types: vec![WasmType::F64, WasmType::F32, WasmType::I32, WasmType::I64],
        argument_types: vec![WasmType::I64, WasmType::I32, WasmType::F32, WasmType::F64],
    };
    assert_eq!(
        generate_allocate_types_buffer_specialized(&RustSignature(&signature_1)),
        "
#[no_mangle]
pub fn allocate_types_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64() -> usize {
    let types_buffer = allocate_signature_types_buffer_ret_4_arg_4();
    wastrumentation_memory_store::<i32>(types_buffer, 3, size_of::<i32>()*0);
    wastrumentation_memory_store::<i32>(types_buffer, 1, size_of::<i32>()*1);
    wastrumentation_memory_store::<i32>(types_buffer, 0, size_of::<i32>()*2);
    wastrumentation_memory_store::<i32>(types_buffer, 2, size_of::<i32>()*3);
    wastrumentation_memory_store::<i32>(types_buffer, 2, size_of::<i32>()*4);
    wastrumentation_memory_store::<i32>(types_buffer, 0, size_of::<i32>()*5);
    wastrumentation_memory_store::<i32>(types_buffer, 1, size_of::<i32>()*6);
    wastrumentation_memory_store::<i32>(types_buffer, 3, size_of::<i32>()*7);
    return types_buffer;
}"
    );

    let signature_2 = Signature {
        return_types: vec![
            WasmType::Ref(RefType::FuncRef),
            WasmType::Ref(RefType::FuncRef),
        ],
        argument_types: vec![
            WasmType::Ref(RefType::ExternRef),
            WasmType::Ref(RefType::ExternRef),
        ],
    };
    assert_eq!(
        generate_allocate_types_buffer_specialized(&RustSignature(&signature_2)),
        "
#[no_mangle]
pub fn allocate_types_ret_ref_func_ref_func_arg_ref_extern_ref_extern() -> usize {
    let types_buffer = allocate_signature_types_buffer_ret_2_arg_2();
    wastrumentation_memory_store::<i32>(types_buffer, 4, size_of::<i32>()*0);
    wastrumentation_memory_store::<i32>(types_buffer, 4, size_of::<i32>()*1);
    wastrumentation_memory_store::<i32>(types_buffer, 5, size_of::<i32>()*2);
    wastrumentation_memory_store::<i32>(types_buffer, 5, size_of::<i32>()*3);
    return types_buffer;
}"
    );
}
