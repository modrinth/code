use std::{ptr::NonNull, sync::Arc};

use eyre::{Context, ContextCompat};

use crate::{
    SandboxEnv,
    ffi::{
        ModrinthSandboxCommand,
        child::ModrinthSandboxChild,
        op::{free, try_return},
    },
};

pub struct ModrinthSandboxEnv {
    pub(super) env: SandboxEnv,
    pub(super) rt: Arc<tokio::runtime::Runtime>,
}

/// Creates a sandbox environment and returns an opaque, Rust-owned handle.
///
/// # Safety
///
/// If `out_env` is non-null, it must be properly aligned and valid for writing
/// a `*mut ModrinthSandboxEnv`. It must not currently contain an owned handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modrinth_sandbox_create_env(
    out_env: *mut *mut ModrinthSandboxEnv,
) -> bool {
    // SAFETY: The function's safety contract requires `out_env` to point to
    // valid, writable pointer storage when non-null.
    unsafe {
        try_return(out_env, || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .build()
                .wrap_err("creating tokio runtime")?;
            let env = rt.block_on(SandboxEnv::new())?;
            Ok(ModrinthSandboxEnv {
                env,
                rt: Arc::new(rt),
            })
        })
    }
}

/// Frees a sandbox environment returned by [`modrinth_sandbox_create_env`].
///
/// Children spawned through the environment remain usable after the environment
/// is freed because they retain the runtime needed for asynchronous operations.
///
/// # Safety
///
/// `env` must be null or a pointer returned by
/// [`modrinth_sandbox_create_env`] that has not already been freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modrinth_sandbox_env_free(
    env: *mut ModrinthSandboxEnv,
) {
    // SAFETY: The function's safety contract requires every non-null pointer
    // to be a uniquely owned allocation returned by `try_return`.
    unsafe { free(env) };
}

/// Spawns a sandboxed child and consumes `*command`.
///
/// Once a non-null command handle has been validated, `*command` is set to
/// null and the command is consumed whether spawning succeeds or fails.
///
/// # Safety
///
/// `env` must point to a live environment returned by
/// [`modrinth_sandbox_create_env`] and must not be freed for the duration of
/// this call. `command` must point to valid, writable pointer storage, and a
/// non-null `*command` must be a uniquely owned handle returned by
/// [`crate::ffi::modrinth_sandbox_prepare_command`]. If `out_child` is
/// non-null, it must be properly aligned and valid for writing a
/// `*mut ModrinthSandboxChild`, and must not contain an owned handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modrinth_sandbox_spawn(
    env: *const ModrinthSandboxEnv,
    command: *mut *mut ModrinthSandboxCommand,
    out_child: *mut *mut ModrinthSandboxChild,
) -> bool {
    let spawn = move || {
        // SAFETY: The caller guarantees that `env` points to a live environment
        // which remains valid for the duration of this call.
        let env = unsafe { env.as_ref() }.wrap_err("`env` must not be null")?;

        // SAFETY: The caller guarantees that `command` points to valid,
        // writable storage containing a command handle.
        let command_slot = unsafe { command.as_mut() }
            .wrap_err("`command` must not be null")?;
        let command = std::mem::replace(command_slot, std::ptr::null_mut());
        let command =
            NonNull::new(command).wrap_err("`*command` must not be null")?;

        // SAFETY: The command pointer was returned by `try_return` during
        // command preparation. Its ownership was removed from the caller by
        // replacing the caller's pointer with null above.
        let command = unsafe { Box::from_raw(command.as_ptr()) };
        let ModrinthSandboxCommand { inner: command } = *command;

        let child = env
            .rt
            .block_on(env.env.spawn(command))
            .wrap_err("spawning sandboxed child")?;
        Ok(ModrinthSandboxChild {
            inner: child,
            rt: Arc::clone(&env.rt),
        })
    };

    // SAFETY: The function's safety contract requires `out_child` to point to
    // valid, writable pointer storage when non-null.
    unsafe { try_return(out_child, spawn) }
}
