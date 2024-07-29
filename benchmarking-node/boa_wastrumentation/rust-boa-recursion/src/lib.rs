extern crate wasm_bindgen;

use boa_engine::{self, Context, Source};
use std::fmt;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub extern "C" fn factorial(n: i32) -> i32 {
    if n == 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}

#[wasm_bindgen]
pub extern "C" fn fibonacci(n: i32) -> i32 {
    if n < 2 {
        1
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

#[wasm_bindgen]
pub extern "C" fn compute_recursive(n: i32) -> i32 {
    factorial(n) + fibonacci(n)
}

#[wasm_bindgen]
pub extern "C" fn compute_recursive_through_js(n: i32) -> i32 {
    let js_code = &fmt::format(format_args!(
        r#"
    function factorial(n) {{
        if (n === 1) {{
            return 1;
        }} else {{
            return n * factorial(n - 1);
        }}
    }}

    function fibonacci(n) {{
        if (n < 2) {{
            return 1;
        }} else {{
            return fibonacci(n - 1) + fibonacci(n - 2);
        }}
    }}

    let total = 0;
    for (let i = 0; i <= 100; i++) {{
        total += factorial({n}) + fibonacci({n});
    }}
    total + factorial({n}) + fibonacci({n})
    "#
    ));

    let mut context = Context::default();
    let result = context.eval(Source::from_bytes(js_code)).unwrap();

    let number = result.as_number().unwrap() as i32;
    number
}

#[cfg(test)]
mod tests {
    use crate::{compute_recursive, compute_recursive_through_js};

    #[test]
    fn foo() {
        for i in 1..13 {
            assert_eq!(compute_recursive(i), compute_recursive_through_js(i));
        }
    }
}
