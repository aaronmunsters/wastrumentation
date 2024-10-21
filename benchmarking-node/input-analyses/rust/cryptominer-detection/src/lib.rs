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

advice! { binary generic (
        operator: BinaryOperator,
        l: WasmValue,
        r: WasmValue,
        _location: Location,
    ) {
        let target_increment = match operator {
            BinaryOperator::I32Add | BinaryOperator::I64Add | BinaryOperator::F32Add | BinaryOperator::F64Add     => Some(INDEX_ADD__),
            BinaryOperator::I32And | BinaryOperator::I64And => Some(INDEX_AND__),
            BinaryOperator::I32Shl | BinaryOperator::I64Shl => Some(INDEX_SHL__),
            BinaryOperator::I32ShrU | BinaryOperator::I64ShrU => Some(INDEX_SHR_U),
            BinaryOperator::I32Xor | BinaryOperator::I64Xor => Some(INDEX_XOR__),
            _ => None,
        };
        match target_increment {
            Some(index) => {
                unsafe { SIGNATURE[index] += 1 }
            },
            None => {},
        }

        operator.apply(l, r)
    }
}
