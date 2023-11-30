(module
 (type $0 (func (param i32 i32) (result i32)))
 (memory $0 0)
 (export "add-two" (func $module/fac))
 (func $module/fac (param $0 i32) (param $1 i32) (result i32)
  local.get $0
  i32.const 1
  i32.le_s
  if (result i32)
   i32.const 1
  else
   local.get $0
   i32.const 1
   i32.sub
   local.get $1
   call $module/fac
   local.get $0
   i32.mul
  end
 )
)
