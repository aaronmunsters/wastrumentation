(module
  (type $void=>i32 (func (;void;) (; => ;) (result i32)))
  (func $0 (type $void=>i32) (i32.const 0)) ;; index 0
  (func $1 (type $void=>i32) (i32.const 1)) ;; index 1
  (func $2 (type $void=>i32) (i32.const 2)) ;; index 2
  (func $3 (type $void=>i32) (i32.const 3)) ;; index 3
  (func $4 (type $void=>i32) (i32.const 4)) ;; index 4
  (func $5 (type $void=>i32) (i32.const 5)) ;; index 5
  (func $6 (type $void=>i32) (i32.const 6)) ;; index 6
  (func $7 (type $void=>i32) (i32.const 7)) ;; index 7

  ;; table $t0, of exact size 17 (min == max == 17) storing funcref's
  (table $t0 17 17 funcref)
  ;;         00 01 02 03 04 05 06 07 08 09 10 11 12 13 14 15 16
  ;; $t0 := [ _  _ $3 $1 $4 $1  _  _  _  _  _  _ $7 $5 $2 $3 $6]
  (elem (table $t0) (i32.const 2)  func $3 $1 $4 $1)
  (elem (table $t0) (i32.const 12) func $7 $5 $2 $3 $6)

  (func (export "check_t0") (param i32) (result i32)
    (call_indirect $t0 (type $void=>i32) (local.get 0))))
