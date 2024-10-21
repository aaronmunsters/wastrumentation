static mut GLOBAL_COUNT_TWO: i32 = 0;

#[no_mangle]
pub extern "C" fn g(input: i32) -> i32 {
    unsafe { GLOBAL_COUNT_TWO += 2 };
    let a = input + 1;
    let b = a * 10;
    let c = b - 25;
    #[allow(arithmetic_overflow)]
    return c >> 1;
}

#[no_mangle]
pub extern "C" fn f(input: i32) -> i32 {
    if input == 0 {
        return 12345;
    };
    let x = 2 * input;
    let y = 5 + x;
    let z = g(y) * g(y);
    return z;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f() {
        assert_eq!(1555009, f(123));
    }
}
