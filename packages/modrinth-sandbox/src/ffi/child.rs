use std::{
    io::{PipeReader, PipeWriter, Read, Write},
    ptr::NonNull,
    slice,
    sync::Arc,
};

use eyre::{Context, ContextCompat, Result};
use libc::size_t;

use crate::{
    SandboxChild, SandboxExitStatus,
    ffi::op::{free, try_do, try_return_optional},
};

pub struct ModrinthSandboxChild {
    pub(super) inner: SandboxChild,
    pub(super) rt: Arc<tokio::runtime::Runtime>,
}

pub struct ModrinthSandboxPipeReader {
    inner: PipeReader,
}

pub struct ModrinthSandboxPipeWriter {
    inner: PipeWriter,
}

/// C representation of a sandboxed child's exit status.
///
/// If `exited` is false, the other fields have no meaning. If `has_code` is
/// false, the process terminated without an exit code, such as from a signal.
#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct ModrinthSandboxExitStatus {
    pub exited: bool,
    pub success: bool,
    pub has_code: bool,
    pub code: i32,
}

impl From<Option<SandboxExitStatus>> for ModrinthSandboxExitStatus {
    fn from(status: Option<SandboxExitStatus>) -> Self {
        let Some(status) = status else {
            return Self::default();
        };
        let code = status.code();
        Self {
            exited: true,
            success: status.success(),
            has_code: code.is_some(),
            code: code.unwrap_or_default(),
        }
    }
}

unsafe fn write_output<T>(out: *mut T, value: T, name: &str) -> Result<()> {
    let out = NonNull::new(out)
        .wrap_err_with(|| format!("`{name}` must not be null"))?;

    // SAFETY: The caller guarantees that `out` points to valid, properly
    // aligned storage for one `T` and does not alias an active reference.
    unsafe { out.write(value) };
    Ok(())
}

/// Returns the child's platform process ID, or zero when no stable process ID
/// is available for this backend or the child has already exited.
///
/// # Safety
///
/// `child` must point to a live child returned by
/// [`crate::ffi::env::modrinth_sandbox_spawn`]. `out_id` must be properly
/// aligned and valid for writing one `u32`. Neither pointer may be concurrently
/// accessed for the duration of this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modrinth_sandbox_child_id(
    child: *const ModrinthSandboxChild,
    out_id: *mut u32,
) -> bool {
    try_do(|| {
        // SAFETY: The function's safety contract requires `child` to point to
        // a live child for the duration of this call.
        let child =
            unsafe { child.as_ref() }.wrap_err("`child` must not be null")?;
        let id = child.inner.id().unwrap_or_default();

        // SAFETY: The function's safety contract requires `out_id` to point to
        // valid, writable storage for one `u32`.
        unsafe { write_output(out_id, id, "out_id") }
    })
}

/// Attempts to collect the child's exit status without blocking.
///
/// On success, `out_status.exited` is false when the child is still running.
///
/// # Safety
///
/// `child` must point to a live child returned by
/// [`crate::ffi::env::modrinth_sandbox_spawn`] and be exclusively accessible
/// for the duration of this call. `out_status` must be properly aligned and
/// valid for writing one [`ModrinthSandboxExitStatus`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modrinth_sandbox_child_try_wait(
    child: *mut ModrinthSandboxChild,
    out_status: *mut ModrinthSandboxExitStatus,
) -> bool {
    try_do(|| {
        // SAFETY: The function's safety contract guarantees exclusive access
        // to a live child for the duration of this call.
        let child =
            unsafe { child.as_mut() }.wrap_err("`child` must not be null")?;
        let status = child
            .inner
            .try_wait()
            .wrap_err("checking sandboxed child status")?;

        // SAFETY: The function's safety contract requires `out_status` to
        // point to valid, writable status storage.
        unsafe { write_output(out_status, status.into(), "out_status") }
    })
}

/// Blocks until the child exits and writes its exit status.
///
/// # Safety
///
/// `child` must point to a live child returned by
/// [`crate::ffi::env::modrinth_sandbox_spawn`] and be exclusively accessible
/// for the duration of this call. `out_status` must be properly aligned and
/// valid for writing one [`ModrinthSandboxExitStatus`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modrinth_sandbox_child_wait(
    child: *mut ModrinthSandboxChild,
    out_status: *mut ModrinthSandboxExitStatus,
) -> bool {
    try_do(|| {
        // SAFETY: The function's safety contract guarantees exclusive access
        // to a live child for the duration of this call.
        let child =
            unsafe { child.as_mut() }.wrap_err("`child` must not be null")?;
        let rt = Arc::clone(&child.rt);
        let status = rt
            .block_on(child.inner.wait())
            .wrap_err("waiting for sandboxed child")?;

        // SAFETY: The function's safety contract requires `out_status` to
        // point to valid, writable status storage.
        unsafe { write_output(out_status, Some(status).into(), "out_status") }
    })
}

