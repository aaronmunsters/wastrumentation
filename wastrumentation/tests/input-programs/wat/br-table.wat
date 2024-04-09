(module
    (export "main" (func $main))
    (func $main (param $a i32) (result i32)
        (block $yield_-1
            (block $yield_0
                (block $yield_1
                    (block $yield_2
                        (local.get $a)
                        ;;                      branch table
                        ;;           ____________________________________
                        (br_table (; [ ;) $yield_0 $yield_1 $yield_2 (; ] ;) $yield_-1)
                        ;;           ''''''''''''''''''
                        ;;                                                    default
                    )
                    (i32.const 2)
                    (return)
                )
                (i32.const 1)
                (return)
            )
            (i32.const 0)
            (return)
        )
        (i32.const -1)
        (return)
    )
)