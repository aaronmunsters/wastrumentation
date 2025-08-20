# TODO:
    - move cheap analyses (as examples) to the main branch
    - move expensive analyses (and benchmarks) to separate repo?
    - move shadow to separate repo

## Paper
- Contribution:
    - Portable intercession
        - Adressed problem: current SOTA does not support portable intercession
        - Evaluation:
            - ✅ implementation of profile-guided memoization analysis
        - Evaluation: ✅ implementation of profile-guided memoization analysis
    - Wasm-compatible analysis language injected in the binary
        - Adressed problem: current SOTA requires JS for analysis language
        - Evaluation:
            - ✅ implementation of Wasabi dynamic analyses within Rust & AssemblyScript
            - 🌥️ additional heavyweight 'faithful shadow execution' dynamic analysis implementation
                - Evaluated to model a correct program execution
                  on the Wasm input test suite that covers a total of 4033 input programs & 20949 assertions.
    - Performance evaluation for portable instrumentation platforms
        - ✅ startup times
        - ✅ long-running execution times
        - ❌ memory footprint (especially for the JS JIT component)
        - ❌ code increase
    - Selective instrumentation
        - ✅ Extend Wasabi's interface with selection for target functions

## TODO: before the deadline!
- Shift to begin/end better hook (i.e. equal to Wasabi ...)
    - Move all analyses to be closer to Wasabi
- Remove the wasm-bindgen from the example memo analysis

## FIXME:
- Do I have block pre and pre block?
- Quite often I get a "recursion limit" error stemming from the `advice!` macro.
    - This is not a skill issue, the macro should better handle wrong usage.

## TODO

- Gas metering
- Implement existing analyses from Wasm-Opt
- wasm-merge >> tests >> find where I use `/****/` to indent and replace with `indoc!`

```
PRIORITY LEGENDA: (🟠) - HIGH, (🔸) - Low, (🔹) - tiny
```

- (🟠) Add hooks to develop Wasabi's taint analysis:
  - ✅ if_         (location ✅, args ✅)
  - ✅ br          (location ✅, args ✅)
  - ✅ br_if       (location ✅, args ✅)
  - ✅ br_table    (location ✅, args ✅)
  - ✅ select      (location ✅, args ✅)
  - ✅ call_pre    (location ✅, args ✅)
  - ✅ call_post   (location ✅, args ✅)
  - ✅ drop        (location ✅, args ✅)
  - ✅ const_      (location ✅, args ✅)
  - ✅ unary       (location ✅, args ✅)
  - ✅ binary      (location ✅, args ✅)
  - ✅ load        (location ✅, args ✅)
  - ✅ store       (location ✅, args ✅)
  - ✅ memory_size (location ✅, args ✅)
  - ✅ memory_grow (location ✅, args ✅)
  - ✅ local       (location ✅, args ✅)
  - ✅ global      (location ✅, args ✅)
  - ✅ return_     (location ✅, args ✅)
  - 🌥️ begin       (location ✅, args ✅)
  - 🌥️ end         (location ✅, args ✅)
  - 🌥️ start       (location ✅, args ✅)
  - 🧐 apply       (location ✅, args ✅)

- (🟠) Implement benchmarks - using those from Wasm-R3?
- (🟠) Implement wasabi analyses?
    - ✅ Instruction mix analysis
    - ✅ Basic block profiling
    - ✅ Instruction coverage
    - ✅ Branch coverage
    - ✅ Call graph analysis
    - ✅ Memory access tracing
    - ✅ Cryptominer detection
    - ❌ Dynamic taint analysis

- (🔸) Test the alteration of call_indirect
- (🔸) Add unique address identification per instruction

- (🔹) Cache rust->wasm builds if source and/or deps does not change?
    - This is easy to provide if I use one single directory accross compilation invocations
- (🔹) What if you define WASP trap multiple times?
- (🔹) Rename stdlib to 'core'?

## TODO - Repo cleanup

- (🔹) Add 'author' / 'based on' wherever applicable! (e.g. in the input analyses, reference Daniel Lehmann)

