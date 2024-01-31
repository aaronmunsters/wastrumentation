(module
    (export "main" (func $main))
    (func $main
        (i32.const 1000)
        (if (result i32)
            (then
                (i32.const 1001)
                (if (result i32)
                    (then
                        (i32.const 1002)
                        (if (result i32)
                            (then (i32.const 0))
                            (else (i32.const 1))))
                    (else
                        (i32.const 1003)
                        (if (result i32)
                            (then (i32.const 2))
                            (else (i32.const 3)))))
                (drop)
                (i32.const 1004)
                (if (result i32)
                    (then (i32.const 4))
                    (else
                        (i32.const 1005)
                        (if (result i32)
                            (then (i32.const 5))
                            (else (i32.const 6))))))
            (else
                (i32.const 1006)
                (if (result i32)
                    (then
                        (i32.const 1007)
                        (if (result i32)
                            (then
                                (i32.const 1008)
                                (if (result i32)
                                    (then (i32.const 7))
                                    (else (i32.const 8))))
                            (else (i32.const 9))))
                    (else
                        (i32.const 1009)
                        (if (result i32)
                            (then
                                (i32.const 1010)
                                (if (result i32)
                                    (then
                                        (i32.const 1011)
                                        (if (result i32)
                                            (then (i32.const 10))
                                            (else (i32.const 11))))
                                    (else (i32.const 12))))
                            (else (i32.const 13))
                        )))
                (drop)
                (i32.const 1012)
                (if (result i32)
                    (then (i32.const 14))
                    (else (i32.const 15)))))
        (drop)))