//! Panic guard + null-safe handle access for the C ABI boundary.
//!
//! Every export runs its body through [`safe`], so an engine panic is caught here
//! and turned into a no-op result instead of unwinding into the host (Swift/C++) —
//! which would abort the whole IME process. [`with_engine_mut`] / [`with_engine_ref`]
//! fold that guard together with the null-handle check so each export stays a thin,
//! uniform wrapper.

use std::panic::{AssertUnwindSafe, catch_unwind};

use funput_engine::Engine;

use crate::FunputEngine;

/// Run `operation`; if it panics, swallow the unwind and return `default` so the
/// panic never crosses the `extern "C"` boundary into the host (Swift/C++). An
/// engine bug thus drops one keystroke instead of aborting the IME process.
pub(crate) fn safe<T>(default: T, operation: impl FnOnce() -> T) -> T {
    match catch_unwind(AssertUnwindSafe(operation)) {
        Ok(value) => value,
        Err(_) => {
            #[cfg(feature = "vxkey-native")]
            NATIVE_FAULT.store(true, std::sync::atomic::Ordering::Release);
            default
        }
    }
}

/// Run `op` against the engine behind a `*mut` handle, guarded by [`safe`]. A null
/// handle — or a panic inside `op` — is a no-op returning `R::default()`.
///
/// # Safety
/// `engine` must come from `funput_engine_new` (and be unfreed), or be null.
pub(crate) unsafe fn with_engine_mut<R: Default>(
    engine: *mut FunputEngine,
    op: impl FnOnce(&mut Engine) -> R,
) -> R {
    safe(R::default(), || {
        unsafe { engine.as_mut() }.map_or_else(R::default, |e| op(&mut e.inner))
    })
}

/// Read-only counterpart of [`with_engine_mut`] for `*const` handles.
///
/// # Safety
/// `engine` must come from `funput_engine_new` (and be unfreed), or be null.
pub(crate) unsafe fn with_engine_ref<R: Default>(
    engine: *const FunputEngine,
    op: impl FnOnce(&Engine) -> R,
) -> R {
    safe(R::default(), || {
        unsafe { engine.as_ref() }.map_or_else(R::default, |e| op(&e.inner))
    })
}

// A panicked engine is never resumed in-place. The native shell bypasses it until
// process restart, then the durable recovery marker still requires explicit Resume.
#[cfg(feature = "vxkey-native")]
static NATIVE_FAULT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
#[cfg(feature = "vxkey-native")]
pub(crate) fn native_faulted() -> bool {
    NATIVE_FAULT.load(std::sync::atomic::Ordering::Acquire)
}
#[cfg(all(test, feature = "vxkey-native"))]
#[test]
fn native_panic_latches_fault_without_unwinding_across_ffi() {
    assert_eq!(safe(7, || panic!("synthetic engine fault")), 7);
    assert!(native_faulted());
    assert_eq!(safe(9, || 11), 11);
    assert!(native_faulted());
}