- (🔹) Move the `test-configurations.json` to a macro (makes use of rustfmt possible)
- (🔹) Assert that the `forward` analysis passes for all input programs; remove from `json` file
- (🔹) Move common dependencies to workspace
- (🔹) Clean up self-defined errors;
    - (🔹) Not use strings but enum values to distinct in cases (https://doc.rust-lang.org/rust-by-example/error/multiple_error_types/define_error_type.html)
    - (🔹) In general aim to move all to the `anyhow` crate–

## API Documentation

```
--- Legenda ---
Performance    - [🐌 slow] - [🦔 medium] - [🐇 fast]
Expressiveness - [🔍 introspection] - [📝 intercession]
Generality     - [🙆‍ general] - [🪖 specific]
---------------
```

## TODO - Control Instructions

[Control Instructions](https://webassembly.github.io/spec/core/exec/instructions.html#control-instructions):

```
- [ ] unreachable
- [ ] nop
- [ ] {!} block ... end {!}
    - Is this ever interesting? Had this before, needed to take alternative
- [ ] {!} loop ... end {!}
    - Is this ever interesting? Had this before, needed to take alternative

Done:
- [x] br ...
- [x] return
- [x] if ... else ... end
- [x] br_if ...
- [x] function application
- [x] call_indirect ...
- [x] call ...
- [x] br_table ...
- [x] block {!} ... {!} end
- [x] loop {!} ... {!} end
- [x] select ...
```

## After minimal platform

- Can I write a branch coverage program? --> See below!
- Train a model where I can discect the language runtime from the user source code
- Once this has a high accuracy, perform concolic testing on the user source

- Go over `#[allow(` statements for clippy

```
Think hard about this:
--> all instrumentation code uses function calls to perform the instrumentation code.
When you transform function calls, you want earlier instrumentation not to have happened yet.
```

Add support for selective instrumentation from the analysis:

```rust
advice! {
    advice apply                        // analysis hook
    @target( index = [1, 2, 3],         // target index
             internal = ["$foo"],       // target WAT name
             export = ["md_to_pdf"],    // target export name
             import = ["read_file"] )   // target import name
    (func: WasmFunction)                // runtime function index
    (arg_first: i32, arg_second: i32)   // runtime function arguments
    (res_first: i32, res_second: i32)   // runtime function results
    {
        /* analysis code */
        // ... pre ...
        func.apply();
        // ... post ...
        /* analysis code */
    }
}
```

---
- `br-if-labeled-if.wat`

```lisp
(module
  (func $if-label (param $a i32)
    (i32.const 0)
    (if $top-level-if (result i32)
      (then
        (i32.const 5)
        (i32.const 0)
        (br_if $top-level-if)
        (drop)
        (i32.const 1))
      (else (i32.const 2)))
    (i32.const 10)
    (i32.add)
    (drop)))
```

## Pointcut Specification

--hooks CallPre
        CallPost
        CallIndirectPre
        CallIndirectPost
        CallPre export=["socket_send", "read_line"]
                    import=["write_to_file"]
                    internal=[456]

### Future Work (high-level events)
c-source=["main"] `# future work`

### Input programs

recursion.toml.rs
```rs
package.name = "rust-recursion"
package.version = "0.1.0"
package.edition = "2021"
lib.crate-type = ["cdylib"]
profile.release.strip = true
profile.release.lto = true
profile.release.panic = "abort"
dependencies.wasm-bindgen = "0.2"
[workspace]
---
extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
pub extern "C" fn factorial(n: i32) -> i32 { if n == 1 { 1 } else { n * factorial(n - 1) } }
#[wasm_bindgen]
pub extern "C" fn fibonacci(n: i32) -> i32 { if n < 2 { 1 } else { fibonacci(n - 1) + fibonacci(n - 2) } }
#[wasm_bindgen]
pub extern "C" fn compute_recursive(n: i32) -> i32 { factorial(n) + fibonacci(n) }
```

boa-recursion-original.toml.rs
```rs
package.name = "rust-boa-recursion"
package.version = "0.1.0"
package.edition = "2021"
lib.crate-type = ["cdylib"]
profile.release.strip = true
profile.release.lto = true
profile.release.panic = "abort"
dependencies.boa_engine = { version = "0.19", features = [] }
dependencies.getrandom = { version = "0.2", features = ["js"] }
dependencies.wasm-bindgen = "0.2"
[workspace]
---
extern crate wasm_bindgen;

use boa_engine::{self, Context, Source};
use std::fmt;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub extern "C" fn factorial(n: i32) -> i32 { if n == 1 { 1 } else { n * factorial(n - 1) } }
#[wasm_bindgen]
pub extern "C" fn fibonacci(n: i32) -> i32 { if n < 2 { 1 } else { fibonacci(n - 1) + fibonacci(n - 2) } }
#[wasm_bindgen]
pub extern "C" fn compute_recursive(n: i32) -> i32 {
    let js_code = &fmt::format(format_args!(
        r#"
    function factorial(n) {{ if (n === 1) {{ return 1; }} else {{ return n * factorial(n - 1); }} }}
    function fibonacci(n) {{ if (n < 2) {{ return 1; }} else {{ return fibonacci(n - 1) + fibonacci(n - 2); }} }}

    let total = 0;
    for (let i = 0; i <= 100; i++) {{ total += factorial({n}) + fibonacci({n}); }}
    total + factorial({n}) + fibonacci({n})
    "#
    ));

    let mut context = Context::default();
    let result = context.eval(Source::from_bytes(js_code)).unwrap();

    let mut number = result.as_number().unwrap() as i32;
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
```
