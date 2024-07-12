(module
    (; ============ TYPES ============ ;)
    (type $void=>void
        (func))
    (type $i32=>void
        (func (param i32) (; => ;) (result)))
    (type $void=>i32
        (func (; => ;) (result i32)))
    (type $void=>i32_i32
        (func (; => ;) (result i32 i32)))
    (type $void=>i32_i32_i32
        (func (; => ;) (result i32 i32 i32)))
    (type $i32=>i32_i32
        (func (param i32) (; => ;) (result i32 i32)))
    (type $i32_i32=>i32
        (func (param i32) (param i32) (; => ;) (result i32)))
    (type $i32_i32_i32=>void
        (func (param i32) (param i32) (param i32) (; => ;) (result)))
    (type $i32_f32_i64_f64=>f64_i64_f32_i32
        (func (param i32) (param f32) (param i64) (param f64) (; => ;) (result f64 i64 f32 i32)))

    (; ============ FUNCTIONS ============ ;)
    (func $assert (param $condition i32)
        (; asserts for the given condition, otherwise trap ;)
        (if (i32.eqz (local.get $condition))
            (then (unreachable))
            (else nop)))
    (func $void=>void (type $void=>void)
        (; no body ;))
    (func $i32=>void (type $i32=>void)
        (; no body ;))
    (func $void=>i32 (type $void=>i32)
        (; return the constant 1, one time ;)
        (i32.const 1))
    (func $void=>i32_i32 (type $void=>i32_i32)
        (; return the constant 2, two times ;)
        (i32.const 2)
        (i32.const 2))
    (func $void=>i32_i32_i32 (type $void=>i32_i32_i32)
        (; return the constant 3, three times ;)
        (i32.const 3)
        (i32.const 3)
        (i32.const 3))
    (func $i32=>i32_i32 (type $i32=>i32_i32)
        (; return the first argument, twice ;)
        (local.get 0)
        (local.get 0))
    (func $i32_i32=>i32 (type $i32_i32=>i32)
        (; return the first argument ;)
        (local.get 0))
    (func $i32_i32_i32=>void (type $i32_i32_i32=>void)
        (; no body ;))
    (func $i32_f32_i64_f64=>f64_i64_f32_i32 (type $i32_f32_i64_f64=>f64_i64_f32_i32)
        (; return the passed arguments in reverse order ;)
        (local.get 3)
        (local.get 2)
        (local.get 1)
        (local.get 0))

    (func $execute_tests
        (; [TEST] ;)
        (call $void=>void)

        (; [TEST] ;)
        (call $i32=>void (i32.const 123456))

        (; [TEST] ;)
        (call $void=>i32)
        (call $assert (i32.eq (i32.const 1)))

        (; [TEST] ;)
        (call $void=>i32_i32)
        (call $assert (i32.eq (i32.const 2)))
        (call $assert (i32.eq (i32.const 2)))

        (; [TEST] ;)
        (call $void=>i32_i32_i32)
        (call $assert (i32.eq (i32.const 3)))
        (call $assert (i32.eq (i32.const 3)))
        (call $assert (i32.eq (i32.const 3)))

        (; [TEST] ;)
        (call $i32=>i32_i32
            (i32.const 4))
        (call $assert (i32.eq (i32.const 4)))
        (call $assert (i32.eq (i32.const 4)))

        (; [TEST] ;)
        (call $i32_i32=>i32
            (i32.const 5)
            (i32.const 55))
        (call $assert (i32.eq (i32.const 5)))

        (; [TEST] ;)
        (call $i32_i32_i32=>void
            (i32.const 6)
            (i32.const 66)
            (i32.const 666))

        (; [TEST] ;)
        (call $i32_f32_i64_f64=>f64_i64_f32_i32
            (i32.const 7)
            (f32.const 77)
            (i64.const 777)
            (f64.const 7777))
        (call $assert (i32.eq (i32.const 7)))
        (call $assert (f32.eq (f32.const 77)))
        (call $assert (i64.eq (i64.const 777)))
        (call $assert (f64.eq (f64.const 7777))))

    (; ============ EXPORTS ============ ;)
    (export "execute_tests" (func $execute_tests))
)
