use wastrumentation_rs_stdlib::WasmType;

// Imports
use super::WasmValue;
use std::cell::RefCell;

//////////////////////////////////
// compile-time severity checks //
//////////////////////////////////

const TRGT_GLOBALS_NOT_INITIALISED: bool = false;
const TRGT_GLOBALS_ONLY_AFFECTED_INTERNALLY: bool = false;

pub fn assert_global_value(actual_value: &WasmValue, shadow_value: &WasmValue) {
    if TRGT_GLOBALS_NOT_INITIALISED && TRGT_GLOBALS_ONLY_AFFECTED_INTERNALLY {
        debug_assert_eq!(actual_value, shadow_value);
    }
}

////////////////
// Public API //
////////////////

thread_local! {
    pub(crate) static GLOBAL_STORE: RefCell<GlobalStore> = const { RefCell::new(GlobalStore(vec![])) };
}

// https://webassembly.github.io/spec/core/exec/runtime.html#store
pub(crate) struct GlobalStore(Vec<GlobalHandle>);

pub(crate) struct GlobalAddress(usize);

impl GlobalAddress {
    #[must_use]
    pub(crate) fn new(x: usize) -> Self {
        Self(x)
    }

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
    pub fn value(&mut self, type_: WasmType, actual: &WasmValue) -> WasmValue {
        if let Some(value) = &self.value {
            debug_assert_eq!(value.type_(), type_);
            debug_assert_eq!(value, actual);
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
        self.value = Some(value);
    }
}

impl GlobalStore {
    #[must_use]
    pub fn global(&mut self, address: &GlobalAddress) -> &mut GlobalHandle {
        let Self(global_store) = self;
        let GlobalAddress(effective_address) = address;
        &mut global_store[*effective_address]
    }

    pub fn assert_global_exists(&mut self, address: &GlobalAddress) {
        let Self(global_store) = self;
        if global_store.len() <= address.value() {
            global_store.resize_with(address.value() + 1, Default::default);
        }
    }
}
