// Source of original pass:
// binaryen/src/passes/SafeHeap.cpp
// --> https://github.com/WebAssembly/binaryen/blob/39bf87eb39543ca14198a16533f262a147816793/src/passes/SafeHeap.cpp

// Instruments code to check for incorrect heap access. This checks
// for dereferencing 0 (null pointer access), reading past the valid
// top of sbrk()-addressible memory, and incorrect alignment notation.

#![no_std]
#![feature(strict_overflow_ops)]
use wastrumentation_rs_stdlib::*;

const SIZE_AS_BYTES: i64 = i64::pow(2, 16);

fn bounds_check(index: i64, bytes: i64, offset: i64) {
    // strict_add asserts that no overflow occurred
    let last_target_byte = index.strict_add(bytes).strict_add(offset);
    assert!(last_target_byte != 0);
    assert!(last_target_byte <= (base_memory_size(0) as i64) * SIZE_AS_BYTES);
}

fn alignment_check(index: i64, size: i64) {
    // turn size to mask (-1) and assert that that mask has zeroes (&)
    assert!(index & (size - 1) == 0); // size == 4 (_32) | 8 (_64)
}

fn safe_load(i: LoadIndex, o: LoadOffset, op: LoadOperation) -> WasmValue {
    // Bound check: check for reading past valid memory: if pointer + offset + bytes
    bounds_check(i.value() as i64, op.target_value_size() as i64, o.value());
    // Alignment check
    alignment_check(i.value() as i64, op.target_value_size() as i64);
    // Perform
    op.perform(&i, &o)
}

fn safe_store(i: StoreIndex, v: WasmValue, o: StoreOffset, op: StoreOperation) -> () {
    // Bound check: check for reading past valid memory: if pointer + offset + bytes
    bounds_check(i.value() as i64, op.target_value_size() as i64, o.value());
    // Alignment check
    alignment_check(i.value() as i64, op.target_value_size() as i64);
    // Perform
    op.perform(&i, &v, &o);
}

// Target program events: loads & stores
advice! { load generic (i: LoadIndex, o: LoadOffset, op: LoadOperation, _l: Location) {
/* */ safe_load(i, o, op)  } }
advice! { store generic (i: StoreIndex, v: WasmValue, o: StoreOffset, op: StoreOperation, _l: Location) {
/* */ safe_store(i, v, o, op); } }

// In `SafeHeap.cpp` the inspiration source for `fn bounds_check` is `makeBoundsCheck`
// https://github.com/WebAssembly/binaryen/blob/39bf87eb39543ca14198a16533f262a147816793/src/passes/SafeHeap.cpp#L396

// In `SafeHeap.cpp` the inspiration source for `fn alignment_check` is `makeAlignCheck`
// https://github.com/WebAssembly/binaryen/blob/39bf87eb39543ca14198a16533f262a147816793/src/passes/SafeHeap.cpp#L375

// In `SafeHeap.cpp` the inspiration source for `fn safe_load` is `addLoadFunc`
// https://github.com/WebAssembly/binaryen/blob/39bf87eb39543ca14198a16533f262a147816793/src/passes/SafeHeap.cpp#L279

// In `SafeHeap.cpp` the inspiration source for `fn safe_store` is `addStoreFunc`
// https://github.com/WebAssembly/binaryen/blob/39bf87eb39543ca14198a16533f262a147816793/src/passes/SafeHeap.cpp#L329

// E.g. for alignment_check:
//      4 => 0000...0100 ==[-1]=> 0011 ==[idx & _]=> ...
//           idx is aligned: 0000
//           idx is misalgd: 00XX where XX != 0
//      8 => 0000...1000 ==[-1]=> 0111 ==[idx & _]=>
//           idx is aligned: 0000
//           idx is misalgd: 0XXX where XXX != 0
