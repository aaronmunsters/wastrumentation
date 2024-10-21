use wastrumentation_rs_stdlib::WasmType;

// Imports
use super::WasmValue;
use core::ptr::addr_of_mut;

//////////////////////////////////
// compile-time severity checks //
//////////////////////////////////

static mut TRGT_GLOBALS_NOT_INITIALISED: bool = false;
static mut TRGT_GLOBALS_ONLY_AFFECTED_INTERNALLY: bool = false;

////////////////
// Public API //
////////////////

pub struct GlobalAddress(usize);

impl GlobalAddress {
    #[must_use]
    pub fn value(&self) -> usize {
        let Self(value) = self;
        *value
    }
}

#[derive(Default)]
pub struct GlobalHandle {
    value: Option<WasmValue>,
}

impl GlobalHandle {
    #[must_use]
    pub fn value(&mut self, type_: &WasmType) -> WasmValue {
        if let Some(value) = &self.value {
            assert_eq!(value.type_(), *type_);
            value.clone()
        } else {
            let default = WasmValue::default_for(type_);
            self.value = Some(default.clone());
            default
        }
    }

    pub fn replace_value_with(&mut self, value: WasmValue) {
        self.value = Some(value)
    }
}

#[must_use]
pub fn global_address(x: usize) -> GlobalAddress {
    GlobalAddress(x)
}

#[must_use]
pub fn global(address: GlobalAddress) -> &'static mut GlobalHandle {
    let global_store = unsafe { addr_of_mut!(GLOBAL_STORE).as_mut().unwrap() };
    let GlobalAddress(effective_address) = address;
    &mut global_store[effective_address]
}

pub fn assert_global_exists(address: &GlobalAddress) {
    let global_store = global_store_mut();
    if global_store.len() <= address.value() {
        global_store_mut().resize_with(address.value() + 1, Default::default);
    }
}

pub fn assert_global_value(actual_value: &WasmValue, shadow_value: &WasmValue) {
    let target_globals_not_initialised = unsafe { TRGT_GLOBALS_NOT_INITIALISED };
    let target_globals_only_affected_internally = unsafe { TRGT_GLOBALS_ONLY_AFFECTED_INTERNALLY };
    let assertion_should_hold =
        target_globals_not_initialised && target_globals_only_affected_internally;
    if assertion_should_hold {
        assert_eq!(actual_value, shadow_value);
    }
}

/////////////////
// private API //
/////////////////

#[must_use]
fn global_store_mut() -> &'static mut Vec<GlobalHandle> {
    unsafe { addr_of_mut!(GLOBAL_STORE).as_mut().unwrap() }
}

// https://webassembly.github.io/spec/core/exec/runtime.html#store
type GlobalStore = Vec<GlobalHandle>;
static mut GLOBAL_STORE: GlobalStore = vec![];
