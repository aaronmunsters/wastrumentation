(module
  (type (;0;) (func (param i32 i32 i32 i32 i32)))
  (type (;1;) (func (param i32 i32) (result i32)))
  (type (;2;) (func (result i32)))
  (type (;3;) (func))
  (type (;4;) (func (param i32) (result i32)))
  (type (;5;) (func (param i32 i32)))
  (type (;6;) (func (param i32 i32 i32)))
  (type (;7;) (func (param i32)))
  (import "WASP_ANALYSIS" "generic_apply" (func (;0;) (type 0)))
  (import "wastrumentation_stack" "allocate_ret_i32_arg_i32_i32" (func (;1;) (type 1)))
  (import "wastrumentation_stack" "allocate_types_ret_i32_arg_i32_i32" (func (;2;) (type 2)))
  (import "wastrumentation_stack" "free_ret_i32_arg_i32_i32" (func (;3;) (type 3)))
  (import "wastrumentation_stack" "load_arg0_ret_i32_arg_i32_i32" (func (;4;) (type 4)))
  (import "wastrumentation_stack" "load_arg1_ret_i32_arg_i32_i32" (func (;5;) (type 4)))
  (import "wastrumentation_stack" "load_ret0_ret_i32_arg_i32_i32" (func (;6;) (type 4)))
  (import "wastrumentation_stack" "store_arg0_ret_i32_arg_i32_i32" (func (;7;) (type 5)))
  (import "wastrumentation_stack" "store_arg1_ret_i32_arg_i32_i32" (func (;8;) (type 5)))
  (import "wastrumentation_stack" "store_args_ret_i32_arg_i32_i32" (func (;9;) (type 6)))
  (import "wastrumentation_stack" "store_ret0_ret_i32_arg_i32_i32" (func (;10;) (type 5)))
  (import "wastrumentation_stack" "store_rets_ret_i32_arg_i32_i32" (func (;11;) (type 5)))
  (func (;12;) (type 1) (param i32 i32) (result i32)
    (local i32 i32)
    local.get 0
    local.get 1
    call 1
    local.set 2
    call 2
    local.set 3
    i32.const 0
    i32.const 2
    i32.const 1
    local.get 2
    local.get 3
    call 0
    local.get 2
    call 6
    call 3)
  (func (;13;) (type 1) (param i32 i32) (result i32)
    local.get 0
    i32.const 1
    i32.le_s
    if (result i32)  ;; label = @1
      i32.const 1
    else
      local.get 0
      i32.const 1
      i32.sub
      local.get 1
      call 12
      local.get 0
      i32.mul
    end)
  (func (;14;) (type 7) (param i32)
    (local i32)
    local.get 0
    local.get 0
    call 4
    local.get 0
    call 5
    call 13
    call 11)
  (func (;15;) (type 5) (param i32 i32)
    local.get 1
    local.get 0
    call_indirect (type 7))
  (table (;0;) 1 1 funcref)
  (memory (;0;) 0)
  (export "add-two" (func 12))
  (export "call_base" (func 15))
  (elem (;0;) (i32.const 0) func 14))
