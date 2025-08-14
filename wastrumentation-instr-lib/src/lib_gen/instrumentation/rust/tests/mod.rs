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
fn generating_allocate_generic_instructions() {
    assert_eq!(
        generate_allocate_generic(0, 0),
        "
#[inline(always)]
fn allocate_ret_0_arg_0() -> usize {
    0 // Fake shadow pointer for empty signatures
}"
    );

    assert_eq!(
        generate_allocate_generic(0, 1),
        "
#[inline(always)]
fn allocate_ret_0_arg_1(a0: WasmValue) -> usize {
    let frame_ptr = stack_allocate_values(1);
    wastrumentation_memory_store(frame_ptr, a0, 0);
    frame_ptr
}"
    );

    assert_eq!(
        generate_allocate_generic(5, 5),
    "
#[inline(always)]
fn allocate_ret_5_arg_5(a0: WasmValue, a1: WasmValue, a2: WasmValue, a3: WasmValue, a4: WasmValue) -> usize {
    let frame_ptr = stack_allocate_values(10);
    wastrumentation_memory_store(frame_ptr, a0, 5);
    wastrumentation_memory_store(frame_ptr, a1, 6);
    wastrumentation_memory_store(frame_ptr, a2, 7);
    wastrumentation_memory_store(frame_ptr, a3, 8);
    wastrumentation_memory_store(frame_ptr, a4, 9);
    frame_ptr
}"
    );
}

#[test]
fn generating_allocate_specialized_instructions() {
    assert_eq!(
        generate_allocate_specialized(&RustSignature(&get_ret_f64_f32_arg_i32_i64())),
        "
#[no_mangle]
pub extern \"C\" fn allocate_ret_f64_f32_arg_i32_i64(a0: i32, a1: i64) -> usize {
    allocate_ret_2_arg_2(WasmValue::new_i32(a0), WasmValue::new_i64(a1))
}
"
    );

    assert_eq!(generate_allocate_specialized(&RustSignature(&get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64())), "
#[no_mangle]
pub extern \"C\" fn allocate_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(a0: i64, a1: i32, a2: f32, a3: f64) -> usize {
    allocate_ret_4_arg_4(WasmValue::new_i64(a0), WasmValue::new_i32(a1), WasmValue::new_f32(a2), WasmValue::new_f64(a3))
}
");
}

#[test]
fn generating_load_generic_instructions() {
    assert_eq!(
        generate_load_generic(0, 1),
        "
#[inline(always)]
fn load_arg0_ret_0_arg_1(stack_ptr: usize) -> WasmValue {
    wastrumentation_memory_load(stack_ptr, 0)
}",
    );
    assert_eq!(
        generate_load_generic(5, 5),
        "
#[inline(always)]
fn load_arg0_ret_5_arg_5(stack_ptr: usize) -> WasmValue {
    wastrumentation_memory_load(stack_ptr, 5)
}

#[inline(always)]
fn load_arg1_ret_5_arg_5(stack_ptr: usize) -> WasmValue {
    wastrumentation_memory_load(stack_ptr, 6)
}

#[inline(always)]
fn load_arg2_ret_5_arg_5(stack_ptr: usize) -> WasmValue {
    wastrumentation_memory_load(stack_ptr, 7)
}

#[inline(always)]
fn load_arg3_ret_5_arg_5(stack_ptr: usize) -> WasmValue {
    wastrumentation_memory_load(stack_ptr, 8)
}

#[inline(always)]
fn load_arg4_ret_5_arg_5(stack_ptr: usize) -> WasmValue {
    wastrumentation_memory_load(stack_ptr, 9)
}

#[inline(always)]
fn load_ret0_ret_5_arg_5(stack_ptr: usize) -> WasmValue {
    wastrumentation_memory_load(stack_ptr, 0)
}

#[inline(always)]
fn load_ret1_ret_5_arg_5(stack_ptr: usize) -> WasmValue {
    wastrumentation_memory_load(stack_ptr, 1)
}

#[inline(always)]
fn load_ret2_ret_5_arg_5(stack_ptr: usize) -> WasmValue {
    wastrumentation_memory_load(stack_ptr, 2)
}

#[inline(always)]
fn load_ret3_ret_5_arg_5(stack_ptr: usize) -> WasmValue {
    wastrumentation_memory_load(stack_ptr, 3)
}

#[inline(always)]
fn load_ret4_ret_5_arg_5(stack_ptr: usize) -> WasmValue {
    wastrumentation_memory_load(stack_ptr, 4)
}"
    );
}

