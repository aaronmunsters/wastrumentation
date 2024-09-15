(module
    (export "select" (func $select))
    (func $select (param $a i32) (param $b i32) (param $c i32) (result i32)
        (local.get $a)
        (local.get $b)
        (local.get $c)
        (select)))
