(module
    (export "main" (func $main))

    (func $main (result i32) ;; 10000
        ;; const $2200001 = 2200001, $2200010 = 2200010, $2200100 = 2200100, $2201000 = 2201000;
        ;; const $true = 1, $flse = 0;
        (local $2200001 i32) (local $2200010 i32) (local $2200100 i32) (local $2201000 i32)
        (local $true i32) (local $flse i32)
        (local.set $2200001 (i32.const 2200001))
        (local.set $2200010 (i32.const 2200010))
        (local.set $2200100 (i32.const 2200100))
        (local.set $2201000 (i32.const 2201000))
        (local.set $true (i32.const 0))
        (local.set $flse (i32.const 1))

        ;; let t0 = $true ? $2200001 : $2200010
        (call $a1==0?a2:a3 (local.get $true) (local.get $2200001) (local.get $2200010))
        ;; let t1 = $false ? $2200100 : $2201000
        (call $a1==0?a2:a3 (local.get $flse) (local.get $2200100) (local.get $2201000))
        ;; return t0 + t1
        (i32.add)) ;; uninstrumented: 4401001

    (func $a1==0?a2:a3 (param $a1 i32) (param $a2 i32) (param $a3 i32) (result i32)
        ;; TOS := a1 === 0
        (i32.eq (local.get $a1) (i32.const 0))
        ;; (if (TOS) a2 a3)
        (if (result i32)
            (then (local.get $a2))
            (else (local.get $a3)))))