#[test]
fn generating_load_specialized_instructions() {
    assert_eq!(
        generate_load_specialized(&RustSignature(&get_ret_f64_f32_arg_i32_i64())),
        "
#[no_mangle]
pub extern \"C\" fn load_arg0_ret_f64_f32_arg_i32_i64(stack_ptr: usize) -> i32 {
    let val = load_arg0_ret_2_arg_2(stack_ptr);
    unsafe { val.i32 }
}

#[no_mangle]
pub extern \"C\" fn load_arg1_ret_f64_f32_arg_i32_i64(stack_ptr: usize) -> i64 {
    let val = load_arg1_ret_2_arg_2(stack_ptr);
    unsafe { val.i64 }
}

#[no_mangle]
pub extern \"C\" fn load_ret0_ret_f64_f32_arg_i32_i64(stack_ptr: usize) -> f64 {
    let val = load_ret0_ret_2_arg_2(stack_ptr);
    unsafe { val.f64 }
}

#[no_mangle]
pub extern \"C\" fn load_ret1_ret_f64_f32_arg_i32_i64(stack_ptr: usize) -> f32 {
    let val = load_ret1_ret_2_arg_2(stack_ptr);
    unsafe { val.f32 }
}"
    );

    assert_eq!(
        generate_load_specialized(&RustSignature(
            &get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64()
        )),
        "
#[no_mangle]
pub extern \"C\" fn load_arg0_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize) -> i64 {
    let val = load_arg0_ret_4_arg_4(stack_ptr);
    unsafe { val.i64 }
}

#[no_mangle]
pub extern \"C\" fn load_arg1_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize) -> i32 {
    let val = load_arg1_ret_4_arg_4(stack_ptr);
    unsafe { val.i32 }
}

#[no_mangle]
pub extern \"C\" fn load_arg2_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize) -> f32 {
    let val = load_arg2_ret_4_arg_4(stack_ptr);
    unsafe { val.f32 }
}

#[no_mangle]
pub extern \"C\" fn load_arg3_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize) -> f64 {
    let val = load_arg3_ret_4_arg_4(stack_ptr);
    unsafe { val.f64 }
}

#[no_mangle]
pub extern \"C\" fn load_ret0_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize) -> f64 {
    let val = load_ret0_ret_4_arg_4(stack_ptr);
    unsafe { val.f64 }
}

#[no_mangle]
pub extern \"C\" fn load_ret1_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize) -> f32 {
    let val = load_ret1_ret_4_arg_4(stack_ptr);
    unsafe { val.f32 }
}

#[no_mangle]
pub extern \"C\" fn load_ret2_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize) -> i32 {
    let val = load_ret2_ret_4_arg_4(stack_ptr);
    unsafe { val.i32 }
}

#[no_mangle]
pub extern \"C\" fn load_ret3_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize) -> i64 {
    let val = load_ret3_ret_4_arg_4(stack_ptr);
    unsafe { val.i64 }
}"
    );
}

