(module
  (export "if-label" (func $if-label))
  ;;    switch ($a)
  ;;      case 111: return 100;
  ;;      case 222: return 101;
  ;;       default: return 111;
  (func $if-label (param $a i32) (result i32)
    (local $total i32)
    (i32.const 100)
    (local.set $total)
    (block $exit-if-label-function (; $void=>void ;)
        (block $add-one-more-and-exit (; $void=>void ;)
            (local.get $a)
            (i32.const 111)
            (i32.eq)
            (br_if $exit-if-label-function)
            (local.get $a)
            (i32.const 222)
            (i32.eq)
            (br_if $add-one-more-and-exit)

            (local.get $total)
            (i32.const 10)
            (i32.add)
            (local.set $total))
        ;; $add-one-more-and-exit
        (local.get $total)
        (i32.const 1)
        (i32.add)
        (local.set $total))
    ;; exit-if-label-function
    (local.get $total)
    ))