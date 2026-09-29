mod op;
mod str;

use eyre::{Context, ContextCompat, Result, bail};

use crate::ffi::{
    op::{free, try_return},
    str::{
        ModrinthSandboxString, ModrinthSandboxStringOption,
        ModrinthSandboxStringPairsSlice, ModrinthSandboxStringSlice,
    },
};

pub struct ModrinthSandboxEnv {
    env: crate::SandboxEnv,
    rt: tokio::runtime::Runtime,
}

#[repr(C)]
pub struct ModrinthSandboxCommandView {
    pub executable: ModrinthSandboxString,
    pub args: ModrinthSandboxStringSlice,
    pub ensure_dirs_exist: ModrinthSandboxStringSlice,
    pub read_only_paths: ModrinthSandboxStringSlice,
    pub read_write_paths: ModrinthSandboxStringSlice,
    pub working_directory: ModrinthSandboxStringOption,
    pub passthrough_environment: ModrinthSandboxStringSlice,
    pub extra_environment: ModrinthSandboxStringPairsSlice,
    pub allow_network: bool,
    pub is_jvm: bool,
    pub die_with_parent: bool,
    pub stdin: u32,
    pub stdout: u32,
    pub stderr: u32,
    pub app_container_name: ModrinthSandboxString,
    pub app_container_description: ModrinthSandboxString,
}

pub struct ModrinthSandboxCommand {
    inner: crate::SandboxCommand,
}

pub struct ModrinthSandboxChild {
    inner: crate::SandboxChild,
}

pub const MODRINTH_SANDBOX_STDIO_NULL: u32 = 0;
pub const MODRINTH_SANDBOX_STDIO_INHERIT: u32 = 1;
pub const MODRINTH_SANDBOX_STDIO_PIPE: u32 = 2;

fn stdio(value: u32) -> Result<crate::SandboxStdio> {
    match value {
        MODRINTH_SANDBOX_STDIO_NULL => Ok(crate::SandboxStdio::Null),
        MODRINTH_SANDBOX_STDIO_INHERIT => Ok(crate::SandboxStdio::Inherit),
        MODRINTH_SANDBOX_STDIO_PIPE => Ok(crate::SandboxStdio::Pipe),
        _ => bail!("unknown sandbox stdio value {value}"),
    }
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
    // SAFETY: caller upholds the variants.
    unsafe {
        try_return(out_env, || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .build()
                .wrap_err("creating tokio runtime")?;
            let env = rt.block_on(crate::create_env())?;
            Ok(ModrinthSandboxEnv { env, rt })
        })
    }
}

/// Frees a sandbox environment returned by [`modrinth_sandbox_create_env`].
///
/// # Safety
///
/// `env` must be null or a pointer returned by
/// [`modrinth_sandbox_create_env`] that has not already been freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modrinth_sandbox_env_free(
    env: *mut ModrinthSandboxEnv,
) {
    unsafe { free(env) };
}

