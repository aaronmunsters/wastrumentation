(module
    (export "add_three" (func $add_three))
    (func $add_three (param $a i32) (param $b i32) (param $c i32) (result i32)
        (local.get $a)
        (local.get $b)
        (local.get $c)
        (i32.add)
        (i32.add)))