/// Forces the child process to exit.
///
/// # Safety
///
/// `child` must point to a live child returned by
/// [`crate::ffi::env::modrinth_sandbox_spawn`] and be exclusively accessible
/// for the duration of this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modrinth_sandbox_child_kill(
    child: *mut ModrinthSandboxChild,
) -> bool {
    try_do(|| {
        // SAFETY: The function's safety contract guarantees exclusive access
        // to a live child for the duration of this call.
        let child =
            unsafe { child.as_mut() }.wrap_err("`child` must not be null")?;
        let rt = Arc::clone(&child.rt);
        rt.block_on(child.inner.kill())
            .wrap_err("killing sandboxed child")
    })
}

/// Takes ownership of the child's standard-input pipe.
///
/// On success, `*out_stdin` is null if stdin was not configured as a pipe or
/// was already taken.
///
/// # Safety
///
/// `child` must point to a live child returned by
/// [`crate::ffi::env::modrinth_sandbox_spawn`] and be exclusively accessible
/// for the duration of this call. `out_stdin` must be properly aligned and
/// valid for writing a `*mut ModrinthSandboxPipeWriter`, and must not contain an
/// owned handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modrinth_sandbox_child_take_stdin(
    child: *mut ModrinthSandboxChild,
    out_stdin: *mut *mut ModrinthSandboxPipeWriter,
) -> bool {
    // SAFETY: The function's safety contract requires `out_stdin` to point to
    // valid, writable pointer storage.
    unsafe {
        try_return_optional(out_stdin, || {
            // SAFETY: The function's safety contract guarantees exclusive
            // access to a live child for the duration of this call.
            let child = child.as_mut().wrap_err("`child` must not be null")?;
            Ok(child
                .inner
                .stdin
                .take()
                .map(|inner| ModrinthSandboxPipeWriter { inner }))
        })
    }
}

/// Takes ownership of the child's standard-output pipe.
///
/// On success, `*out_stdout` is null if stdout was not configured as a pipe or
/// was already taken.
///
/// # Safety
///
/// `child` must point to a live child returned by
/// [`crate::ffi::env::modrinth_sandbox_spawn`] and be exclusively accessible
/// for the duration of this call. `out_stdout` must be properly aligned and
/// valid for writing a `*mut ModrinthSandboxPipeReader`, and must not contain an
/// owned handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modrinth_sandbox_child_take_stdout(
    child: *mut ModrinthSandboxChild,
    out_stdout: *mut *mut ModrinthSandboxPipeReader,
) -> bool {
    // SAFETY: The function's safety contract requires `out_stdout` to point to
    // valid, writable pointer storage.
    unsafe {
        try_return_optional(out_stdout, || {
            // SAFETY: The function's safety contract guarantees exclusive
            // access to a live child for the duration of this call.
            let child = child.as_mut().wrap_err("`child` must not be null")?;
            Ok(child
                .inner
                .stdout
                .take()
                .map(|inner| ModrinthSandboxPipeReader { inner }))
        })
    }
}

/// Takes ownership of the child's standard-error pipe.
///
/// On success, `*out_stderr` is null if stderr was not configured as a pipe or
/// was already taken.
///
/// # Safety
///
/// `child` must point to a live child returned by
/// [`crate::ffi::env::modrinth_sandbox_spawn`] and be exclusively accessible
/// for the duration of this call. `out_stderr` must be properly aligned and
/// valid for writing a `*mut ModrinthSandboxPipeReader`, and must not contain an
/// owned handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modrinth_sandbox_child_take_stderr(
    child: *mut ModrinthSandboxChild,
    out_stderr: *mut *mut ModrinthSandboxPipeReader,
) -> bool {
    // SAFETY: The function's safety contract requires `out_stderr` to point to
    // valid, writable pointer storage.
    unsafe {
        try_return_optional(out_stderr, || {
            // SAFETY: The function's safety contract guarantees exclusive
            // access to a live child for the duration of this call.
            let child = child.as_mut().wrap_err("`child` must not be null")?;
            Ok(child
                .inner
                .stderr
                .take()
                .map(|inner| ModrinthSandboxPipeReader { inner }))
        })
    }
}

