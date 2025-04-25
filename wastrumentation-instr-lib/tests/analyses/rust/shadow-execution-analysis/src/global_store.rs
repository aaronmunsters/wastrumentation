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
    /// Get value from global handle.
    /// The actual value allows us to assert that the global value in the
    /// shadow execution matches the actual global value.
    // FIXME: Split this into an assertion where the caller is debug_assert!;
    //        this kind of assertion is skipped in --release builds.
    #[must_use]
    pub fn value(&mut self, type_: &WasmType, actual: &WasmValue) -> WasmValue {
        if let Some(value) = &self.value {
            assert_eq!(value.type_(), *type_);
            assert_eq!(value, actual);
            // If this assertion fails, the host must have changed it.
            // If the host changed it and we were not notified, this is a bug.
            // FIXME: add infrastructure for notification.
            value.clone()
        } else {
            // The actual value will be either the default value (if
            // uninitialized) or a fixed compile-time value if an
            // initializer is present in the binary.
            self.value = Some(actual.clone());
            actual.clone()
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
