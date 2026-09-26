use std::{
    collections::BTreeMap,
    ffi::CString,
    io::{ErrorKind, PipeReader, PipeWriter},
    os::{
        fd::{AsRawFd, OwnedFd, RawFd},
        unix::ffi::OsStringExt,
    },
    path::PathBuf,
};

use async_trait::async_trait;
use eyre::Result;

use crate::{
    SandboxExitStatus, SandboxStdio,
    backend::{SandboxChildOp, Pipes},
    util::{RawStringVec, SandboxArg},
};

pub(crate) fn spawn(
    program: SandboxArg,
    arguments: Vec<SandboxArg>,
    environment: BTreeMap<SandboxArg, SandboxArg>,
    stdin: SandboxStdio,
    stdout: SandboxStdio,
    stderr: SandboxStdio,
    working_directory: Option<PathBuf>,
    pass_fds: Vec<OwnedFd>,
    dev_null: libc::c_int,
    #[cfg(target_os = "linux")] die_with_parent: bool,
) -> Result<(Pipes, UnixChild)> {
    let program = CString::new(program.into_os_string().into_vec())?;

    // Arguments
    let mut argv = RawStringVec::with_capacity(arguments.len() + 1);
    argv.push_c(program.clone()); // arg0 is program name
    for arg in arguments {
        argv.push_os(arg.into_os_string())?;
    }

    // Environment
    let mut env = RawStringVec::with_capacity(environment.len());
    for (k, v) in environment {
        let mut k = k.into_os_string();
        k.reserve_exact(v.as_os_str().len() + 2);
        k.push("=");
        k.push(v.as_os_str());

        env.push_os(k)?;
    }

    // Workdir
    let workdir = if let Some(working_directory) = working_directory {
        Some(CString::new(working_directory.into_os_string().into_vec())?)
    } else {
        None
    };

    // Stdio
    let mut stdin_write = None;
    let mut stdout_read = None;
    let mut stderr_read = None;

    #[expect(
        clippy::collection_is_never_read,
        reason = "keep fds alive until we drop them"
    )]
    let mut fds_to_drop: Vec<OwnedFd> = Vec::new();
    let mut stdin_read = None;
    let mut stdout_write = None;
    let mut stderr_write = None;

    match stdin {
        SandboxStdio::Null => {
            stdin_read = Some(dev_null);
        }
        SandboxStdio::Inherit => {}
        SandboxStdio::Pipe => {
            let (read, write) = std::io::pipe()?;
            stdin_write = Some(write);
            stdin_read = Some(read.as_raw_fd());
            fds_to_drop.push(read.into());
        }
    }
    match stdout {
        SandboxStdio::Pipe => {
            let (read, write) = std::io::pipe()?;
            stdout_read = Some(read);
            stdout_write = Some(write.as_raw_fd());
            fds_to_drop.push(write.into());
        }
        SandboxStdio::Null => {
            stdout_write = Some(dev_null);
        }
        SandboxStdio::Inherit => {}
    }
    match stderr {
        SandboxStdio::Pipe => {
            let (read, write) = std::io::pipe()?;
            stderr_read = Some(read);
            stderr_write = Some(write.as_raw_fd());
            fds_to_drop.push(write.into());
        }
        SandboxStdio::Null => {
            stderr_write = Some(dev_null);
        }
        SandboxStdio::Inherit => {}
    }

    argv.ensure_null_terminated();
    env.ensure_null_terminated();

    let pid = unsafe { cvt(libc::fork())? };
    if pid == 0 {
        _ = exec(
            program.as_ptr(),
            argv.into_null_terminated_ptr() as *const *const libc::c_char,
            env.into_null_terminated_ptr() as *const *const libc::c_char,
            stdin_read,
            stdout_write,
            stderr_write,
            workdir.as_ref().map(|dir| dir.as_ptr()),
            &pass_fds,
            #[cfg(target_os = "linux")]
            die_with_parent,
        );
        unsafe { libc::_exit(1) }
    }

    Ok((
        Pipes {
            stdin: stdin_write,
            stdout: stdout_read,
            stderr: stderr_read,
        },
        UnixChild {
            pid,
            exit_status: None,
        },
    ))
}

