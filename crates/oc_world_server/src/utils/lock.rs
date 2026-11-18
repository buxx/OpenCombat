#[macro_export]
macro_rules! rw_lock_imports {
    () => {
        #[cfg(feature = "debug_locks")]
        use oc_utils::debug::lock::{RwLock, RwLockReadGuard, RwLockWriteGuard};
        #[cfg(not(feature = "debug_locks"))]
        use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
    };
}

#[macro_export]
#[cfg(not(feature = "debug_locks"))]
macro_rules! rw_lock {
    ($value:expr) => {
        RwLock::new($value)
    };
}

#[macro_export]
#[cfg(feature = "debug_locks")]
macro_rules! rw_lock {
    ($value:expr) => {
        RwLock::with_period_and_callback(
            $value,
            std::time::Duration::from_secs(1),
            Some(Arc::new(|report| tracing::error!("{}", report.log()))),
        )
    };
}
