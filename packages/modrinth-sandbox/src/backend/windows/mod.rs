use std::{collections::BTreeMap, ffi::{OsStr, OsString, c_void}, os::windows::{ffi::OsStrExt, io::{AsRawHandle, HandleOrInvalid, OwnedHandle}}, path::{Path, PathBuf}};

use async_trait::async_trait;
use eyre::{eyre, Result};
use windows::{Win32::{Foundation::{DUPLICATE_SAME_ACCESS, DuplicateHandle, GENERIC_READ, GENERIC_WRITE, HANDLE, HANDLE_FLAG_INHERIT, SetHandleInformation, TRUE}, Security::SECURITY_ATTRIBUTES, Storage::FileSystem::{CreateFileW, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING}, System::{Console::{GetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE}, JobObjects::{AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation, SetInformationJobObject}, Threading::{CREATE_UNICODE_ENVIRONMENT, CreateProcessW, EXTENDED_STARTUPINFO_PRESENT, GetCurrentProcess, LPPROC_THREAD_ATTRIBUTE_LIST, PROCESS_CREATION_FLAGS, PROCESS_INFORMATION, STARTF_FORCEONFEEDBACK, STARTF_USESTDHANDLES, STARTUPINFOEXW, STARTUPINFOW}}}, core::Free};

use crate::{SandboxArg, SandboxExitStatus, SandboxStdio, backend::{Pipes, SandboxChildOp}};

pub(crate) mod appcontainer;
pub(crate) mod runas;

