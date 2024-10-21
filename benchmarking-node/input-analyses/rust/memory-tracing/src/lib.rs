extern crate wastrumentation_rs_stdlib;

use std::ptr::{addr_of, addr_of_mut};
use wastrumentation_rs_stdlib::*;

#[derive(Clone)]
struct Access {
    pub funct_index: i64,
    pub instr_index: i64,
    pub address: i32,
    pub access_kind: AccessKind,
}

#[derive(Clone)]
enum AccessKind {
    Write,
    Read,
}

static mut ACCESSES: Vec<Access> = Vec::new();

fn add_access(funct_index: i64, instr_index: i64, address: i32, access_kind: AccessKind) {
    let accesses = unsafe { addr_of_mut!(ACCESSES).as_mut().unwrap() };
    accesses.push(Access {
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
pub fn get_total_accesses() -> i32 {
    let accesses = unsafe { addr_of!(ACCESSES).as_ref().unwrap() };
    accesses.len().try_into().unwrap()
}

#[no_mangle]
pub fn get_nth_funct_index(index: i32) -> i64 {
    get_nth_access(index).funct_index
}

#[no_mangle]
pub fn get_nth_instr_index(index: i32) -> i64 {
    get_nth_access(index).instr_index
}

#[no_mangle]
pub fn get_nth_address(index: i32) -> i32 {
    get_nth_access(index).address
}

#[no_mangle]
pub fn get_nth_operation(index: i32) -> i32 {
    match get_nth_access(index).access_kind {
        AccessKind::Read => 0,
        AccessKind::Write => 1,
    }
}

advice! { store generic (store_index: StoreIndex, value: WasmValue, offset: StoreOffset, operation: StoreOperation, location: Location) {
        // perform unaltered operation
        operation.perform(&store_index, &value, &offset);

        // perform analysis
        let offset: i32 = offset.value().try_into().unwrap();
        let address = store_index.value() + offset;
        let funct_index = location.function_index();
        let instr_index = location.instruction_index();
        let access_kind = AccessKind::Write;
        add_access(funct_index, instr_index, address, access_kind);
    }
}

advice! { load generic (load_index: LoadIndex, offset: LoadOffset, operation: LoadOperation, location: Location) {
        // perform unaltered operation
        let outcome = operation.perform(&load_index, &offset);

        // perform analysis
        let offset: i32 = offset.value().try_into().unwrap();
        let address = load_index.value() + offset;
        let funct_index = location.function_index();
        let instr_index = location.instruction_index();
        let access_kind = AccessKind::Read;
        add_access(funct_index, instr_index, address, access_kind);

        outcome
    }
}
