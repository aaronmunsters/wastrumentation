
(module
  (type (func (result i32)))  ;; type #0
  (func (result i32) (i32.const 0))  ;; index 0
  (table $t0 1 1 funcref)
  (table $t1 1 1 funcref)
  (elem (table $t0) (i32.const 0) func 0)
  (func (export "copy-t0-to-t1")
    (table.copy $t1 $t0 (i32.const 0) (i32.const 0) (i32.const 1))
    )
  (func (export "check_t0") (param i32) (result i32)
    (call_indirect $t0 (type 0) (local.get 0)))
)
