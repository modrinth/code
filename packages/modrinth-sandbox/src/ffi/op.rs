use std::{
    ptr::{self, NonNull},
    sync::Mutex,
};

use eyre::ContextCompat;
use libc::size_t;

static LAST_ERROR: Mutex<Option<String>> = Mutex::new(None);

/// Returns the most recent FFI error message.
///
/// The returned message is not null-terminated and remains valid until the
/// next FFI operation that updates the last error.
///
/// # Safety
///
/// `out_message` and `out_message_len` must be valid, properly aligned,
/// writable pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modrinth_sandbox_get_last_error(
    out_message: *mut *const u8,
    out_message_len: *mut size_t,
) {
    let error = LAST_ERROR.lock().expect("should never be poisoned");
    if let Some(error) = error.as_ref() {
        // SAFETY: The function's safety contract requires both output
        // parameters to point to valid, writable storage.
        unsafe {
            out_message.write(error.as_ptr());
            out_message_len.write(error.len());
        }
    } else {
        // SAFETY: The function's safety contract requires both output
        // parameters to point to valid, writable storage.
        unsafe {
            out_message.write(std::ptr::null());
            out_message_len.write(0);
        }
    }
}

/// Runs `f` and handles failure.
///
/// Before every run, the last error is cleared.
///
/// If `f` returns [`Ok`], this returns `true` and no error is written.
///
/// If `f` returns [`Err`], this returns `false` and sets the last error to
/// the error message.
pub fn try_do(f: impl FnOnce() -> eyre::Result<()>) -> bool {
    *LAST_ERROR.lock().expect("should never be poisoned") = None;
    match f() {
        Ok(()) => true,
        Err(err) => {
            *LAST_ERROR.lock().expect("should never be poisoned") =
                Some(format!("{err:#}"));
            false
        }
    }
}

/// Runs `f` and writes its return value to a Rust-owned opaque pointer.
///
/// # Safety
///
/// If `out` is non-null, it must be properly aligned and valid for writing a
/// `*mut R`. It must not currently contain an owned pointer that would be
/// leaked by overwriting it.
pub unsafe fn try_return<R>(
    out: *mut *mut R,
    f: impl FnOnce() -> eyre::Result<R>,
) -> bool {
    try_do(move || {
        let out = NonNull::new(out)
            .wrap_err("out parameter value must not be null")?;

        // SAFETY: The caller guarantees that `out` points to valid, writable
        // storage for one pointer. Writing null establishes the failure state
        // without reading or dropping the previous pointer value.
        unsafe { out.write(ptr::null_mut()) };

        match f() {
            Ok(r) => {
                let r = Box::new(r);

                // SAFETY: The caller guarantees that `out` points to valid,
                // writable storage for one pointer. `Box::into_raw` returns a
                // valid owned pointer for the caller to release through the
                // corresponding FFI free function.
                unsafe { out.write(Box::into_raw(r)) };
                Ok(())
            }
            Err(err) => Err(err),
        }
    })
}

/// Frees the value behind `ptr` if the pointer is not null.
///
/// # Safety
///
/// `ptr` must be null or point to a valid `T` that has not already been freed.
pub unsafe fn free<T>(ptr: *mut T) {
    if !ptr.is_null() {
        let value = unsafe { Box::from_raw(ptr) };
        drop(value);
    }
}
