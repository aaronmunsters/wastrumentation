(module
  (export "if-label" (func $if-label))
  ;;    switch ($a)
  ;;      case 111: return 100;
  ;;      case 222: return 101;
  ;;       default: return 111;
  (func $if-label (param $a i32) (result i32)
    (local $total i32)
    ;; $total = 100;
    (local.set $total (i32.const 100))
    (block $exit-if-label-function (; $void=>void ;)
        (block $add-one-more-and-exit (; $void=>void ;)
            ;; $total == 111;
            (i32.eq (local.get $a) (i32.const 111))
            (br_if $exit-if-label-function)
            ;; $total == 222;
            (i32.eq (local.get $a) (i32.const 222))
            (br_if $add-one-more-and-exit)
            ;; $total += 10;
            (local.set
              $total  
              (i32.add
                (local.get $total)
                (i32.const 10))))
        ;; $add-one-more-and-exit
        ;; $total += 1;
        (local.set
          $total
          (i32.add
            (local.get $total)
            (i32.const 1))))
    ;; exit-if-label-function
    (local.get $total)
    ))