(module
          (export "main" (func $main))
          (type $void=>void (func))

        ;; let main a = if a == 0 then 1 else 2 ;;
        ;; ==================================== ;;
          (func $main (param $a i32) (result i32)
            (local $result i32)                 ;; i32 $result;
            (local.set $result (i32.const 2))   ;; $result = 2;

            (i32.eqz (local.get $a))                ;; \
            (if (type $void=>void)                  ;;  |
              (then                                 ;;  |
                (local.set $result                  ;;  | if ($a) $result -= 1;
                           (i32.add                 ;;  |
                             (local.get $result)    ;;  |
                             (i32.const -1)))))     ;; /

            (local.get $result)))               ;; return $result;
        ;; ==================================== ;;