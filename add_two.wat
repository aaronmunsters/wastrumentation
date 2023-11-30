(module
    (func $add_two (param $a i32) (param $b i32) (result i32)
        (local.get $a)
        (local.get $b)
        (i32.add))
    (export "add-two" (func $add_two)))