pub(crate) fn spawn(
    program: PathBuf,
    arguments: Vec<SandboxArg>,
    environment: BTreeMap<SandboxArg, SandboxArg>,
    stdin: SandboxStdio,
    stdout: SandboxStdio,
    stderr: SandboxStdio,
    working_directory: Option<PathBuf>,
    job_handle: HANDLE,
    null_device: HANDLE,
    attributes: Option<LPPROC_THREAD_ATTRIBUTE_LIST>
) -> Result<(Pipes, WindowsChild)> {
    let program = resolve_path(program)?;
    let working_directory = working_directory.map(resolve_path).transpose()?;
    let application_name = program.as_os_str().encode_wide()
        .chain([0])
        .collect::<Vec<_>>();
    dbg!(&arguments, join_windows_shell_arg(arguments.as_slice()));
    let mut command_line = join_windows_shell_arg(arguments.as_slice()).encode_wide()
        .chain([0])
        .collect::<Vec<_>>();
    let current_directory = working_directory.map(|dir| dir.as_os_str().encode_wide()
        .chain([0])
        .collect::<Vec<_>>());

    // Create env
    let mut env = Vec::new();
    if environment.is_empty() {
        env.push(0);
    }
    for (k, v) in environment {
        if k.as_os_str().as_encoded_bytes().contains(&b'\0') || v.as_os_str().as_encoded_bytes().contains(&b'\0') {
            return Err(eyre!("environment variable contained null byte"));
        }
        if k.as_os_str().as_encoded_bytes().contains(&b'=') || v.as_os_str().as_encoded_bytes().contains(&b'=') {
            return Err(eyre!("environment variable contained equals sign"));
        }
        env.extend(k.as_os_str().encode_wide());
        env.push('=' as u16);
        env.extend(v.as_os_str().encode_wide());
        env.push(0);
    }
    env.push(0);

    // Stdio
    let mut stdin_write = None;
    let mut stdout_read = None;
    let mut stderr_read = None;

    let mut handles_to_close = Vec::new();

    let mut stdin_read = None;
    let mut stdout_write = None;
    let mut stderr_write = None;

    match stdin {
        SandboxStdio::Null => {
            stdin_read = Some(null_device);
        },
        SandboxStdio::Inherit => {
            let handle = unsafe { GetStdHandle(STD_INPUT_HANDLE) }?;
            if !handle.is_invalid() {
                let handle = duplicate_handle_as_inheritable(handle)?;
                handles_to_close.push(to_owned_handle(handle)?);
                stdin_read = Some(handle);
            }
        },
        SandboxStdio::Pipe => {
            let (read, write) = std::io::pipe()?;
            let owned: OwnedHandle = read.into();

            stdin_write = Some(write);
            unsafe { SetHandleInformation(HANDLE(owned.as_raw_handle()), HANDLE_FLAG_INHERIT.0, HANDLE_FLAG_INHERIT)? };
            stdin_read = Some(HANDLE(owned.as_raw_handle()));
            handles_to_close.push(owned);
        }
    }
    match stdout {
        SandboxStdio::Pipe => {
            let (read, write) = std::io::pipe()?;
            let owned: OwnedHandle = write.into();

            stdout_read = Some(read);
            unsafe { SetHandleInformation(HANDLE(owned.as_raw_handle()), HANDLE_FLAG_INHERIT.0, HANDLE_FLAG_INHERIT)? };
            stdout_write = Some(HANDLE(owned.as_raw_handle()));
            handles_to_close.push(owned);
        },
        SandboxStdio::Null => {
            stdout_write = Some(null_device);
        },
        SandboxStdio::Inherit => {
            let handle = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) }?;
            if !handle.is_invalid() {
                let handle = duplicate_handle_as_inheritable(handle)?;
                handles_to_close.push(to_owned_handle(handle)?);
                stdout_write = Some(handle);
            }
        },
    }
    match stderr {
        SandboxStdio::Pipe => {
            let (read, write) = std::io::pipe()?;
            let owned: OwnedHandle = write.into();

            stderr_read = Some(read);
            unsafe { SetHandleInformation(HANDLE(owned.as_raw_handle()), HANDLE_FLAG_INHERIT.0, HANDLE_FLAG_INHERIT)? };
            stderr_write = Some(HANDLE(owned.as_raw_handle()));
            handles_to_close.push(owned);
        },
        SandboxStdio::Null => {
            stderr_write = Some(null_device);
        },
        SandboxStdio::Inherit => {
            let handle = unsafe { GetStdHandle(STD_ERROR_HANDLE) }?;
            if !handle.is_invalid() {
                let handle = duplicate_handle_as_inheritable(handle)?;
                handles_to_close.push(to_owned_handle(handle)?);
                stderr_write = Some(handle);
            }
        },
    }

    let mut process_creation_flags = PROCESS_CREATION_FLAGS::default();
    process_creation_flags |= CREATE_UNICODE_ENVIRONMENT;

    let mut si: STARTUPINFOW = Default::default();
    si.cb = size_of::<STARTUPINFOW>() as u32;

    if let Some(stdin_read) = stdin_read {
        si.dwFlags |= STARTF_USESTDHANDLES;
        si.hStdInput = stdin_read;
    }
    if let Some(stdout_write) = stdout_write {
        si.dwFlags |= STARTF_USESTDHANDLES;
        si.hStdOutput = stdout_write;
    }
    if let Some(stderr_write) = stderr_write {
        si.dwFlags |= STARTF_USESTDHANDLES;
        si.hStdError = stderr_write;
    }

    si.dwFlags |= STARTF_FORCEONFEEDBACK;

    let mut sip = &si as *const STARTUPINFOW;
    let si_ex;
    if let Some(attributes) = attributes {
        process_creation_flags |= EXTENDED_STARTUPINFO_PRESENT;
        si.cb = size_of::<STARTUPINFOEXW>() as u32;
        si_ex = STARTUPINFOEXW {
            StartupInfo: si,
            lpAttributeList: attributes,
        };
        sip = &si_ex as *const _ as *const STARTUPINFOW;
    }

    let mut pi: PROCESS_INFORMATION = Default::default();
    unsafe {
        CreateProcessW(
            windows::core::PCWSTR(application_name.as_ptr()),
            Some(windows::core::PWSTR(command_line.as_mut_ptr())),
            None,
            None,
            stdin_read.is_some() || stdout_write.is_some() || stderr_write.is_some(),
            process_creation_flags,
            Some(env.as_ptr() as *mut c_void),
            current_directory.as_ref().map(|dir| windows::core::PCWSTR(dir.as_ptr())).unwrap_or_default(),
            sip,
            &mut pi
        )?
    }

    unsafe { pi.hThread.free(); };

    if pi.hProcess.is_invalid() {
        return Err(eyre!("CreateProcessW returned invalid process handle"));
    }

    drop(handles_to_close);

    unsafe {
        _ = AssignProcessToJobObject(job_handle, pi.hProcess);
    }

    return Ok((
        Pipes {
            stdin: stdin_write,
            stdout: stdout_read,
            stderr: stderr_read,
        },
        WindowsChild {
            process_handle: pi.hProcess,
            exit_status: None,
        }
    ));
}