#[test]
fn generating_store_generic_instructions() {
    assert_eq!(
        generate_store_generic(0, 1),
        "
#[inline(always)]
fn store_arg0_ret_0_arg_1(stack_ptr: usize, a0: WasmValue) {
    wastrumentation_memory_store(stack_ptr, a0, 0);
}",
    );
    assert_eq!(
        generate_store_generic(5, 5),
        "
#[inline(always)]
fn store_arg0_ret_5_arg_5(stack_ptr: usize, a0: WasmValue) {
    wastrumentation_memory_store(stack_ptr, a0, 5);
}

#[inline(always)]
fn store_arg1_ret_5_arg_5(stack_ptr: usize, a1: WasmValue) {
    wastrumentation_memory_store(stack_ptr, a1, 6);
}

#[inline(always)]
fn store_arg2_ret_5_arg_5(stack_ptr: usize, a2: WasmValue) {
    wastrumentation_memory_store(stack_ptr, a2, 7);
}

#[inline(always)]
fn store_arg3_ret_5_arg_5(stack_ptr: usize, a3: WasmValue) {
    wastrumentation_memory_store(stack_ptr, a3, 8);
}

#[inline(always)]
fn store_arg4_ret_5_arg_5(stack_ptr: usize, a4: WasmValue) {
    wastrumentation_memory_store(stack_ptr, a4, 9);
}

#[inline(always)]
fn store_ret0_ret_5_arg_5(stack_ptr: usize, r0: WasmValue) {
    wastrumentation_memory_store(stack_ptr, r0, 0);
}

#[inline(always)]
fn store_ret1_ret_5_arg_5(stack_ptr: usize, r1: WasmValue) {
    wastrumentation_memory_store(stack_ptr, r1, 1);
}

#[inline(always)]
fn store_ret2_ret_5_arg_5(stack_ptr: usize, r2: WasmValue) {
    wastrumentation_memory_store(stack_ptr, r2, 2);
}

#[inline(always)]
fn store_ret3_ret_5_arg_5(stack_ptr: usize, r3: WasmValue) {
    wastrumentation_memory_store(stack_ptr, r3, 3);
}

#[inline(always)]
fn store_ret4_ret_5_arg_5(stack_ptr: usize, r4: WasmValue) {
    wastrumentation_memory_store(stack_ptr, r4, 4);
}"
    );
}

#[test]
fn generating_store_specialized_instructions() {
    assert_eq!(
        generate_store_specialized(&RustSignature(&get_ret_f64_f32_arg_i32_i64())),
        "
#[no_mangle]
pub extern \"C\" fn store_arg0_ret_f64_f32_arg_i32_i64(stack_ptr: usize, a0: i32) {
    store_arg0_ret_2_arg_2(stack_ptr, WasmValue::new_i32(a0));
}

#[no_mangle]
pub extern \"C\" fn store_arg1_ret_f64_f32_arg_i32_i64(stack_ptr: usize, a1: i64) {
    store_arg1_ret_2_arg_2(stack_ptr, WasmValue::new_i64(a1));
}

#[no_mangle]
pub extern \"C\" fn store_ret0_ret_f64_f32_arg_i32_i64(stack_ptr: usize, a0: f64) {
    store_ret0_ret_2_arg_2(stack_ptr, WasmValue::new_f64(a0));
}

#[no_mangle]
pub extern \"C\" fn store_ret1_ret_f64_f32_arg_i32_i64(stack_ptr: usize, a1: f32) {
    store_ret1_ret_2_arg_2(stack_ptr, WasmValue::new_f32(a1));
}",
    );

    assert_eq!(
        generate_store_specialized(&RustSignature(
            &get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64()
        )),
        "
#[no_mangle]
pub extern \"C\" fn store_arg0_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a0: i64) {
    store_arg0_ret_4_arg_4(stack_ptr, WasmValue::new_i64(a0));
}

#[no_mangle]
pub extern \"C\" fn store_arg1_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a1: i32) {
    store_arg1_ret_4_arg_4(stack_ptr, WasmValue::new_i32(a1));
}

#[no_mangle]
pub extern \"C\" fn store_arg2_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a2: f32) {
    store_arg2_ret_4_arg_4(stack_ptr, WasmValue::new_f32(a2));
}

