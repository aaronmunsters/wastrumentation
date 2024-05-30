(module
    (export "fib" (func $fib))
    (func $fib (param $0 i32) (result i32)
        local.get $0
        i32.const 1
        i32.le_s
        if (result i32)
            i32.const 1
        else
            local.get $0
            i32.const 1
            i32.sub
            call $fib
            local.get $0
            i32.const 2
            i32.sub
            call $fib
            i32.add
        end
    )
)
