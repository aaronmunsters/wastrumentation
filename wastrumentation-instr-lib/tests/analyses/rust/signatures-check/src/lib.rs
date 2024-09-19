extern crate wastrumentation_rs_stdlib;
use ordered_float::OrderedFloat;
use wastrumentation_rs_stdlib::{
    advice, MutDynArgs, MutDynResults, WasmFunction, WasmType, WasmValue,
};

extern crate alloc;
use alloc::vec::Vec;

static mut ALL_EXPECTATIONS_MET: bool = false;

#[no_mangle]
pub extern "C" fn are_all_expectations_met() -> i32 {
    if unsafe { ALL_EXPECTATIONS_MET } {
        1
    } else {
        0
    }
}

struct CallExpectation {
    arguments: Vec<WasmValue>,
    results: Vec<WasmValue>,
}

#[derive(Eq, PartialEq, Clone)]
enum WasmValueEq {
    I32(i32),
    F32(OrderedFloat<f32>),
    I64(i64),
    F64(OrderedFloat<f64>),
}

impl From<&WasmValue> for WasmValueEq {
    fn from(value: &WasmValue) -> Self {
        match value {
            WasmValue::I32(v) => WasmValueEq::I32(*v),
            WasmValue::F32(v) => WasmValueEq::F32((*v).into()),
            WasmValue::I64(v) => WasmValueEq::I64(*v),
            WasmValue::F64(v) => WasmValueEq::F64((*v).into()),
        }
    }
}

impl From<&WasmValueEq> for WasmValue {
    fn from(value: &WasmValueEq) -> Self {
        match value {
            WasmValueEq::I32(v) => WasmValue::I32(*v),
            WasmValueEq::F32(OrderedFloat(v)) => WasmValue::F32(*v),
            WasmValueEq::I64(v) => WasmValue::I64(*v),
            WasmValueEq::F64(OrderedFloat(v)) => WasmValue::F64(*v),
        }
    }
}

impl CallExpectation {
    fn new(arguments: Vec<WasmValue>, results: Vec<WasmValue>) -> Self {
        Self { arguments, results }
    }

    fn validate_call(&self, args: MutDynArgs, ress: MutDynResults) {
        for (index, expected_arg) in self.arguments.iter().enumerate() {
            let actual_arg = args.get_arg_type(index.try_into().unwrap());
            match expected_arg {
                WasmValue::I32(_) => assert!(matches!(actual_arg, WasmType::I32)),
                WasmValue::F32(_) => assert!(matches!(actual_arg, WasmType::F32)),
                WasmValue::I64(_) => assert!(matches!(actual_arg, WasmType::I64)),
                WasmValue::F64(_) => assert!(matches!(actual_arg, WasmType::F64)),
            }
            if *expected_arg != (args.get_arg(index.try_into().unwrap())) {
                panic!()
            }
        }

        for (index, expected_res) in self.results.iter().enumerate() {
            let actual_res = ress.get_res_type(index.try_into().unwrap());
            match expected_res {
                WasmValue::I32(_) => assert!(matches!(actual_res, WasmType::I32)),
                WasmValue::F32(_) => assert!(matches!(actual_res, WasmType::F32)),
                WasmValue::I64(_) => assert!(matches!(actual_res, WasmType::I64)),
                WasmValue::F64(_) => assert!(matches!(actual_res, WasmType::F64)),
            }
            if *expected_res != (ress.get_res(index.try_into().unwrap())) {
                panic!()
            }
        }
    }
}

