(module
  (table $table_0 10 funcref)
  (elem $elem_0 func $double $triple)
  (type $type_0 (func (param i32) (result i32)))

  (func $double (param $x i32) (result i32)
    local.get $x
    i32.const 2
    i32.mul
  )

  (func $triple (param $x i32) (result i32)
    local.get $x
    i32.const 3
    i32.mul
  )

  (func $assert_eq (param $a i32) (param $b i32)
    local.get $a
    local.get $b
    i32.ne
    if
      unreachable
    end
  )


  (func (export "main") (result i32)

    ;; TEST: ref.func, ref.null and ref.is_null
    ref.func $double
    ref.is_null
    i32.const 0
    call $assert_eq
    ref.null func
    ref.is_null
    i32.const 1
    call $assert_eq

    ;; TEST: table.set
    i32.const 2
    ref.func $double
    table.set $table_0

    ;; TEST: table.get
    i32.const 3
    ref.func $triple
    table.set $table_0

    i32.const 3
    table.get $table_0
    drop

    ;; TEST: call_indirect
    i32.const 21
    i32.const 2
    call_indirect $table_0 (type $type_0)
    i32.const 42
    call $assert_eq

    ;; TEST: table.grow
    ref.null func       ;; init null func ref to grow the table with
    i32.const 5         ;; size to grow the table by
    table.grow $table_0 ;; grow, returns old size
    i32.const 10
    call $assert_eq

    ;; TEST: table.size
    table.size $table_0 ;; size, returns new size
    i32.const 15
    call $assert_eq

    ;; TEST: table.fill
    i32.const 5         ;; start index
    ref.func $double    ;; init double func ref to fill the table with
    i32.const 8         ;; size to fill the table with
    table.fill $table_0 ;; fill
    i32.const 32
    i32.const 7
    call_indirect $table_0 (type $type_0)
    i32.const 64
    call $assert_eq

    ;; TEST: table.copy
    i32.const 7                   ;; dest index
    i32.const 2                   ;; source index (funcref to $double)
    i32.const 1                   ;; number of elements to copy
    table.copy $table_0 $table_0  ;; copy within the table

    i32.const 14                  ;; operand for double
    i32.const 7                   ;; index of copied funcref (to $double)
    call_indirect $table_0 (type $type_0)
    i32.const 28
    call $assert_eq

    ;; TEST: table.init
    i32.const 0                   ;; dest index
    i32.const 0                   ;; elem segment index (funcref to $double and $triple)
    i32.const 2                   ;; number of elements to init
    table.init $table_0 $elem_0   ;; init table from elem segment
    i32.const 3                   ;; operand for triple
    i32.const 1                   ;; index of init funcref (to $triple)
    call_indirect $table_0 (type $type_0)
    i32.const 9
    call $assert_eq

    ;; TEST: elem.drop
    elem.drop $elem_0
    ;; i32.const 0                   ;; dest index
    ;; i32.const 0                   ;; elem segment index
    ;; i32.const 2                   ;; number of elements to init
    ;; table.init $table_0 $elem_0   ;; init table from elem segment (should trap)

    i32.const 0
  )
)
