extern crate wastrumentation_rs_stdlib;

use lazy_static::lazy_static;
use ordered_float::OrderedFloat;
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::Mutex;
use wastrumentation_rs_stdlib::{advice, MutDynArgs, MutDynResults, WasmFunction, WasmValue};

// Global cache structure using a Mutex and AtomicUsize for thread safety
lazy_static! {
    static ref CACHE: Mutex<HashMap<CacheKey, Vec<WasmValueEq>>> = Mutex::new(HashMap::new());
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

// Define a CacheKey structure to uniquely identify cached function calls
#[derive(Eq, PartialEq, Hash)]
struct CacheKey {
    instr_f_idx: i32,
    args: Vec<WasmValueEq>,
}

#[derive(Eq, PartialEq, Hash, Clone)]
enum WasmValueEq {
    I32(i32),
    F32(OrderedFloat<f32>),
    I64(i64),
    F64(OrderedFloat<f64>),
}

impl Into<WasmValueEq> for WasmValue {
    fn into(self) -> WasmValueEq {
        match self {
            WasmValue::I32(v) => WasmValueEq::I32(v),
            WasmValue::F32(v) => WasmValueEq::F32(v.into()),
            WasmValue::I64(v) => WasmValueEq::I64(v),
            WasmValue::F64(v) => WasmValueEq::F64(v.into()),
        }
    }
}

impl Into<WasmValue> for &WasmValueEq {
    fn into(self) -> WasmValue {
        match self {
            WasmValueEq::I32(v) => WasmValue::I32(*v),
            WasmValueEq::F32(OrderedFloat(v)) => WasmValue::F32(*v),
            WasmValueEq::I64(v) => WasmValue::I64(*v),
            WasmValueEq::F64(OrderedFloat(v)) => WasmValue::F64(*v),
        }
    }
}

fn cache_hit(key: &CacheKey) -> bool {
    CACHE.lock().unwrap().contains_key(&key)
}

fn cache_retrieve(key: &CacheKey, results: &mut MutDynResults) {
    if let Some(cached_results) = CACHE.lock().unwrap().get(&key) {
        for (index, wasm_value) in cached_results.iter().enumerate() {
            results.set_res(i32::try_from(index).unwrap(), wasm_value.into());
        }
    } else {
        unreachable!()
    }
}

fn cache_insert(key: CacheKey, results: &MutDynResults) {
    let mut cached_results: Vec<WasmValueEq> =
        Vec::with_capacity(usize::try_from(results.resc).unwrap());

    for index in 0..results.resc {
        cached_results.push(results.get_res(index).into())
    }

    if cached_results.len() != usize::try_from(results.resc).unwrap() {
        unreachable!()
    }

    CACHE.lock().unwrap().insert(key, cached_results);
}

advice! {
    advice apply
    (func: WasmFunction, args: MutDynArgs, results: MutDynResults) {
        let mut wasm_value_vec: Vec<WasmValueEq> = Vec::with_capacity(usize::try_from(args.argc).unwrap());

        for index in 0..args.argc {
            wasm_value_vec.push(args.get_arg(index).into())
        }

        let key = CacheKey {
            instr_f_idx: func.instr_f_idx,
            args: wasm_value_vec,
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
