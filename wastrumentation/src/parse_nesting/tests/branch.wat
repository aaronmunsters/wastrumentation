(module
    (func $foo
        (i32.const 0)
        (if (result i32)
            (then (i32.const 1))
            (else (i32.const 2)))
        (drop)))
