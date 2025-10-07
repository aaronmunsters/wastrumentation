(module
    ;; Memory management
    (memory 1)
    (global $stack_free_ptr (mut i32) (i32.const 0))
    (global $total_memory   (mut i32) (i32.const 1))

    ;; Constants
    (global $memory_growth_size i32 (i32.const     1))
    (global $page_byte_size     i32 (i32.const 65536))
    (global $size_wasm_value    i32 (i32.const     8)) ;; sizeof i64
    (global $size_wasm_type     i32 (i32.const     4)) ;; sizeof i32

    ;; Grow memory
    (func $grow_memory
        (if (i32.eq (memory.grow (global.get $memory_growth_size)) (i32.const -1))
            (then unreachable)
            (else
                (global.set
                    $total_memory
                    (i32.add
                        (global.get $total_memory)
                        (global.get $memory_growth_size))))))

    ;; Calculate total used memory in bytes
    (func $total_used_memory_in_bytes (result i32)
        (i32.mul (global.get $total_memory) (global.get $page_byte_size)))

    ;; Stack allocate raw bytes
    (func $stack_allocate (param $bytes i32) (result i32)
        (local $stack_free_ptr_before i32)
        (local $stack_free_ptr_after i32)

        (local.set $stack_free_ptr_before (global.get $stack_free_ptr))
        (local.set
            $stack_free_ptr_after
            (i32.add (global.get $stack_free_ptr) (local.get $bytes)))

        ;; Grow memory if needed
        (loop $grow_loop
            (if (i32.gt_u (local.get $stack_free_ptr_after) (call $total_used_memory_in_bytes))
                (then
                    (call $grow_memory)
                    (br $grow_loop))))

        (global.set $stack_free_ptr (local.get $stack_free_ptr_after))
        (local.get $stack_free_ptr_before))

    ;; Stack deallocate
    (func $stack_deallocate (param $ptr i32) (param $bytes i32)
        (global.set
            $stack_free_ptr
            (i32.sub (global.get $stack_free_ptr) (local.get $bytes))))

    ;; Stack allocate for WASM values (i64 slots)
    (func $stack_allocate_values (param $count i32) (result i32)
        (call $stack_allocate
            (i32.mul (local.get $count) (global.get $size_wasm_value))))

    ;; Stack deallocate for WASM values
    (func $stack_deallocate_values (param $ptr i32) (param $count i32)
        (call $stack_deallocate (local.get $ptr)
            (i32.mul (local.get $count) (global.get $size_wasm_value))))

    ;; Stack allocate for WASM types (i32 slots)
    (func $stack_allocate_types (param $count i32) (result i32)
        (call $stack_allocate
            (i32.mul (local.get $count) (global.get $size_wasm_type))))

    ;; Stack deallocate for WASM types
    (func $stack_deallocate_types (param $ptr i32) (param $count i32)
        (call $stack_deallocate (local.get $ptr)
            (i32.mul (local.get $count) (global.get $size_wasm_type))))

    ;; Memory load/store operations
    (func $wastrumentation_memory_load (param $ptr i32) (param $offset i32) (result i64)
        (i64.load
            (i32.add
                (local.get $ptr)
                (i32.mul
                    (local.get $offset)
                    (global.get $size_wasm_value)))))

    (func $wastrumentation_memory_store (param $ptr i32) (param $value i64) (param $offset i32)
        (i64.store
            (i32.add
                (local.get $ptr)
                (i32.mul
                    (local.get $offset)
                    (global.get $size_wasm_value)))
            (local.get $value)))

    ;; Type operations
    (func (export "wastrumentation_stack_load_type") (param $ptr i32) (param $offset i32) (result i32)
        (i32.load
            (i32.add
                (local.get $ptr)
                (i32.mul
                    (local.get $offset)
                    (global.get $size_wasm_type)))))

    (func $wastrumentation_stack_store_type (export "wastrumentation_stack_store_type") (param $ptr i32) (param $offset i32) (param $ty i32)
        (i32.store
            (i32.add
                (local.get $ptr)
                (i32.mul
                    (local.get $offset)
                    (global.get $size_wasm_type)))
            (local.get $ty)))

    ;; Typed load/store operations for export
    (func (export "wastrumentation_stack_load_i32") (param $ptr i32) (param $offset i32) (result i32)
        (call $wastrumentation_memory_load (local.get $ptr) (local.get $offset))
        i32.wrap_i64)

    (func (export "wastrumentation_stack_load_f32") (param $ptr i32) (param $offset i32) (result f32)
        (call $wastrumentation_memory_load (local.get $ptr) (local.get $offset))
        i32.wrap_i64
        f32.reinterpret_i32)

    (func (export "wastrumentation_stack_load_i64") (param $ptr i32) (param $offset i32) (result i64)
        (call $wastrumentation_memory_load (local.get $ptr) (local.get $offset)))

    (func (export "wastrumentation_stack_load_f64") (param $ptr i32) (param $offset i32) (result f64)
        (call $wastrumentation_memory_load (local.get $ptr) (local.get $offset))
        f64.reinterpret_i64)

    (func (export "wastrumentation_stack_store_i32") (param $ptr i32) (param $value i32) (param $offset i32)
        (call $wastrumentation_memory_store (local.get $ptr)
        (i64.extend_i32_u (local.get $value)) (local.get $offset)))

    (func (export "wastrumentation_stack_store_f32") (param $ptr i32) (param $value f32) (param $offset i32)
        (call $wastrumentation_memory_store (local.get $ptr)
        (i64.extend_i32_u (i32.reinterpret_f32 (local.get $value))) (local.get $offset)))

    (func (export "wastrumentation_stack_store_i64") (param $ptr i32) (param $value i64) (param $offset i32)
        (call $wastrumentation_memory_store (local.get $ptr) (local.get $value) (local.get $offset)))

    (func (export "wastrumentation_stack_store_f64") (param $ptr i32) (param $value f64) (param $offset i32)
        (call $wastrumentation_memory_store (local.get $ptr)
        (i64.reinterpret_f64 (local.get $value)) (local.get $offset)))

    (type $type_ret_arg (func ))
    (type $type_ret_f32_f64_arg_i32_i64 (func (param i32) (param i64) (result f32 f64)))
    (type $type_ret_f64_f32_arg_i32_i64 (func (param i32) (param i64) (result f64 f32)))
    (type $type_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64 (func (param i64) (param i32) (param f32) (param f64) (result f64 f32 i32 i64)))

    (func $allocate_ret_0_arg_0 (result i32) (i32.const 0))
    (func $free_values_ret_0_arg_0 (param $ptr i32)
        (;No deallocation needed for empty signatures;))
    (func $allocate_ret_2_arg_2 (param $a0 i64) (param $a1 i64) (result i32)
        (local $stack_ptr i32)        ;; Allocate 4 slots
        (local.set $stack_ptr (call $stack_allocate_values (i32.const 4)))
        ;; Store argument 0
        (local.get $stack_ptr)
        (local.get $a0)
        (i32.const 2)
        (call $wastrumentation_memory_store)
        ;; Store argument 1
        (local.get $stack_ptr)
        (local.get $a1)
        (i32.const 3)
        (call $wastrumentation_memory_store)
        (local.get $stack_ptr))
    (func $load_arg0_ret_2_arg_2 (param $stack_ptr i32) (result i64)
        (local.get $stack_ptr)
        (i32.const 2)
        (call $wastrumentation_memory_load))
    (func $load_arg1_ret_2_arg_2 (param $stack_ptr i32) (result i64)
        (local.get $stack_ptr)
        (i32.const 3)
        (call $wastrumentation_memory_load))
    (func $load_ret0_ret_2_arg_2 (param $stack_ptr i32) (result i64)
        (local.get $stack_ptr)
        (i32.const 0)
        (call $wastrumentation_memory_load))
    (func $load_ret1_ret_2_arg_2 (param $stack_ptr i32) (result i64)
        (local.get $stack_ptr)
        (i32.const 1)
        (call $wastrumentation_memory_load))
    (func $store_arg0_ret_2_arg_2 (param $stack_ptr i32) (param $value i64)
        (local.get $stack_ptr)
        (local.get $value)
        (i32.const 2)
        (call $wastrumentation_memory_store))
    (func $store_arg1_ret_2_arg_2 (param $stack_ptr i32) (param $value i64)
        (local.get $stack_ptr)
        (local.get $value)
        (i32.const 3)
        (call $wastrumentation_memory_store))
    (func $store_ret0_ret_2_arg_2 (param $stack_ptr i32) (param $value i64)
        (local.get $stack_ptr)
        (local.get $value)
        (i32.const 0)
        (call $wastrumentation_memory_store))
    (func $store_ret1_ret_2_arg_2 (param $stack_ptr i32) (param $value i64)
        (local.get $stack_ptr)
        (local.get $value)
        (i32.const 1)
        (call $wastrumentation_memory_store))
    (func $free_values_ret_2_arg_2 (param $ptr i32)
        (local.get $ptr)
        (i32.const 4)
        (call $stack_deallocate_values))
    (func $allocate_ret_4_arg_4 (param $a0 i64) (param $a1 i64) (param $a2 i64) (param $a3 i64) (result i32)
        (local $stack_ptr i32)        ;; Allocate 8 slots
        (local.set $stack_ptr (call $stack_allocate_values (i32.const 8)))
        ;; Store argument 0
        (local.get $stack_ptr)
        (local.get $a0)
        (i32.const 4)
        (call $wastrumentation_memory_store)
        ;; Store argument 1
        (local.get $stack_ptr)
        (local.get $a1)
        (i32.const 5)
        (call $wastrumentation_memory_store)
        ;; Store argument 2
        (local.get $stack_ptr)
        (local.get $a2)
        (i32.const 6)
        (call $wastrumentation_memory_store)
        ;; Store argument 3
        (local.get $stack_ptr)
        (local.get $a3)
        (i32.const 7)
        (call $wastrumentation_memory_store)
        (local.get $stack_ptr))
    (func $load_arg0_ret_4_arg_4 (param $stack_ptr i32) (result i64)
        (local.get $stack_ptr)
        (i32.const 4)
        (call $wastrumentation_memory_load))
    (func $load_arg1_ret_4_arg_4 (param $stack_ptr i32) (result i64)
        (local.get $stack_ptr)
        (i32.const 5)
        (call $wastrumentation_memory_load))
    (func $load_arg2_ret_4_arg_4 (param $stack_ptr i32) (result i64)
        (local.get $stack_ptr)
        (i32.const 6)
        (call $wastrumentation_memory_load))
    (func $load_arg3_ret_4_arg_4 (param $stack_ptr i32) (result i64)
        (local.get $stack_ptr)
        (i32.const 7)
        (call $wastrumentation_memory_load))
    (func $load_ret0_ret_4_arg_4 (param $stack_ptr i32) (result i64)
        (local.get $stack_ptr)
        (i32.const 0)
        (call $wastrumentation_memory_load))
    (func $load_ret1_ret_4_arg_4 (param $stack_ptr i32) (result i64)
        (local.get $stack_ptr)
        (i32.const 1)
        (call $wastrumentation_memory_load))
    (func $load_ret2_ret_4_arg_4 (param $stack_ptr i32) (result i64)
        (local.get $stack_ptr)
        (i32.const 2)
        (call $wastrumentation_memory_load))
    (func $load_ret3_ret_4_arg_4 (param $stack_ptr i32) (result i64)
        (local.get $stack_ptr)
        (i32.const 3)
        (call $wastrumentation_memory_load))
    (func $store_arg0_ret_4_arg_4 (param $stack_ptr i32) (param $value i64)
        (local.get $stack_ptr)
        (local.get $value)
        (i32.const 4)
        (call $wastrumentation_memory_store))
    (func $store_arg1_ret_4_arg_4 (param $stack_ptr i32) (param $value i64)
        (local.get $stack_ptr)
        (local.get $value)
        (i32.const 5)
        (call $wastrumentation_memory_store))
    (func $store_arg2_ret_4_arg_4 (param $stack_ptr i32) (param $value i64)
        (local.get $stack_ptr)
        (local.get $value)
        (i32.const 6)
        (call $wastrumentation_memory_store))
    (func $store_arg3_ret_4_arg_4 (param $stack_ptr i32) (param $value i64)
        (local.get $stack_ptr)
        (local.get $value)
        (i32.const 7)
        (call $wastrumentation_memory_store))
    (func $store_ret0_ret_4_arg_4 (param $stack_ptr i32) (param $value i64)
        (local.get $stack_ptr)
        (local.get $value)
        (i32.const 0)
        (call $wastrumentation_memory_store))
    (func $store_ret1_ret_4_arg_4 (param $stack_ptr i32) (param $value i64)
        (local.get $stack_ptr)
        (local.get $value)
        (i32.const 1)
        (call $wastrumentation_memory_store))
    (func $store_ret2_ret_4_arg_4 (param $stack_ptr i32) (param $value i64)
        (local.get $stack_ptr)
        (local.get $value)
        (i32.const 2)
        (call $wastrumentation_memory_store))
    (func $store_ret3_ret_4_arg_4 (param $stack_ptr i32) (param $value i64)
        (local.get $stack_ptr)
        (local.get $value)
        (i32.const 3)
        (call $wastrumentation_memory_store))
    (func $free_values_ret_4_arg_4 (param $ptr i32)
        (local.get $ptr)
        (i32.const 8)
        (call $stack_deallocate_values))
    (func (export "allocate_ret_arg") (result i32)
        (call $allocate_ret_0_arg_0))
    (func (export "store_rets_ret_arg") (param $stack_ptr i32)
        (;No return values to store;))
    (func (export "allocate_types_ret_arg") (result i32)
        (i32.const 0))
    (func (export "free_types_ret_arg") (param $ptr i32)
        (;No deallocation needed for empty signatures;))
    (func (export "free_values_ret_arg") (param $ptr i32)
        (call $free_values_ret_0_arg_0 (local.get $ptr)))
    (func (export "allocate_ret_f32_f64_arg_i32_i64") (param $a0 i32) (param $a1 i64) (result i32)
        (local.get $a0)
        (i64.extend_i32_u)
        (local.get $a1)
        (call $allocate_ret_2_arg_2))
    (func (export "load_arg0_ret_f32_f64_arg_i32_i64") (param $stack_ptr i32) (result i32)
        local.get $stack_ptr
        (call $load_arg0_ret_2_arg_2)
        (i32.wrap_i64))
    (func (export "load_arg1_ret_f32_f64_arg_i32_i64") (param $stack_ptr i32) (result i64)
        local.get $stack_ptr
        (call $load_arg1_ret_2_arg_2)
        )
    (func (export "load_ret0_ret_f32_f64_arg_i32_i64") (param $stack_ptr i32) (result f32)
        local.get $stack_ptr
        (call $load_ret0_ret_2_arg_2)
        (i32.wrap_i64) (;then;) (f32.reinterpret_i32))
    (func (export "load_ret1_ret_f32_f64_arg_i32_i64") (param $stack_ptr i32) (result f64)
        local.get $stack_ptr
        (call $load_ret1_ret_2_arg_2)
        (f64.reinterpret_i64))
    (func (export "store_arg0_ret_f32_f64_arg_i32_i64") (param $stack_ptr i32) (param $value i32)
        local.get $stack_ptr
        local.get $value
        (i64.extend_i32_u)
        (call $store_arg0_ret_2_arg_2))
    (func (export "store_arg1_ret_f32_f64_arg_i32_i64") (param $stack_ptr i32) (param $value i64)
        local.get $stack_ptr
        local.get $value
        (call $store_arg1_ret_2_arg_2))
    (func (export "store_ret0_ret_f32_f64_arg_i32_i64") (param $stack_ptr i32) (param $value f32)
        local.get $stack_ptr
        local.get $value
        (i32.reinterpret_f32) (;then;) (i64.extend_i32_u)
        (call $store_ret0_ret_2_arg_2))
    (func (export "store_ret1_ret_f32_f64_arg_i32_i64") (param $stack_ptr i32) (param $value f64)
        local.get $stack_ptr
        local.get $value
        (i64.reinterpret_f64)
        (call $store_ret1_ret_2_arg_2))
    (func (export "store_rets_ret_f32_f64_arg_i32_i64") (param $stack_ptr i32) (param $ret0 f32) (param $ret1 f64)
        ;; Store return value 0
        local.get $stack_ptr
        local.get $ret0
        (i32.reinterpret_f32) (;then;) (i64.extend_i32_u)
        (call $store_ret0_ret_2_arg_2)
        ;; Store return value 1
        local.get $stack_ptr
        local.get $ret1
        (i64.reinterpret_f64)
        (call $store_ret1_ret_2_arg_2)
    )
    (func (export "allocate_types_ret_f32_f64_arg_i32_i64") (result i32)
        (local $types_buffer i32)
        ;; Allocate types buffer
        (local.set $types_buffer (call $stack_allocate_types (i32.const 4)))
        ;; Fill in the types
        ;; Store return type 0
        (local.get $types_buffer)
        (i32.const 0)
        (i32.const 1)
        (call $wastrumentation_stack_store_type )
        ;; Store return type 1
        (local.get $types_buffer)
        (i32.const 1)
        (i32.const 3)
        (call $wastrumentation_stack_store_type )
        ;; Store argument type 0
        (local.get $types_buffer)
        (i32.const 2)
        (i32.const 0)
        (call $wastrumentation_stack_store_type)
        ;; Store argument type 1
        (local.get $types_buffer)
        (i32.const 3)
        (i32.const 2)
        (call $wastrumentation_stack_store_type)

        ;; Return the buffer
        (local.get $types_buffer))
    (func (export "free_types_ret_f32_f64_arg_i32_i64") (param $ptr i32)
        (call $stack_deallocate_types (local.get $ptr) (i32.const 4)))
    (func (export "free_values_ret_f32_f64_arg_i32_i64") (param $ptr i32)
        (call $free_values_ret_2_arg_2 (local.get $ptr)))
    (func (export "allocate_ret_f64_f32_arg_i32_i64") (param $a0 i32) (param $a1 i64) (result i32)
        (local.get $a0)
        (i64.extend_i32_u)
        (local.get $a1)
        (call $allocate_ret_2_arg_2))
    (func (export "load_arg0_ret_f64_f32_arg_i32_i64") (param $stack_ptr i32) (result i32)
        local.get $stack_ptr
        (call $load_arg0_ret_2_arg_2)
        (i32.wrap_i64))
    (func (export "load_arg1_ret_f64_f32_arg_i32_i64") (param $stack_ptr i32) (result i64)
        local.get $stack_ptr
        (call $load_arg1_ret_2_arg_2)
        )
    (func (export "load_ret0_ret_f64_f32_arg_i32_i64") (param $stack_ptr i32) (result f64)
        local.get $stack_ptr
        (call $load_ret0_ret_2_arg_2)
        (f64.reinterpret_i64))
    (func (export "load_ret1_ret_f64_f32_arg_i32_i64") (param $stack_ptr i32) (result f32)
        local.get $stack_ptr
        (call $load_ret1_ret_2_arg_2)
        (i32.wrap_i64) (;then;) (f32.reinterpret_i32))
    (func (export "store_arg0_ret_f64_f32_arg_i32_i64") (param $stack_ptr i32) (param $value i32)
        local.get $stack_ptr
        local.get $value
        (i64.extend_i32_u)
        (call $store_arg0_ret_2_arg_2))
    (func (export "store_arg1_ret_f64_f32_arg_i32_i64") (param $stack_ptr i32) (param $value i64)
        local.get $stack_ptr
        local.get $value
        (call $store_arg1_ret_2_arg_2))
    (func (export "store_ret0_ret_f64_f32_arg_i32_i64") (param $stack_ptr i32) (param $value f64)
        local.get $stack_ptr
        local.get $value
        (i64.reinterpret_f64)
        (call $store_ret0_ret_2_arg_2))
    (func (export "store_ret1_ret_f64_f32_arg_i32_i64") (param $stack_ptr i32) (param $value f32)
        local.get $stack_ptr
        local.get $value
        (i32.reinterpret_f32) (;then;) (i64.extend_i32_u)
        (call $store_ret1_ret_2_arg_2))
    (func (export "store_rets_ret_f64_f32_arg_i32_i64") (param $stack_ptr i32) (param $ret0 f64) (param $ret1 f32)
        ;; Store return value 0
        local.get $stack_ptr
        local.get $ret0
        (i64.reinterpret_f64)
        (call $store_ret0_ret_2_arg_2)
        ;; Store return value 1
        local.get $stack_ptr
        local.get $ret1
        (i32.reinterpret_f32) (;then;) (i64.extend_i32_u)
        (call $store_ret1_ret_2_arg_2)
    )
    (func (export "allocate_types_ret_f64_f32_arg_i32_i64") (result i32)
        (local $types_buffer i32)
        ;; Allocate types buffer
        (local.set $types_buffer (call $stack_allocate_types (i32.const 4)))
        ;; Fill in the types
        ;; Store return type 0
        (local.get $types_buffer)
        (i32.const 0)
        (i32.const 3)
        (call $wastrumentation_stack_store_type )
        ;; Store return type 1
        (local.get $types_buffer)
        (i32.const 1)
        (i32.const 1)
        (call $wastrumentation_stack_store_type )
        ;; Store argument type 0
        (local.get $types_buffer)
        (i32.const 2)
        (i32.const 0)
        (call $wastrumentation_stack_store_type)
        ;; Store argument type 1
        (local.get $types_buffer)
        (i32.const 3)
        (i32.const 2)
        (call $wastrumentation_stack_store_type)

        ;; Return the buffer
        (local.get $types_buffer))
    (func (export "free_types_ret_f64_f32_arg_i32_i64") (param $ptr i32)
        (call $stack_deallocate_types (local.get $ptr) (i32.const 4)))
    (func (export "free_values_ret_f64_f32_arg_i32_i64") (param $ptr i32)
        (call $free_values_ret_2_arg_2 (local.get $ptr)))
    (func (export "allocate_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64") (param $a0 i64) (param $a1 i32) (param $a2 f32) (param $a3 f64) (result i32)
        (local.get $a0)
        (local.get $a1)
        (i64.extend_i32_u)
        (local.get $a2)
        (i32.reinterpret_f32) (;then;) (i64.extend_i32_u)
        (local.get $a3)
        (i64.reinterpret_f64)
        (call $allocate_ret_4_arg_4))
    (func (export "load_arg0_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64") (param $stack_ptr i32) (result i64)
        local.get $stack_ptr
        (call $load_arg0_ret_4_arg_4)
        )
    (func (export "load_arg1_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64") (param $stack_ptr i32) (result i32)
        local.get $stack_ptr
        (call $load_arg1_ret_4_arg_4)
        (i32.wrap_i64))
    (func (export "load_arg2_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64") (param $stack_ptr i32) (result f32)
        local.get $stack_ptr
        (call $load_arg2_ret_4_arg_4)
        (i32.wrap_i64) (;then;) (f32.reinterpret_i32))
    (func (export "load_arg3_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64") (param $stack_ptr i32) (result f64)
        local.get $stack_ptr
        (call $load_arg3_ret_4_arg_4)
        (f64.reinterpret_i64))
    (func (export "load_ret0_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64") (param $stack_ptr i32) (result f64)
        local.get $stack_ptr
        (call $load_ret0_ret_4_arg_4)
        (f64.reinterpret_i64))
    (func (export "load_ret1_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64") (param $stack_ptr i32) (result f32)
        local.get $stack_ptr
        (call $load_ret1_ret_4_arg_4)
        (i32.wrap_i64) (;then;) (f32.reinterpret_i32))
    (func (export "load_ret2_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64") (param $stack_ptr i32) (result i32)
        local.get $stack_ptr
        (call $load_ret2_ret_4_arg_4)
        (i32.wrap_i64))
    (func (export "load_ret3_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64") (param $stack_ptr i32) (result i64)
        local.get $stack_ptr
        (call $load_ret3_ret_4_arg_4)
        )
    (func (export "store_arg0_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64") (param $stack_ptr i32) (param $value i64)
        local.get $stack_ptr
        local.get $value
        (call $store_arg0_ret_4_arg_4))
    (func (export "store_arg1_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64") (param $stack_ptr i32) (param $value i32)
        local.get $stack_ptr
        local.get $value
        (i64.extend_i32_u)
        (call $store_arg1_ret_4_arg_4))
    (func (export "store_arg2_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64") (param $stack_ptr i32) (param $value f32)
        local.get $stack_ptr
        local.get $value
        (i32.reinterpret_f32) (;then;) (i64.extend_i32_u)
        (call $store_arg2_ret_4_arg_4))
    (func (export "store_arg3_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64") (param $stack_ptr i32) (param $value f64)
        local.get $stack_ptr
        local.get $value
        (i64.reinterpret_f64)
        (call $store_arg3_ret_4_arg_4))
    (func (export "store_ret0_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64") (param $stack_ptr i32) (param $value f64)
        local.get $stack_ptr
        local.get $value
        (i64.reinterpret_f64)
        (call $store_ret0_ret_4_arg_4))
    (func (export "store_ret1_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64") (param $stack_ptr i32) (param $value f32)
        local.get $stack_ptr
        local.get $value
        (i32.reinterpret_f32) (;then;) (i64.extend_i32_u)
        (call $store_ret1_ret_4_arg_4))
    (func (export "store_ret2_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64") (param $stack_ptr i32) (param $value i32)
        local.get $stack_ptr
        local.get $value
        (i64.extend_i32_u)
        (call $store_ret2_ret_4_arg_4))
    (func (export "store_ret3_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64") (param $stack_ptr i32) (param $value i64)
        local.get $stack_ptr
        local.get $value
        (call $store_ret3_ret_4_arg_4))
    (func (export "store_rets_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64") (param $stack_ptr i32) (param $ret0 f64) (param $ret1 f32) (param $ret2 i32) (param $ret3 i64)
        ;; Store return value 0
        local.get $stack_ptr
        local.get $ret0
        (i64.reinterpret_f64)
        (call $store_ret0_ret_4_arg_4)
        ;; Store return value 1
        local.get $stack_ptr
        local.get $ret1
        (i32.reinterpret_f32) (;then;) (i64.extend_i32_u)
        (call $store_ret1_ret_4_arg_4)
        ;; Store return value 2
        local.get $stack_ptr
        local.get $ret2
        (i64.extend_i32_u)
        (call $store_ret2_ret_4_arg_4)
        ;; Store return value 3
        local.get $stack_ptr
        local.get $ret3
        (call $store_ret3_ret_4_arg_4)
    )
    (func (export "allocate_types_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64") (result i32)
        (local $types_buffer i32)
        ;; Allocate types buffer
        (local.set $types_buffer (call $stack_allocate_types (i32.const 8)))
        ;; Fill in the types
        ;; Store return type 0
        (local.get $types_buffer)
        (i32.const 0)
        (i32.const 3)
        (call $wastrumentation_stack_store_type )
        ;; Store return type 1
        (local.get $types_buffer)
        (i32.const 1)
        (i32.const 1)
        (call $wastrumentation_stack_store_type )
        ;; Store return type 2
        (local.get $types_buffer)
        (i32.const 2)
        (i32.const 0)
        (call $wastrumentation_stack_store_type )
        ;; Store return type 3
        (local.get $types_buffer)
        (i32.const 3)
        (i32.const 2)
        (call $wastrumentation_stack_store_type )
        ;; Store argument type 0
        (local.get $types_buffer)
        (i32.const 4)
        (i32.const 2)
        (call $wastrumentation_stack_store_type)
        ;; Store argument type 1
        (local.get $types_buffer)
        (i32.const 5)
        (i32.const 0)
        (call $wastrumentation_stack_store_type)
        ;; Store argument type 2
        (local.get $types_buffer)
        (i32.const 6)
        (i32.const 1)
        (call $wastrumentation_stack_store_type)
        ;; Store argument type 3
        (local.get $types_buffer)
        (i32.const 7)
        (i32.const 3)
        (call $wastrumentation_stack_store_type)

        ;; Return the buffer
        (local.get $types_buffer))
    (func (export "free_types_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64") (param $ptr i32)
        (call $stack_deallocate_types (local.get $ptr) (i32.const 8)))
    (func (export "free_values_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64") (param $ptr i32)
        (call $free_values_ret_4_arg_4 (local.get $ptr)))
)
