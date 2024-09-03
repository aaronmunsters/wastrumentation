extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

use std::{f64::consts::PI, i32};

/// Calculates the sine of an angle in radians.
#[wasm_bindgen]
pub extern "C" fn sin(x: f64) -> f64 {
    // Normalize the angle to the range [-π, π]
    let x = x % (2.0 * PI);

    // Use the Taylor series approximation for small angles
    if x.abs() < 0.1 {
        return x - x.powi(3) / 6.0 + x.powi(5) / 120.0;
    }

    // Use the reduction formula to reduce the angle to the range [-π/4, π/4]
    let quadrant = (x / (PI / 2.0)).floor() as i32;
    let x = x - (quadrant as f64) * (PI / 2.0);

    // Use the Taylor series approximation for the reduced angle
    let sin_x = x - x.powi(3) / 6.0 + x.powi(5) / 120.0;

    // Apply the appropriate sign based on the quadrant
    match quadrant {
        0 | 1 => sin_x,
        2 | 3 => -sin_x,
        _ => unreachable!(),
    }
}

/// Calculates the cosine of an angle in radians.
#[wasm_bindgen]
pub extern "C" fn cos(x: f64) -> f64 {
    // Use the identity cos(x) = sin(x + π/2)
    sin(x + PI / 2.0)
}

/// Calculates the tangent of an angle in radians.
#[wasm_bindgen]
pub extern "C" fn tan(x: f64) -> f64 {
    // Use the identity tan(x) = sin(x) / cos(x)
    sin(x) / cos(x)
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
    let mut total: f64 = 0.0;
    for _ in 0..1000 {
        total += tan(total) + f64::from(fibonacci(n));
    }
    total as i32 // potential overflow, but that is ok
}
