use wastrumentation_rs_stdlib::WasmType;

use crate::{ ShadowMeta, ShadowValue };

//////////////////////////////////
// compile-time severity checks //
//////////////////////////////////

const TRGT_GLOBALS_NOT_INITIALISED: bool = false;
const TRGT_GLOBALS_ONLY_AFFECTED_INTERNALLY: bool = false;

pub fn assert_global_value<M: ShadowMeta>(
    actual_value: &ShadowValue<M>,
    shadow_value: &ShadowValue<M>
) {
    if TRGT_GLOBALS_NOT_INITIALISED && TRGT_GLOBALS_ONLY_AFFECTED_INTERNALLY {
        debug_assert_eq!(actual_value, shadow_value);
    }
}

////////////////
// Public API //
////////////////

// https://webassembly.github.io/spec/core/exec/runtime.html#store

pub struct GlobalHandle<M: ShadowMeta> {
    value: Option<ShadowValue<M>>,
}

impl<M: ShadowMeta> Default for GlobalHandle<M> {
    fn default() -> Self {
        Self { value: None }
    }
}

impl<M: ShadowMeta> GlobalHandle<M> {
    /// Get value from global handle.
    /// The actual value allows us to assert that the global value in the
    /// shadow execution matches the actual global value.
    // FIXME: Split this into an assertion where the caller is debug_assert!;
    //        this kind of assertion is skipped in --release builds.
    #[must_use]
    pub fn value(&mut self, type_: WasmType, actual: &ShadowValue<M>) -> ShadowValue<M> {
        if let Some(shadow_value) = &self.value {
            debug_assert_eq!(shadow_value.value.type_(), type_);
            debug_assert_eq!(shadow_value, actual);
            // If this assertion fails, the host must have changed it.
            // If the host changed it and we were not notified, this is a bug.
            // FIXME: add infrastructure for notification.
            shadow_value.clone()
        } else {
            // The actual value will be either the default value (if
            // uninitialized) or a fixed compile-time value if an
            // initializer is present in the binary.
            self.value = Some(actual.clone());
            actual.clone()
        }
    }

    /// Overwrite the shadow value for this global (e.g. after a `global.set`).
    pub fn replace_value_with(&mut self, value: ShadowValue<M>) {
        self.value = Some(value);
    }
}

pub struct GlobalAddress(usize);

impl GlobalAddress {
    #[must_use]
    pub fn new(x: usize) -> Self {
        Self(x)
    }

    #[must_use]
    pub fn value(&self) -> usize {
        let Self(value) = self;
        *value
    }
}

pub struct GlobalStore<M: ShadowMeta>(Vec<GlobalHandle<M>>);

impl<M: ShadowMeta> GlobalStore<M> {
    pub const fn new() -> Self {
        Self(vec![])
    }
}

impl<M: ShadowMeta> Default for GlobalStore<M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<M: ShadowMeta> GlobalStore<M> {
    #[must_use]
    pub fn global(&mut self, address: &GlobalAddress) -> &mut GlobalHandle<M> {
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