pub(crate) fn resolve_path(path: PathBuf) -> Result<PathBuf> {
        let Ok(path) = path.canonicalize() else {
            return Err(eyre!("executable file doesn't exist: {:?}", path));
        };

        debug_assert!(path.is_absolute());

        // Try to remove the \\?\ verbatim path prefix since it can break some applications
        let encoded_bytes = path.as_os_str().as_encoded_bytes();
        if let Some(rest) = encoded_bytes.strip_prefix(b"\\\\?\\") {
            return Ok(PathBuf::from(unsafe { OsStr::from_encoded_bytes_unchecked(&rest) }));
        }

        Ok(path)
    }

#[derive(Debug)]
pub(crate) struct WindowsChild {
    process_handle: HANDLE,
    exit_status: Option<SandboxExitStatus>,
}

// HANDLE can be sent across threads despite containing *mut c_void
unsafe impl Send for WindowsChild {}
unsafe impl Sync for WindowsChild {}

impl Drop for WindowsChild {
    fn drop(&mut self) {
        use windows::core::Free;
        unsafe {
            std::mem::take(&mut self.process_handle).free();
        }
    }
}

#[async_trait]
impl SandboxChildOp for WindowsChild {
    fn id(&self) -> Option<u32>  {
        if self.exit_status.is_some() {
            return None;
        }
        return Some(unsafe {
            windows::Win32::System::Threading::GetProcessId(self.process_handle)
        });
    }

    fn try_wait(&mut self) -> Result<Option<SandboxExitStatus>> {
        if let Some(exit_status) = self.exit_status {
            return Ok(Some(exit_status));
        }

        unsafe {
            let wait = windows::Win32::System::Threading::WaitForSingleObject(self.process_handle, 0);
            if wait == windows::Win32::Foundation::WAIT_FAILED {
                return Err(eyre!("wait failed"));
            } else if wait == windows::Win32::Foundation::WAIT_TIMEOUT {
                return Ok(None);
            }

            let mut code = 0;
            windows::Win32::System::Threading::GetExitCodeProcess(self.process_handle, &mut code)?;
            self.exit_status = Some(SandboxExitStatus { imp: WindowsSandboxExitStatus(code) });
            return Ok(self.exit_status);
        }
    }

    async fn wait(&mut self) -> Result<SandboxExitStatus> {
        if let Some(exit_status) = self.exit_status {
            return Ok(exit_status);
        }

        unsafe {
            let wait = windows::Win32::System::Threading::WaitForSingleObject(self.process_handle, windows::Win32::System::Threading::INFINITE);
            if wait == windows::Win32::Foundation::WAIT_FAILED {
                return Err(eyre!("wait failed"));
            }

            let mut code = 0;
            windows::Win32::System::Threading::GetExitCodeProcess(self.process_handle, &mut code)?;
            self.exit_status = Some(SandboxExitStatus { imp: WindowsSandboxExitStatus(code) });
            return Ok(SandboxExitStatus { imp: WindowsSandboxExitStatus(code) });
        }
    }