#[no_mangle]
pub extern \"C\" fn store_arg3_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a3: f64) {
    store_arg3_ret_4_arg_4(stack_ptr, WasmValue::new_f64(a3));
}

#[no_mangle]
pub extern \"C\" fn store_ret0_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a0: f64) {
    store_ret0_ret_4_arg_4(stack_ptr, WasmValue::new_f64(a0));
}

#[no_mangle]
pub extern \"C\" fn store_ret1_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a1: f32) {
    store_ret1_ret_4_arg_4(stack_ptr, WasmValue::new_f32(a1));
}

#[no_mangle]
pub extern \"C\" fn store_ret2_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a2: i32) {
    store_ret2_ret_4_arg_4(stack_ptr, WasmValue::new_i32(a2));
}

#[no_mangle]
pub extern \"C\" fn store_ret3_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a3: i64) {
    store_ret3_ret_4_arg_4(stack_ptr, WasmValue::new_i64(a3));
}"
    );
}

#[test]
fn generating_free_generic_instruction() {
    assert_eq!(
        generate_free_values_buffer_generic(0, 1),
        "
#[inline(always)]
fn free_values_ret_0_arg_1(ptr: usize) {
    stack_deallocate_values(ptr, 1);
}"
    );
    assert_eq!(
        generate_free_values_buffer_generic(5, 5),
        "
#[inline(always)]
fn free_values_ret_5_arg_5(ptr: usize) {
    stack_deallocate_values(ptr, 10);
}"
    );
}

#[test]
fn generating_free_specialized_instruction() {
    assert_eq!(
        generate_free_values_buffer_specialized(&RustSignature(&get_ret_f64_f32_arg_i32_i64())),
        "
#[no_mangle]
pub extern \"C\" fn free_values_ret_f64_f32_arg_i32_i64(ptr: usize) {
    free_values_ret_2_arg_2(ptr);
}",
    );

    assert_eq!(
        generate_free_values_buffer_specialized(&RustSignature(
            &get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64()
        )),
        "
#[no_mangle]
pub extern \"C\" fn free_values_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(ptr: usize) {
    free_values_ret_4_arg_4(ptr);
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
    stack_deallocate_types(ptr, 1);
}"
    );
    assert_eq!(
        generate_free_types_buffer_generic(5, 5),
        "
#[inline(always)]
fn free_types_ret_5_arg_5(ptr: usize) {
    stack_deallocate_types(ptr, 10);
}"
    );
}

#[test]
fn generating_free_types_specialized_instruction() {
    assert_eq!(
        generate_free_types_buffer_specialized(&RustSignature(&get_ret_f64_f32_arg_i32_i64())),
        "
#[no_mangle]
pub extern \"C\" fn free_types_ret_f64_f32_arg_i32_i64(ptr: usize) {
    free_types_ret_2_arg_2(ptr);
}"
    );
    assert_eq!(
        generate_free_types_buffer_specialized(&RustSignature(
            &get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64()
        )),
        "
#[no_mangle]
pub extern \"C\" fn free_types_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(ptr: usize) {
    free_types_ret_4_arg_4(ptr);
}"
    );
}