// Expected order of computations, this is post-order evaluation
// Global EXPECTATIONS
use lazy_static::lazy_static;
use std::sync::Mutex;
// Expected order of computations, this is post-order evaluation
// Global EXPECTATIONS
lazy_static! {
  static ref EXPECTATIONS_INDEX: Mutex<usize> = Mutex::new(0);
  static ref EXPECTATIONS: Mutex<Vec<CallExpectation>> = Mutex::new(vec![
    // (call $void=>void)
    CallExpectation::new(vec![], vec![]),
    // (call $i32=>void (i32.const 123456))
    CallExpectation::new(
      vec![WasmValue::I32(123456)],
      vec![],
    ),
    // (call $void=>i32)
    CallExpectation::new(
      vec![],
      vec![WasmValue::I32(1)],
    ),
    // (call $assert (i32.eq (i32.const 1)))
    CallExpectation::new(vec![WasmValue::I32(1)], vec![]),
    // (call $void=>i32_i32)
    CallExpectation::new(
      vec![],
      vec![WasmValue::I32(2), WasmValue::I32(2)],
    ),
    // (call $assert (i32.eq (i32.const 2)))
    CallExpectation::new(vec![WasmValue::I32(1)], vec![]),
    // (call $assert (i32.eq (i32.const 2)))
    CallExpectation::new(vec![WasmValue::I32(1)], vec![]),
    // (call $void=>i32_i32_i32)
    CallExpectation::new(
      vec![],
      vec![
        WasmValue::I32(3),
        WasmValue::I32(3),
        WasmValue::I32(3)
      ],
    ),
    // (call $assert (i32.eq (i32.const 3)))
    CallExpectation::new(vec![WasmValue::I32(1)], vec![]),
    // (call $assert (i32.eq (i32.const 3)))
    CallExpectation::new(vec![WasmValue::I32(1)], vec![]),
    // (call $assert (i32.eq (i32.const 3)))
    CallExpectation::new(vec![WasmValue::I32(1)], vec![]),
    // (call $i32=>i32_i32 (i32.const 4))
    CallExpectation::new(
      vec![
        WasmValue::I32(4)
      ],
      vec![
        WasmValue::I32(4),
        WasmValue::I32(4),
      ],
    ),
    // (call $assert (i32.eq (i32.const 4)))
    CallExpectation::new(vec![WasmValue::I32(1)], vec![]),
    // (call $assert (i32.eq (i32.const 4)))
    CallExpectation::new(vec![WasmValue::I32(1)], vec![]),
    // (call $i32_i32=>i32 (i32.const 5) (i32.const 55))
    CallExpectation::new(
      vec![
        WasmValue::I32(5),
        WasmValue::I32(55),
      ],
      vec![
        WasmValue::I32(5),
      ],
    ),
    // (call $assert (i32.eq (i32.const 5)))
    CallExpectation::new(vec![WasmValue::I32(1)], vec![]),
    // (call $i32_i32_i32=>void (i32.const 6) (i32.const 66) (i32.const 666))
    CallExpectation::new(
      vec![
        WasmValue::I32(6),
        WasmValue::I32(66),
        WasmValue::I32(666),
      ],
      vec![],
    ),
    // (call $i32_f32_i64_f64=>f64_i64_f32_i32 (i32.const 7) (f32.const 77) (i64.const 777) (f64.const 7777))
    CallExpectation::new(
        vec![
          WasmValue::I32(7),
          WasmValue::F32(77.0),
          WasmValue::I64(777),
          WasmValue::F64(7777.0),
        ],
        vec![
          WasmValue::F64(7777.0),
          WasmValue::I64(777),
          WasmValue::F32(77.0),
          WasmValue::I32(7),
        ],
      ),
    // (call $assert (i32.eq (i32.const 7)))
    CallExpectation::new(vec![WasmValue::I32(1)], vec![]),
    // (call $assert (f32.eq (f32.const 77)))
    CallExpectation::new(vec![WasmValue::I32(1)], vec![]),
    // (call $assert (i64.eq (i64.const 777)))
    CallExpectation::new(vec![WasmValue::I32(1)], vec![]),
    // (call $assert (f64.eq (f64.const 7777)))
    CallExpectation::new(vec![WasmValue::I32(1)], vec![]),
    // (call $execute_tests)
    CallExpectation::new(vec![], vec![]),
]);
}

advice! {
    advice apply
    (func: WasmFunction, args: MutDynArgs, results: MutDynResults) {
        func.apply();

        let mut expectation_index = EXPECTATIONS_INDEX.lock().unwrap();
        let expectations = EXPECTATIONS.lock().unwrap();
        // 1. Assert there are as many expectations as actual calls
        let expectation = expectations.get(*expectation_index).expect("There are as many expectations as actual calls");
        // 2. Assert the validation as described by the call expectation
        expectation.validate_call(args, results);
        *expectation_index += 1;
        // 3. Assert that there are not more calls
        if *expectation_index == expectations.len() {
          unsafe { ALL_EXPECTATIONS_MET = true }
        }
    }
}