fn exec(
    program: *const libc::c_char,
    argv: *const *const libc::c_char,
    env: *const *const libc::c_char,
    stdin: Option<RawFd>,
    stdout: Option<RawFd>,
    stderr: Option<RawFd>,
    workdir: Option<*const libc::c_char>,
    pass_fds: &[OwnedFd],
    #[cfg(target_os = "linux")] die_with_parent: bool,
) -> std::io::Result<()> {
    unsafe {
        *environ() = env;

        if let Some(mut fd) = stdin {
            if fd > 0 && fd <= libc::STDERR_FILENO {
                fd = cvt_r(|| libc::dup(fd))?;
            }
            cvt_r(|| libc::dup2(fd, libc::STDIN_FILENO))?;
        }
        if let Some(mut fd) = stdout {
            if fd > 0 && fd <= libc::STDERR_FILENO {
                fd = cvt_r(|| libc::dup(fd))?;
            }
            cvt_r(|| libc::dup2(fd, libc::STDOUT_FILENO))?;
        }
        if let Some(mut fd) = stderr {
            if fd > 0 && fd <= libc::STDERR_FILENO {
                fd = cvt_r(|| libc::dup(fd))?;
            }
            cvt_r(|| libc::dup2(fd, libc::STDERR_FILENO))?;
        }

        // Set working directory
        if let Some(workdir) = workdir {
            cvt_r(|| libc::chdir(workdir))?;
        }

        // Unset "close on exec" flag so fds are kept after the exec
        for fd in pass_fds {
            cvt_r(|| libc::ioctl(fd.as_raw_fd(), libc::FIONCLEX))?;
        }

        // Set PR_SET_PDEATHSIG to SIGKILL
        // This will kill the process when the parent thread or process dies
        #[cfg(target_os = "linux")]
        if die_with_parent {
            cvt_r(|| {
                libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL, 0, 0, 0)
            })?;
        }

        cvt(libc::execvp(program, argv))?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct UnixChild {
    pid: libc::pid_t,
    exit_status: Option<SandboxExitStatus>,
}

#[async_trait]
impl SandboxChildOp for UnixChild {
    fn id(&self) -> Option<u32> {
        if self.exit_status.is_some() {
            return None;
        }
        Some(self.pid as u32)
    }

    fn try_wait(&mut self) -> Result<Option<SandboxExitStatus>> {
        // Need to remember the exit status due to waitpid at-most-once semantics
        if let Some(exit_status) = self.exit_status {
            return Ok(Some(exit_status));
        }

        let mut status = 0 as libc::c_int;
        let pid = cvt_r(|| unsafe {
            libc::waitpid(self.pid, &raw mut status, libc::WNOHANG)
        })?;

        if pid == 0 {
            Ok(None)
        } else {
            self.exit_status = Some(SandboxExitStatus {
                imp: UnixSandboxExitStatus(status),
            });
            Ok(self.exit_status)
        }
    }

    // TODO: this must be made cancel-safe - see how tokio::process does it?
    async fn wait(&mut self) -> Result<SandboxExitStatus> {
        // Need to remember the exit status due to waitpid at-most-once semantics
        if let Some(exit_status) = self.exit_status {
            return Ok(exit_status);
        }

        let wait_for_pid = self.pid;
        let status = tokio::task::spawn_blocking(move || {
            let mut status = 0 as libc::c_int;
            cvt_r(|| unsafe {
                libc::waitpid(wait_for_pid, &raw mut status, 0)
            })?;
            eyre::Ok(status)
        })
        .await??;

        self.exit_status = Some(SandboxExitStatus {
            imp: UnixSandboxExitStatus(status),
        });
        return Ok(SandboxExitStatus {
            imp: UnixSandboxExitStatus(status),
        });
    }

    async fn kill(&mut self) -> Result<()> {
        let kill_pid = self.pid;
        tokio::task::spawn_blocking(move || {
            cvt_r(|| unsafe { libc::kill(kill_pid, libc::SIGKILL) })
        })
        .await??;
        Ok(())
    }
}

#[derive(Default, Clone, Copy)]
pub(crate) struct UnixSandboxExitStatus(pub(crate) libc::c_int);

impl UnixSandboxExitStatus {
    pub(crate) fn success(&self) -> bool {
        self.code() == Some(0)
    }

    pub(crate) fn code(&self) -> Option<i32> {
        if libc::WIFEXITED(self.0) {
            Some(libc::WEXITSTATUS(self.0))
        } else {
            None
        }
    }
}

impl std::fmt::Display for UnixSandboxExitStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // std::process::ExitStatus
        if libc::WIFEXITED(self.0) {
            f.write_fmt(format_args!("exitcode={}", libc::WEXITSTATUS(self.0)))
        } else if libc::WIFSIGNALED(self.0) {
            f.write_fmt(format_args!("signal={}", libc::WTERMSIG(self.0)))
        } else {
            f.write_fmt(format_args!("unknownwait=0x{:#x}", self.0))
        }
    }
}

impl std::fmt::Debug for UnixSandboxExitStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("PandoraExitStatus");
        debug.field("raw", &self.0);
        if libc::WIFEXITED(self.0) {
            debug.field("exitcode", &libc::WEXITSTATUS(self.0));
        }
        if libc::WIFSIGNALED(self.0) {
            debug.field("signal", &libc::WTERMSIG(self.0));
        }
        debug.finish()
    }
}

pub fn open_dev_null() -> eyre::Result<libc::c_int> {
    Ok(unsafe { cvt(libc::open(c"/dev/null".as_ptr(), libc::O_RDWR))? })
}

#[doc(hidden)]
pub trait IsMinusOne {
    fn is_minus_one(&self) -> bool;
}

macro_rules! impl_is_minus_one {
    ($($t:ident)*) => ($(impl IsMinusOne for $t {
        fn is_minus_one(&self) -> bool {
            *self == -1
        }
    })*)
}

impl_is_minus_one! { i8 i16 i32 i64 isize }

/// Converts native return values to Result using the *-1 means error is in `errno`*  convention.
/// Non-error values are `Ok`-wrapped.
pub fn cvt<T: IsMinusOne>(t: T) -> std::io::Result<T> {
    if t.is_minus_one() {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(t)
    }
}

/// `-1` → look at `errno` → retry on `EINTR`. Otherwise `Ok()`-wrap the closure return value.
pub fn cvt_r<T, F>(mut f: F) -> std::io::Result<T>
where
    T: IsMinusOne,
    F: FnMut() -> T,
{
    loop {
        match cvt(f()) {
            Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
            other => return other,
        }
    }
}

#[cfg(target_os = "macos")]
pub unsafe fn environ() -> *mut *const *const libc::c_char {
    unsafe { libc::_NSGetEnviron() as *mut *const *const libc::c_char }
}

// Use the `environ` static which is part of POSIX.
#[cfg(not(target_os = "macos"))]
pub unsafe fn environ() -> *mut *const *const libc::c_char {
    unsafe extern "C" {
        static mut environ: *const *const libc::c_char;
    }
    &raw mut environ
}
