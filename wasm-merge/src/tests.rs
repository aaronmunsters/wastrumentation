use super::*;

use indoc::indoc;
use wasmtime::*;
use wat::parse_str;

const WAT_ODD: &str = r#"
    (module
        (import "even" "even" (func $even (param i32) (result i32)))
        (export "odd" (func $odd))
        (func $odd (param $0 i32) (result i32)
         local.get $0
         i32.eqz
         if
          i32.const 0
          return
         end
         local.get $0
         i32.const 1
         i32.sub
         call $even))"#;

const WAT_EVEN: &str = r#"
    (module
        (import "odd" "odd" (func $odd (param i32) (result i32)))
        (export "even" (func $even))
        (func $even (param $0 i32) (result i32)
         local.get $0
         i32.eqz
         if
          i32.const 1
          return
         end
         local.get $0
         i32.const 1
         i32.sub
         call $odd))"#;

#[test]
fn test_merge() {
    let wat_even = parse_str(WAT_EVEN).unwrap();
    let wat_odd = parse_str(WAT_ODD).unwrap();
    let merge_options = MergeOptions {
        primary: None,
        input_modules: vec![
            InputModule {
                module: &wat_even,
                namespace: String::from("even"),
            },
            InputModule {
                module: &wat_odd,
                namespace: String::from("odd"),
            },
        ],
        no_validation: options::NoValidate::Enable,
        rename_export_conflicts: options::RenameExportConflicts::Enable,
        ..Default::default()
    };

    // Merge even & odd
    let merged_wasm = merge_options.merge().unwrap();

    // Interpret even & odd
    let mut store = Store::<()>::default();
    let module = Module::from_binary(store.engine(), &merged_wasm).unwrap();
    let instance = Instance::new(&mut store, &module, &[]).unwrap();

    // Fetch `even` and `odd` export
    let even = instance
        .get_typed_func::<i32, i32>(&mut store, "even")
        .unwrap();

    let odd = instance
        .get_typed_func::<i32, i32>(&mut store, "odd")
        .unwrap();

    assert_eq!(even.call(&mut store, 12345).unwrap(), 0);
    assert_eq!(even.call(&mut store, 12346).unwrap(), 1);
    assert_eq!(odd.call(&mut store, 12345).unwrap(), 1);
    assert_eq!(odd.call(&mut store, 12346).unwrap(), 0);
}

#[test]
fn test_merge_fail() {
    let merge_options = MergeOptions {
        primary: Some(InputModule {
            module: &[99, 88, 77, 66],
            namespace: String::from("foo"),
        }),
        input_modules: vec![InputModule {
            module: &[99, 88, 77, 66],
            namespace: String::from("bar"),
        }],
        multimemory: options::Multimemory::Enable,
        bulk_memory: options::BulkMemory::Enable,
        ..Default::default()
    };

    let merge_error = merge_options.merge().unwrap_err();
    assert!(merge_error.to_string().contains("Fatal"));
}

#[test]
fn test_debug() {
    let merge_options = MergeOptions {
        primary: None,
        input_modules: vec![],
        ..Default::default()
    };

    assert_eq!(
        format!("{merge_options:#?}"),
        indoc! { r"
        MergeOptions {
            primary: None,
            input_modules: [],
            no_validation: Disable,
            rename_export_conflicts: Disable,
            bulk_memory: Disable,
            bulk_memory_opt: Disable,
            call_indirect_overlong: Disable,
            extended_const: Disable,
            exception_handling: Disable,
            fp16: Disable,
            gc: Disable,
            memory64: Disable,
            multimemory: Disable,
            multivalue: Disable,
            mutable_globals: Disable,
            nontrapping_float_to_int: Disable,
            reference_types: Disable,
            relaxed_simd: Disable,
            shared_everything: Disable,
            sign_ext: Disable,
            simd: Disable,
            strings: Disable,
            tail_call: Disable,
            threads: Disable,
            typed_continuations: Disable,
        }" }
    );

    let input_module = InputModule {
        module: &[],
        namespace: "namespace".into(),
    };

    assert_eq!(
        format!("{input_module:#?}"),
        indoc! { r#"
        InputModule {
            module: [],
            namespace: "namespace",
        }"# }
    );
}
