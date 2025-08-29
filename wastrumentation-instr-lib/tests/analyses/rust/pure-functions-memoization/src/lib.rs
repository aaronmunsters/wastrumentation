#![no_std]

use heapless::{index_map::FnvIndexMap, Vec};
use wastrumentation_rs_stdlib::{advice, MutDynArgs, MutDynResults, WasmFunction, WasmValue};

// Global cache structure using a Mutex and AtomicUsize for thread safety
const CACHE_CAPACITY: usize = 1024;
const CACHED_VALUES_MAX_CAPACITY: usize = 10;
const CACHED_RESULTS_MAX_CAPACITY: usize = 10;

static mut CACHE: FnvIndexMap<
    CacheKey,
    Vec<WasmValue, CACHED_RESULTS_MAX_CAPACITY>,
    CACHE_CAPACITY,
> = FnvIndexMap::new();
static mut CACHE_HITS: i32 = 0;

#[no_mangle]
pub extern "C" fn CACHE_SIZE_REPORT() -> i32 {
    unsafe { (*(&raw mut CACHE)).len() as i32 }
}

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
    args: Vec<BitPattern, CACHED_VALUES_MAX_CAPACITY>,
}

fn cache_hit(key: &CacheKey) -> bool {
    unsafe { (*(&raw mut CACHE)).contains_key(&key) }
}

fn cache_retrieve(key: &CacheKey, results: &mut MutDynResults) {
    let cached_results = unsafe { (*(&raw mut CACHE)).get(&key).unwrap() };
    cached_results
        .into_iter()
        .enumerate()
        .for_each(|(index, wasm_value)| {
            results.set_res(i32::try_from(index).unwrap(), wasm_value.clone())
        });
}

fn cache_insert(key: CacheKey, results: &MutDynResults) {
    let cached_results: Vec<WasmValue, CACHED_RESULTS_MAX_CAPACITY> = results.ress_iter().collect();
    assert_eq!(cached_results.len(), usize::try_from(results.resc).unwrap());
    unsafe {
        (*(&raw mut CACHE))
            .insert(key, cached_results)
            .map_err(|_| ())
            .unwrap();
    }
}

advice! { apply (func: WasmFunction, args: MutDynArgs, results: MutDynResults) {
        // If args / results would not fit in cache, continue as normal
        let too_many_args = args.argc > CACHED_VALUES_MAX_CAPACITY as i32;
        let too_many_ress = results.resc > CACHED_RESULTS_MAX_CAPACITY as i32;
        let cache_full = unsafe { (*(&raw mut CACHE)).len() } == CACHE_CAPACITY;

        if  too_many_args || too_many_ress || cache_full {
            func.apply();
            return;
        };

        // Construct cache key
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