/// Copies a borrowed C command view into a Rust-owned sandbox command.
///
/// Stdio values use `0` for null, `1` for inherit, and `2` for pipe.
///
/// # Safety
///
/// Every pointer reachable through `command_view` must uphold its string or
/// slice type's safety contract for the duration of this call. If `out_command`
/// is non-null, it must be properly aligned and valid for writing a
/// `*mut ModrinthSandboxCommand`, and must not contain an owned handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modrinth_sandbox_prepare_command(
    command_view: ModrinthSandboxCommandView,
    out_command: *mut *mut ModrinthSandboxCommand,
) -> bool {
    // SAFETY: The function's safety contract requires `out_command` to point
    // to valid, writable pointer storage when non-null. The closure copies all
    // memory borrowed by `command_view` before returning.
    unsafe {
        try_return(out_command, move || {
            let executable = command_view
                .executable
                .try_to_owned()
                .wrap_err("converting `executable`")?;
            let args = command_view
                .args
                .try_to_owned()
                .wrap_err("converting `args`")?;
            let ensure_dirs_exist = command_view
                .ensure_dirs_exist
                .try_to_owned()
                .wrap_err("converting `ensure_dirs_exist`")?;
            let read_only_paths = command_view
                .read_only_paths
                .try_to_owned()
                .wrap_err("converting `read_only_paths`")?;
            let read_write_paths = command_view
                .read_write_paths
                .try_to_owned()
                .wrap_err("converting `read_write_paths`")?;
            let working_directory = command_view
                .working_directory
                .try_to_owned()
                .wrap_err("converting `working_directory`")?;
            let passthrough_environment = command_view
                .passthrough_environment
                .try_to_owned()
                .wrap_err("converting `passthrough_environment`")?;
            let extra_environment = command_view
                .extra_environment
                .try_to_owned()
                .wrap_err("converting `extra_environment`")?;
            let app_container_name = command_view
                .app_container_name
                .try_to_owned()
                .wrap_err("converting `app_container_name`")?;
            let app_container_description = command_view
                .app_container_description
                .try_to_owned()
                .wrap_err("converting `app_container_description`")?;

            let stdin =
                stdio(command_view.stdin).wrap_err("converting `stdin`")?;
            let stdout =
                stdio(command_view.stdout).wrap_err("converting `stdout`")?;
            let stderr =
                stdio(command_view.stderr).wrap_err("converting `stderr`")?;

            let command = crate::SandboxCommand {
                executable: executable.as_ref().into(),
                args: args
                    .into_iter()
                    .map(String::from)
                    .map(Into::into)
                    .collect(),
                ensure_dirs_exist: ensure_dirs_exist
                    .into_iter()
                    .map(|path| path.as_ref().into())
                    .collect(),
                read_only_paths: read_only_paths
                    .into_iter()
                    .map(|path| path.as_ref().into())
                    .collect(),
                read_write_paths: read_write_paths
                    .into_iter()
                    .map(|path| path.as_ref().into())
                    .collect(),
                working_directory: working_directory
                    .map(|path| path.as_ref().into()),
                passthrough_environment: passthrough_environment
                    .into_iter()
                    .map(String::from)
                    .map(Into::into)
                    .collect(),
                extra_environment: extra_environment
                    .into_iter()
                    .map(|(key, value)| (key.into(), value.into()))
                    .collect(),
                allow_network: command_view.allow_network,
                is_jvm: command_view.is_jvm,
                die_with_parent: command_view.die_with_parent,
                stdin,
                stdout,
                stderr,
                app_container_name: String::from(app_container_name).into(),
                app_container_description: String::from(
                    app_container_description,
                )
                .into(),
            };

            Ok(ModrinthSandboxCommand { inner: command })
        })
    }
}

/// Frees a command returned by [`modrinth_sandbox_prepare_command`].
///
/// # Safety
///
/// `command` must be null or a pointer returned by
/// [`modrinth_sandbox_prepare_command`] that has not already been freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modrinth_sandbox_command_free(
    command: *mut ModrinthSandboxCommand,
) {
    // SAFETY: The function's safety contract requires every non-null pointer
    // to be a uniquely owned allocation returned by `try_return`.
    unsafe { free(command) };
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
/// [`modrinth_sandbox_prepare_command`]. If `out_child` is non-null, it must be
/// properly aligned and valid for writing a `*mut ModrinthSandboxChild`, and
/// must not contain an owned handle.
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
        let command = std::ptr::NonNull::new(command)
            .wrap_err("`*command` must not be null")?;

        // SAFETY: The command pointer was returned by `try_return` during
        // command preparation. Its ownership was removed from the caller by
        // replacing the caller's pointer with null above.
        let command = unsafe { Box::from_raw(command.as_ptr()) };
        let ModrinthSandboxCommand { inner: command } = *command;

        let child = env
            .rt
            .block_on(env.env.spawn(command))
            .wrap_err("spawning sandboxed child")?;
        Ok(ModrinthSandboxChild { inner: child })
    };

    // SAFETY: The function's safety contract requires `out_child` to point to
    // valid, writable pointer storage when non-null.
    unsafe { try_return(out_child, spawn) }
}

/// Frees a child returned by [`modrinth_sandbox_spawn`].
///
/// # Safety
///
/// `child` must be null or a pointer returned by [`modrinth_sandbox_spawn`]
/// that has not already been freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn modrinth_sandbox_child_free(
    child: *mut ModrinthSandboxChild,
) {
    // SAFETY: The function's safety contract requires every non-null pointer
    // to be a uniquely owned allocation returned by `try_return`.
    unsafe { free(child) };
}
