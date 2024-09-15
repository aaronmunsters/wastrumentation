(module
  (export "nested-blocks" (func $nested-blocks))
  (func $nested-blocks (result i32)
    (local $a i32)
    (local.set $a (i32.const 0))
    (block
        (block
            (block
                (block
                    (block
                        (block
                            (block
                                (block
                                    (block
                                        (block
                                            (block
                                                (block
                                                    (block
                                                        (block
                                                            (local.set $a (i32.const 1))))))))))))))))
    (local.get $a)))