    async fn kill(&mut self) -> Result<()> {
        todo!()
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct WindowsSandboxExitStatus(pub(crate) u32);

#[cfg(windows)]
impl std::fmt::Display for WindowsSandboxExitStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 & 0x80000000 != 0 {
            f.write_fmt(format_args!("exitcode=0x{:#x}", self.0))
        } else {
            f.write_fmt(format_args!("exitcode={}", self.0))
        }
    }
}

impl WindowsSandboxExitStatus {
    pub fn success(&self) -> bool {
        self.0 == 0
    }

    pub fn code(&self) -> Option<i32> {
        Some(self.0 as i32)
    }
}

pub fn join_windows_shell_arg(args: &[SandboxArg]) -> OsString {
    let mut string = Vec::new();

    let mut first = true;
    for arg in args {
        let arg = arg.as_os_str();
        let mut backslashes = 0;

        if first {
            first = false;
        } else {
            string.push(b' ');
        }

        if arg.is_empty() {
            string.extend(b"\"\"");
            continue;
        }

        let arg_raw = arg.as_encoded_bytes();
        let quoted = arg_raw.contains(&b' ') || arg_raw.contains(&b'\t');
        if quoted {
            string.push(b'"');
        }

        for byte in arg_raw {
            if *byte == b'\\' {
                backslashes += 1;
            } else if *byte == b'"' {
                for _ in 0..backslashes*2 {
                    string.push(b'\\');
                }
                string.push(b'\\');
                string.push(b'"');
                backslashes = 0;
            } else {
                for _ in 0..backslashes {
                    string.push(b'\\');
                }
                backslashes = 0;
                string.push(*byte);
            }
        }

        if quoted {
            for _ in 0..backslashes*2 {
                string.push(b'\\');
            }
        } else {
            for _ in 0..backslashes {
                string.push(b'\\');
            }
        }

        if quoted {
            string.push(b'"');
        }
    }

    unsafe {
        OsString::from_encoded_bytes_unchecked(string)
    }
}

pub fn create_job_object() -> Result<HANDLE> {
    let job_handle = unsafe {
        CreateJobObjectW(
            None,
            windows::core::PCWSTR::default()
        )?
    };
    if job_handle.is_invalid() {
        return Err(eyre!("CreateJobObjectW returned invalid handle"));
    }
    let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
    info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    unsafe {
        SetInformationJobObject(
            job_handle,
            JobObjectExtendedLimitInformation,
            &info as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION as _,
            size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32
        )?
    }
    Ok(job_handle)
}

pub fn open_null_device() -> Result<HANDLE> {
    let sa = SECURITY_ATTRIBUTES {
        nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: std::ptr::null_mut(),
        bInheritHandle: TRUE,
    };
    let handle = unsafe {
        CreateFileW(
            windows::core::w!(r"\\.\NUL"),
            (GENERIC_READ | GENERIC_WRITE).0,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            Some(&sa),
            OPEN_EXISTING,
            Default::default(),
            None,
        )?
    };
    if handle.is_invalid() {
        return Err(eyre!("CreateFileW returned invalid handle"));
    }
    Ok(handle)
}

pub fn duplicate_handle_as_inheritable(handle: HANDLE) -> Result<HANDLE> {
    let mut duplicated = HANDLE::default();
    unsafe {
        let cur_proc = GetCurrentProcess();
        DuplicateHandle(
            cur_proc,
            handle,
            cur_proc,
            &mut duplicated,
            0,
            true,
            DUPLICATE_SAME_ACCESS,
        )?;
    }
    if duplicated.is_invalid() {
        return Err(eyre!("DuplicateHandle returned invalid handle"));
    }
    Ok(duplicated)
}

pub fn to_owned_handle(handle: HANDLE) -> Result<OwnedHandle> {
    let handle = unsafe { HandleOrInvalid::from_raw_handle(handle.0) };
    if let Ok(handle) = OwnedHandle::try_from(handle) {
        Ok(handle)
    } else {
        Err(eyre!("cannot convert invalid handle to owned handle"))
    }
}
