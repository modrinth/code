use std::{io::{Error, ErrorKind}, os::windows::io::AsRawHandle, path::{Path, PathBuf}};

use windows::Win32::{Foundation::HANDLE, System::JobObjects::AssignProcessToJobObject, UI::Shell::SHELLEXECUTEINFOW};
use eyre::{Result, WrapErr, eyre};

use crate::SandboxArg;

pub fn spawn(
    program: PathBuf,
    arguments: Vec<SandboxArg>,
) -> Result<super::WindowsChild> {
    use std::os::windows::ffi::OsStrExt;
    let program = super::resolve_path(&program).wrap_err("resolving program path")?;
    let application_name = program
        .as_os_str()
        .encode_wide()
        .chain([0])
        .collect::<Vec<_>>();
    let mut command_line = super::join_windows_shell_arg(arguments.as_slice())
        .encode_wide()
        .chain([0])
        .collect::<Vec<_>>();

    let mut sei: SHELLEXECUTEINFOW = SHELLEXECUTEINFOW::default();
    sei.fMask = windows::Win32::UI::Shell::SEE_MASK_NOASYNC | windows::Win32::UI::Shell::SEE_MASK_NOCLOSEPROCESS;
    sei.cbSize = std::mem::size_of::<SHELLEXECUTEINFOW>() as _;
    sei.lpVerb = windows::core::w!("runas");
    sei.lpFile = windows::core::PCWSTR(application_name.as_ptr());
    sei.lpParameters = windows::core::PCWSTR::from_raw(command_line.as_ptr());
    sei.nShow = windows::Win32::UI::WindowsAndMessaging::SW_HIDE.0;

    unsafe {
        windows::Win32::UI::Shell::ShellExecuteExW(&mut sei)?;
    }

    if sei.hProcess.is_invalid() {
        return Err(eyre!("ShellExecuteExW returned invalid process handle. Operation completed via DDE?"));
    }

    Ok(super::WindowsChild {
        process_handle: sei.hProcess,
        exit_status: None,
    })
}
