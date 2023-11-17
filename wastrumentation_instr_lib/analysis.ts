@external("instrumented_input", "call_base")
declare function call_base(f_apply: i32, argv: i32): void
@external("wastrumentation_stack", "load_arg0_ret_i32_arg_i32_i32")
declare function load_arg0_ret_i32_arg_i32_i32(argv: i32): i32
@external("wastrumentation_stack", "load_arg1_ret_i32_arg_i32_i32")
declare function load_arg1_ret_i32_arg_i32_i32(argv: i32): i32
@external("wastrumentation_stack", "load_ret0_ret_i32_arg_i32_i32")
declare function load_ret0_ret_i32_arg_i32_i32(argv: i32): i32

// around
export function generic_apply(f_apply: i32, argc: i32, argv: i32): void {
    // before
    const a0 = load_arg0_ret_i32_arg_i32_i32(argv);
    const a1 = load_arg1_ret_i32_arg_i32_i32(argv);
    console.log(` Pre: Signature with room for ${argc}, a0 = ${a0}, a1 = ${a1}`);

    // compute base
    call_base(f_apply, argv);

    // after
    const r0 = load_ret0_ret_i32_arg_i32_i32(argv);
    console.log(`Post: Signature with room for ${argc}, a0 = ${a0}, a1 = ${a1}, r0 = ${r0}`);
}