#[test]
fn generating_store_rets_generic_instruction() {
    assert_eq!(
        generate_store_rets_generic(0, 1),
        "
#[inline(always)]
fn store_rets_ret_0_arg_1(_stack_ptr: usize) {

}",
    );
    assert_eq!(generate_store_rets_generic(5, 5), "
#[inline(always)]
fn store_rets_ret_5_arg_5(stack_ptr: usize, a0: WasmValue, a1: WasmValue, a2: WasmValue, a3: WasmValue, a4: WasmValue) {
    store_ret0_ret_5_arg_5(stack_ptr, a0);
    store_ret1_ret_5_arg_5(stack_ptr, a1);
    store_ret2_ret_5_arg_5(stack_ptr, a2);
    store_ret3_ret_5_arg_5(stack_ptr, a3);
    store_ret4_ret_5_arg_5(stack_ptr, a4);
}");
}

#[test]
fn generating_store_rets_specialized_instruction() {
    assert_eq!(
        generate_store_rets_specialized(&RustSignature(&get_ret_f64_f32_arg_i32_i64())),
        "
#[no_mangle]
pub extern \"C\" fn store_rets_ret_f64_f32_arg_i32_i64(stack_ptr: usize, a0: f64, a1: f32) {
    store_rets_ret_2_arg_2(stack_ptr, WasmValue::new_f64(a0), WasmValue::new_f32(a1));
}"
    );

    assert_eq!(generate_store_rets_specialized(&RustSignature(&get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64())), "
#[no_mangle]
pub extern \"C\" fn store_rets_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(stack_ptr: usize, a0: f64, a1: f32, a2: i32, a3: i64) {
    store_rets_ret_4_arg_4(stack_ptr, WasmValue::new_f64(a0), WasmValue::new_f32(a1), WasmValue::new_i32(a2), WasmValue::new_i64(a3));
}");
}

#[test]
fn generating_allocate_types_generic_specialized() {
    assert_eq!(
        generate_allocate_types_buffer_generic(0, 1),
        "
#[inline(always)]
fn allocate_signature_types_buffer_ret_0_arg_1() -> usize {
    stack_allocate_types(1) // inlined
}"
    );

    assert_eq!(
        generate_allocate_types_buffer_generic(5, 5),
        "
#[inline(always)]
fn allocate_signature_types_buffer_ret_5_arg_5() -> usize {
    stack_allocate_types(10) // inlined
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
pub extern \"C\" fn allocate_types_ret_arg() -> usize {
    let types_buffer = allocate_signature_types_buffer_ret_0_arg_0();

    types_buffer
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
pub extern \"C\" fn allocate_types_ret_f64_f32_arg_i32_i64() -> usize {
    let types_buffer = allocate_signature_types_buffer_ret_2_arg_2();
    wastrumentation_stack_store_type(types_buffer, 0, RuntimeType::F64);
    wastrumentation_stack_store_type(types_buffer, 1, RuntimeType::F32);
    wastrumentation_stack_store_type(types_buffer, 2, RuntimeType::I32);
    wastrumentation_stack_store_type(types_buffer, 3, RuntimeType::I64);
    types_buffer
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
pub extern \"C\" fn allocate_types_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64() -> usize {
    let types_buffer = allocate_signature_types_buffer_ret_4_arg_4();
    wastrumentation_stack_store_type(types_buffer, 0, RuntimeType::F64);
    wastrumentation_stack_store_type(types_buffer, 1, RuntimeType::F32);
    wastrumentation_stack_store_type(types_buffer, 2, RuntimeType::I32);
    wastrumentation_stack_store_type(types_buffer, 3, RuntimeType::I64);
    wastrumentation_stack_store_type(types_buffer, 4, RuntimeType::I64);
    wastrumentation_stack_store_type(types_buffer, 5, RuntimeType::I32);
    wastrumentation_stack_store_type(types_buffer, 6, RuntimeType::F32);
    wastrumentation_stack_store_type(types_buffer, 7, RuntimeType::F64);
    types_buffer
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
pub extern \"C\" fn allocate_types_ret_ref_func_ref_func_arg_ref_extern_ref_extern() -> usize {
    let types_buffer = allocate_signature_types_buffer_ret_2_arg_2();
    wastrumentation_stack_store_type(types_buffer, 0, RuntimeType::FuncRef);
    wastrumentation_stack_store_type(types_buffer, 1, RuntimeType::FuncRef);
    wastrumentation_stack_store_type(types_buffer, 2, RuntimeType::ExternRef);
    wastrumentation_stack_store_type(types_buffer, 3, RuntimeType::ExternRef);
    types_buffer
}"
    );
}