/// Reads up to `buffer_len` bytes from a child output pipe.
///
/// A successful read of zero bytes indicates end-of-file. A null `buffer` is
/// permitted only when `buffer_len` is zero.
///
/// # Safety
///
/// `reader` must point to a live reader returned by a child pipe accessor and
/// be exclusively accessible for the duration of this call. If `buffer_len` is
/// nonzero, `buffer` must be valid for writes of `buffer_len` bytes and may not
/// alias `reader` or `out_read`. `out_read` must be properly aligned and valid
/// for writing one `size_t`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modrinth_sandbox_pipe_reader_read(
    reader: *mut ModrinthSandboxPipeReader,
    buffer: *mut u8,
    buffer_len: size_t,
    out_read: *mut size_t,
) -> bool {
    try_do(|| {
        // SAFETY: The function's safety contract guarantees exclusive access
        // to a live reader for the duration of this call.
        let reader =
            unsafe { reader.as_mut() }.wrap_err("`reader` must not be null")?;
        let buffer = if buffer_len == 0 {
            &mut []
        } else {
            let buffer = NonNull::new(buffer).wrap_err(
                "`buffer` must not be null when `buffer_len` is nonzero",
            )?;
            // SAFETY: The function's safety contract guarantees that `buffer`
            // is valid for writes of `buffer_len` bytes and uniquely borrowed.
            unsafe { slice::from_raw_parts_mut(buffer.as_ptr(), buffer_len) }
        };
        let read = reader
            .inner
            .read(buffer)
            .wrap_err("reading child output pipe")?;

        // SAFETY: The function's safety contract requires `out_read` to point
        // to valid, writable storage for one `size_t`.
        unsafe { write_output(out_read, read, "out_read") }
    })
}

/// Writes up to `buffer_len` bytes to a child input pipe.
///
/// A null `buffer` is permitted only when `buffer_len` is zero.
///
/// # Safety
///
/// `writer` must point to a live writer returned by
/// [`modrinth_sandbox_child_take_stdin`] and be exclusively accessible for the
/// duration of this call. If `buffer_len` is nonzero, `buffer` must be valid for
/// reads of `buffer_len` bytes and may not alias `writer` or `out_written`.
/// `out_written` must be properly aligned and valid for writing one `size_t`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modrinth_sandbox_pipe_writer_write(
    writer: *mut ModrinthSandboxPipeWriter,
    buffer: *const u8,
    buffer_len: size_t,
    out_written: *mut size_t,
) -> bool {
    try_do(|| {
        // SAFETY: The function's safety contract guarantees exclusive access
        // to a live writer for the duration of this call.
        let writer =
            unsafe { writer.as_mut() }.wrap_err("`writer` must not be null")?;
        let buffer = if buffer_len == 0 {
            &[]
        } else {
            let buffer = NonNull::new(buffer.cast_mut()).wrap_err(
                "`buffer` must not be null when `buffer_len` is nonzero",
            )?;
            // SAFETY: The function's safety contract guarantees that `buffer`
            // is valid for reads of `buffer_len` bytes.
            unsafe { slice::from_raw_parts(buffer.as_ptr(), buffer_len) }
        };
        let written = writer
            .inner
            .write(buffer)
            .wrap_err("writing child input pipe")?;

        // SAFETY: The function's safety contract requires `out_written` to
        // point to valid, writable storage for one `size_t`.
        unsafe { write_output(out_written, written, "out_written") }
    })
}

/// Flushes a child input pipe.
///
/// # Safety
///
/// `writer` must point to a live writer returned by
/// [`modrinth_sandbox_child_take_stdin`] and be exclusively accessible for the
/// duration of this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modrinth_sandbox_pipe_writer_flush(
    writer: *mut ModrinthSandboxPipeWriter,
) -> bool {
    try_do(|| {
        // SAFETY: The function's safety contract guarantees exclusive access
        // to a live writer for the duration of this call.
        let writer =
            unsafe { writer.as_mut() }.wrap_err("`writer` must not be null")?;
        writer.inner.flush().wrap_err("flushing child input pipe")
    })
}

/// Frees a child output pipe reader.
///
/// # Safety
///
/// `reader` must be null or a pointer returned by a child pipe accessor that
/// has not already been freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modrinth_sandbox_pipe_reader_free(
    reader: *mut ModrinthSandboxPipeReader,
) {
    // SAFETY: The function's safety contract requires every non-null pointer
    // to be a uniquely owned allocation returned by `try_return_optional`.
    unsafe { free(reader) };
}

/// Frees a child input pipe writer, closing the child's standard input.
///
/// # Safety
///
/// `writer` must be null or a pointer returned by
/// [`modrinth_sandbox_child_take_stdin`] that has not already been freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modrinth_sandbox_pipe_writer_free(
    writer: *mut ModrinthSandboxPipeWriter,
) {
    // SAFETY: The function's safety contract requires every non-null pointer
    // to be a uniquely owned allocation returned by `try_return_optional`.
    unsafe { free(writer) };
}

/// Frees a child returned by [`crate::ffi::modrinth_sandbox_spawn`].
///
/// # Safety
///
/// `child` must be null or a pointer returned by
/// [`crate::ffi::env::modrinth_sandbox_spawn`] that has not already been freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modrinth_sandbox_child_free(
    child: *mut ModrinthSandboxChild,
) {
    // SAFETY: The function's safety contract requires every non-null pointer
    // to be a uniquely owned allocation returned by `try_return`.
    unsafe { free(child) };
}
