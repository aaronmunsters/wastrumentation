extern crate wastrumentation_rs_stdlib;

use wastrumentation_rs_stdlib::*;

static mut SIGNATURE: &mut [i64; 5] = &mut [0; 5];
const INDEX_ADD__: usize = 0;
const INDEX_AND__: usize = 1;
const INDEX_SHL__: usize = 2;
const INDEX_SHR_U: usize = 3;
const INDEX_XOR__: usize = 4;

#[no_mangle]
pub fn get_signature(index: i32) -> i64 {
    let index: usize = index.try_into().unwrap();
    unsafe { SIGNATURE[index] }
}

advice! { binary (operator: BinaryOperator, l: WasmValue, r: WasmValue, _location: Location) {
        use wastrumentation_rs_stdlib::BinaryOperator::*;
        let target_increment = match operator {
            I32Add  | I64Add | F32Add | F64Add => Some(INDEX_ADD__),
            I32And  | I64And                   => Some(INDEX_AND__),
            I32Shl  | I64Shl                   => Some(INDEX_SHL__),
            I32ShrU | I64ShrU                  => Some(INDEX_SHR_U),
            I32Xor  | I64Xor                   => Some(INDEX_XOR__),
            _ => None,
        };

        target_increment.map(|idx| unsafe { SIGNATURE[idx] += 1 });
        operator.apply(l, r)
    }
}
