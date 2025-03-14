use std::ptr::{addr_of, addr_of_mut};
use wastrumentation_rs_stdlib::*;
use LoadOperation::*;
use StoreOperation::*;

// READS & WRITES keep count per byte in linear memory
static mut READS: Vec<i64> = vec![];
static mut WRITES: Vec<i64> = vec![];

// grow if out of bounds and set the bits
fn increase_at(buffer: &mut Vec<i64>, address: usize, target_size: usize) {
    if buffer.len() <= address + target_size {
        buffer.resize_with(address + target_size, || 0);
    }
    for i in 0..target_size {
        let byte = (address + i) as usize;
        buffer[byte] += 1;
    }
}

#[no_mangle]
pub fn get_read_at(mem_idx: i32) -> i32 {
    let reads = unsafe { addr_of!(READS).as_ref().unwrap() };
    reads[mem_idx as usize] as i32
}

#[no_mangle]
pub fn get_write_at(mem_idx: i32) -> i32 {
    let writes = unsafe { addr_of!(WRITES).as_ref().unwrap() };
    writes[mem_idx as usize] as i32
}

advice! {
    load (i: LoadIndex, o: LoadOffset, op: LoadOperation, _l: Location) {
        let reads = unsafe { addr_of_mut!(READS).as_mut().unwrap() };
        let size = match op {
            I32Load8S | I32Load8U | I64Load8S | I64Load8U => 1,
            I32Load16S | I32Load16U | I64Load16S | I64Load16U => 2,
            I32Load | F32Load | I64Load32S | I64Load32U => 4,
            I64Load | F64Load => 8,
        };
        increase_at(reads, (i.value() as i64 + o.value()) as _, size);
        op.perform(&i, &o)
    }

    store (i: StoreIndex, v: WasmValue, o: StoreOffset, op: StoreOperation, _l: Location) {
        let writes = unsafe { addr_of_mut!(WRITES).as_mut().unwrap() };
        let touched_bytes = match op {
            I32Store8 | I64Store8 => 1,
            I32Store16 | I64Store16 => 2,
            F32Store | I32Store | I64Store32 => 4,
            I64Store | F64Store => 8,
        };
        increase_at(writes, (i.value() as i64 + o.value()) as _, touched_bytes);
        op.perform(&i, &v, &o);
    }
}
