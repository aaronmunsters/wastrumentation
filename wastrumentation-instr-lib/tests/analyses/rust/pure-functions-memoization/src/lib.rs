extern crate wastrumentation_rs_stdlib;

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::Mutex;
use wastrumentation_rs_stdlib::{advice, MutDynArgs, MutDynResults, WasmFunction, WasmValue};

// Global cache structure using a Mutex and AtomicUsize for thread safety
lazy_static! {
    static ref CACHE: Mutex<HashMap<CacheKey, Vec<WasmValue>>> = Mutex::new(HashMap::new());
    static ref CACHE_SIZE: AtomicUsize = AtomicUsize::new(0);
}

#[no_mangle]
pub extern "C" fn CACHE_SIZE_REPORT() -> i32 {
    CACHE.lock().unwrap().len() as i32
}

static mut CACHE_HITS: i32 = 0;

#[no_mangle]
pub extern "C" fn CACHE_HIT_REPORT() -> i32 {
    unsafe { CACHE_HITS }
}

type BitPattern = [u8; 8];
fn to_bit_pattern(value: WasmValue) -> BitPattern {
    let mut pattern = [0; 8];
    match value {
        WasmValue::I32(v) => pattern[..4].copy_from_slice(&v.to_le_bytes()),
        WasmValue::F32(v) => pattern[..4].copy_from_slice(&v.to_le_bytes()),
        WasmValue::I64(v) => pattern[..].copy_from_slice(&v.to_le_bytes()),
        WasmValue::F64(v) => pattern[..].copy_from_slice(&v.to_le_bytes()),
    }
    pattern
}

// Define a CacheKey structure to uniquely identify cached function calls
#[derive(Eq, PartialEq, Hash)]
struct CacheKey {
    instr_f_idx: i32,
    args: Vec<BitPattern>,
}

fn cache_hit(key: &CacheKey) -> bool {
    CACHE.lock().unwrap().contains_key(&key)
}

fn cache_retrieve(key: &CacheKey, results: &mut MutDynResults) {
    let cache = CACHE.lock().unwrap();
    let cached_results = cache.get(&key).unwrap();
    cached_results
        .into_iter()
        .enumerate()
        .for_each(|(index, wasm_value)| {
            results.set_res(i32::try_from(index).unwrap(), wasm_value.to_owned())
        });
}

fn cache_insert(key: CacheKey, results: &MutDynResults) {
    let cached_results: Vec<WasmValue> = results.ress_iter().collect();
    assert_eq!(cached_results.len(), usize::try_from(results.resc).unwrap());
    CACHE.lock().unwrap().insert(key, cached_results);
}

advice! { apply (func: WasmFunction, args: MutDynArgs, results: MutDynResults) {
        let key = CacheKey {
            instr_f_idx: func.instr_f_idx,
            args: args.args_iter().map(to_bit_pattern).collect(),
        };

        if cache_hit(&key) {
            unsafe { CACHE_HITS += 1 };
            cache_retrieve(&key, &mut results);
        } else {
            func.apply();
            cache_insert(key, &results);
        }
    }
}
