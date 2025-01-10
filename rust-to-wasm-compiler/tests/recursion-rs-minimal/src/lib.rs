#![no_std]

extern crate wee_alloc;

#[no_mangle]
pub extern "C" fn factorial(n: i32) -> i32 {
    if n == 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}

#[no_mangle]
pub extern "C" fn fibonacci(n: i32) -> i32 {
    if n < 2 {
        1
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

#[global_allocator]
pub static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Optionally use primitives from core::arch::wasm
// https://doc.rust-lang.org/stable/core/arch/wasm/index.html
#[cfg(target_arch = "wasm32")]
#[panic_handler]
fn panic(_panic: &core::panic::PanicInfo<'_>) -> ! {
    core::arch::wasm32::unreachable()
}

extern crate alloc;
