#![no_std]
use circular_buffer::CircularBuffer;
use core::ptr::{addr_of, addr_of_mut};
use wastrumentation_rs_stdlib::*;

#[derive(Clone)]
struct Access {
    funct_index: u32,
    instr_index: u32,
    address: i64,
    access_kind: AccessKind,
}

#[derive(Clone)]
enum AccessKind {
    Write,
    Read,
}

const CIRCULAR_BUFFER_CAPACITY: usize = 1_000_000;
static mut ACCESSES: CircularBuffer<CIRCULAR_BUFFER_CAPACITY, Access> = CircularBuffer::new();
static mut ACCESSES_COUNT: i64 = 0;

fn add_access(funct_index: u32, instr_index: u32, address: i64, access_kind: AccessKind) {
    let accesses_count = unsafe { addr_of_mut!(ACCESSES_COUNT).as_mut().unwrap() };
    let accesses = unsafe { addr_of_mut!(ACCESSES).as_mut().unwrap() };

    *accesses_count += 1;
    accesses.push_back(Access {
        funct_index,
        instr_index,
        address,
        access_kind,
    });
}

fn get_nth_access(index: i32) -> Access {
    let accesses = unsafe { addr_of!(ACCESSES).as_ref().unwrap() };
    let index: usize = index.try_into().unwrap();
    accesses[index].clone()
}

#[no_mangle]
pub fn total_accesses() -> i64 {
    *unsafe { addr_of!(ACCESSES_COUNT).as_ref().unwrap() }
}

#[no_mangle]
pub fn get_total_accesses_from_buffer() -> i32 {
    let accesses = unsafe { addr_of!(ACCESSES).as_ref().unwrap() };
    accesses.len().try_into().unwrap()
}

#[no_mangle]
pub fn get_nth_funct_index(index: i32) -> i64 {
    get_nth_access(index).funct_index.into()
}

#[no_mangle]
pub fn get_nth_instr_index(index: i32) -> i64 {
    get_nth_access(index).instr_index.into()
}

#[no_mangle]
pub fn get_nth_address(index: i32) -> i64 {
    get_nth_access(index).address.into()
}

#[no_mangle]
pub fn get_nth_operation(index: i32) -> i32 {
    match get_nth_access(index).access_kind {
        AccessKind::Read => 0,
        AccessKind::Write => 1,
    }
}

advice! {
    store (store_index: StoreIndex, value: WasmValue, offset: StoreOffset, operation: StoreOperation, location: Location) {
        // perform unaltered operation
        operation.perform(&store_index, &value, &offset);

        // perform analysis
        let offset = offset.value();
        let address = store_index.value() as i64 + offset;
        let funct_index = location.function_index().try_into().unwrap();
        let instr_index = location.instruction_index().try_into().unwrap();

        let access_kind = AccessKind::Write;
        add_access(funct_index, instr_index, address, access_kind);
    }

    load (load_index: LoadIndex, offset: LoadOffset, operation: LoadOperation, location: Location) {
        // perform unaltered operation
        let outcome = operation.perform(&load_index, &offset);

        // // perform analysis
        let offset = offset.value();
        let address = load_index.value() as i64 + offset;
        let funct_index = location.function_index().try_into().unwrap();
        let instr_index = location.instruction_index().try_into().unwrap();

        let access_kind = AccessKind::Read;
        add_access(funct_index, instr_index, address, access_kind);

        outcome
    }
}
