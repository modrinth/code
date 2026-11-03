use std::{ffi::OsString, ptr::NonNull};

use eyre::ContextCompat;

use super::{op::try_do, str::ModrinthSandboxStringSlice};

/// Handles a sandbox helper invocation for the default helper command.
///
/// Call this near the beginning of the executable, before parsing application
/// arguments or creating a sandbox environment. On success, `out_handled` is
/// true when a helper invocation was handled; the caller should then exit
/// successfully without continuing application initialization. Otherwise,
/// normal application startup should continue.
///
/// Returns false on failure and sets the last FFI error. A non-null
/// `out_handled` is initialized to false before running the helper.
///
/// # Safety
///
/// If `out_handled` is non-null, it must be properly aligned and valid for
/// writing one `bool`, and must not be concurrently accessed during this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modrinth_sandbox_helper_run_default(
    out_handled: *mut bool,
) -> bool {
    try_do(|| {
        let out_handled = NonNull::new(out_handled)
            .wrap_err("`out_handled` must not be null")?;

        // SAFETY: The caller guarantees valid, writable storage for one bool.
        unsafe { out_handled.write(false) };
        let handled = crate::helper::run_default()?;
        // SAFETY: The output storage remains valid for the duration of the call.
        unsafe { out_handled.write(handled) };
        Ok(())
    })
}

/// Handles a sandbox helper invocation using the provided UTF-8 arguments.
///
/// The argument list must not include the executable name or
/// `--modrinth-sandbox-helper` argument. On success, `out_handled` is true
/// when a helper invocation was handled; the caller should then exit
/// successfully without continuing application initialization.
///
/// Returns false on failure and sets the last FFI error. A non-null
/// `out_handled` is initialized to false before converting arguments or
/// running the helper. A null argument pointer with zero length is allowed.
///
/// # Safety
///
/// Every pointer reachable through `args` must uphold
/// [`ModrinthSandboxStringSlice::try_to_owned`]'s safety contract for the
/// duration of this call. If `out_handled` is non-null, it must be properly
/// aligned and valid for writing one `bool`, and must not be concurrently
/// accessed during this call or overlap the argument storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modrinth_sandbox_helper_run_with_args(
    args: ModrinthSandboxStringSlice,
    out_handled: *mut bool,
) -> bool {
    try_do(|| {
        let out_handled = NonNull::new(out_handled)
            .wrap_err("`out_handled` must not be null")?;

        // SAFETY: The caller guarantees valid, writable storage for one bool.
        unsafe { out_handled.write(false) };
        // SAFETY: The caller guarantees valid argument descriptors and strings.
        let args = unsafe { args.try_to_owned() }?;
        let handled = crate::helper::run_with_args(
            args.into_iter().map(String::from).map(OsString::from),
        )?;
        // SAFETY: The output storage remains valid for the duration of the call.
        unsafe { out_handled.write(handled) };
        Ok(())
    })
}
