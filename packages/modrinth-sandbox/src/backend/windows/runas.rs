use std::os::windows::ffi::OsStrExt;

use eyre::{Result, eyre};
use windows::Win32::UI::Shell::SHELLEXECUTEINFOW;

use crate::{SandboxArg, helper::MakeHelper};

pub fn spawn(
    make_helper: MakeHelper,
    extra_arguments: Vec<SandboxArg>,
) -> Result<super::WindowsChild> {
    let helper = (make_helper)();
    let application_name = helper
        .get_program()
        .encode_wide()
        .chain([0])
        .collect::<Vec<_>>();
    let arguments = helper
        .get_args()
        .map(|argument| argument.to_os_string().into())
        .chain(extra_arguments)
        .collect::<Vec<_>>();
    let command_line = super::join_windows_shell_arg(arguments.as_slice())
        .encode_wide()
        .chain([0])
        .collect::<Vec<_>>();

    let mut sei: SHELLEXECUTEINFOW = SHELLEXECUTEINFOW::default();
    sei.fMask = windows::Win32::UI::Shell::SEE_MASK_NOASYNC
        | windows::Win32::UI::Shell::SEE_MASK_NOCLOSEPROCESS;
    sei.cbSize = std::mem::size_of::<SHELLEXECUTEINFOW>() as _;
    sei.lpVerb = windows::core::w!("runas");
    sei.lpFile = windows::core::PCWSTR(application_name.as_ptr());
    sei.lpParameters = windows::core::PCWSTR::from_raw(command_line.as_ptr());
    sei.nShow = windows::Win32::UI::WindowsAndMessaging::SW_HIDE.0;

    unsafe {
        windows::Win32::UI::Shell::ShellExecuteExW(&mut sei)?;
    }

    if sei.hProcess.is_invalid() {
        return Err(eyre!(
            "ShellExecuteExW returned invalid process handle. Operation completed via DDE?"
        ));
    }

    Ok(super::WindowsChild {
        process_handle: sei.hProcess,
        exit_status: None,
    })
}
