use eyre::{Context, Result, bail};

use crate::ffi::{
    op::{free, try_return},
    str::{
        ModrinthSandboxString, ModrinthSandboxStringOption,
        ModrinthSandboxStringPairsSlice, ModrinthSandboxStringSlice,
    },
};

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
    pub child_stdin: u32,
    pub child_stdout: u32,
    pub child_stderr: u32,
    pub app_container_name: ModrinthSandboxString,
    pub app_container_description: ModrinthSandboxString,
}

pub struct ModrinthSandboxCommand {
    pub(super) inner: crate::SandboxCommand,
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

            let stdin = stdio(command_view.child_stdin)
                .wrap_err("converting `child_stdin`")?;
            let stdout = stdio(command_view.child_stdout)
                .wrap_err("converting `child_stdout`")?;
            let stderr = stdio(command_view.child_stderr)
                .wrap_err("converting `child_stderr`")?;

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
