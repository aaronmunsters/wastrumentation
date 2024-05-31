(module
  (export "nested-loops" (func $nested-loops))
  (func $nested-loops (result i32)
    (local $a i32)
    (local.set $a (i32.const 0))
    (loop
        (loop
            (loop
                (loop
                    (loop
                        (loop
                            (loop
                                (loop
                                    (loop
                                        (loop
                                            (loop
                                                (loop
                                                    (loop
                                                        (loop
                                                            (local.set $a (i32.const 1))))))))))))))))
    (local.get $a)))
