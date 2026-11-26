#[cfg(target_os = "linux")]
use std::os::fd::RawFd;
use std::{
    collections::BTreeMap,
    ffi::CString,
    io::ErrorKind,
    os::{
        fd::{AsRawFd, OwnedFd},
        unix::ffi::OsStringExt,
    },
    path::PathBuf,
};

use anyhow::{Context, Result};
use async_trait::async_trait;
use futures::FutureExt;

use crate::{
    SandboxExitStatus, SandboxStdio,
    backend::{Pipes, SandboxChildOp},
    util::{RawStringVec, SandboxArg},
};

pub fn spawn(
    program: SandboxArg,
    arguments: Vec<SandboxArg>,
    environment: BTreeMap<SandboxArg, SandboxArg>,
    stdin: SandboxStdio,
    stdout: SandboxStdio,
    stderr: SandboxStdio,
    working_directory: Option<PathBuf>,
    pass_fds: Vec<OwnedFd>,
    dev_null: libc::c_int,
    double_fork: bool,
    #[cfg(target_os = "linux")] die_with_parent: bool,
    #[cfg(target_os = "macos")] sandbox_profile: CString,
    #[cfg(target_os = "macos")] mut sandbox_params: RawStringVec,
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
    #[cfg(target_os = "macos")]
    sandbox_params.ensure_null_terminated();

    // Built before forking so that the child doesn't need to allocate
    #[cfg(target_os = "macos")]
    let spawn_config = macos_posix_spawn::PosixSpawnConfig::new(
        stdin_read,
        stdout_write,
        stderr_write,
        &pass_fds,
    )
    .context("preparing posix_spawn configuration")?;

    // Read before forking so that the child doesn't need to allocate
    #[cfg(target_os = "linux")]
    let kernel_fd_limit = read_kernel_fd_limit();

    let child_pid = unsafe { cvt(libc::fork())? };
    if child_pid == 0 {
        if double_fork {
            if unsafe { libc::setsid() } == -1 {
                unsafe { libc::_exit(1) };
            }

            match unsafe { libc::fork() } {
                -1 => unsafe { libc::_exit(1) },
                0 => {}
                _ => unsafe { libc::_exit(0) },
            }
        }

        #[cfg(target_os = "linux")]
        {
            _ = exec(
                program.as_ptr(),
                argv.into_null_terminated_ptr() as *const *const libc::c_char,
                env.into_null_terminated_ptr() as *const *const libc::c_char,
                stdin_read,
                stdout_write,
                stderr_write,
                workdir.as_ref().map(|dir| dir.as_ptr()),
                &pass_fds,
                die_with_parent,
                kernel_fd_limit,
            );
        }
        #[cfg(target_os = "macos")]
        {
            _ = exec(
                program.as_ptr(),
                argv.into_null_terminated_ptr(),
                env.into_null_terminated_ptr(),
                workdir.as_ref().map(|dir| dir.as_ptr()),
                &spawn_config,
                sandbox_profile,
                sandbox_params,
            );
        }
        unsafe { libc::_exit(1) }
    }

    if double_fork {
        let mut status = 0 as libc::c_int;
        cvt_r(|| unsafe { libc::waitpid(child_pid, &raw mut status, 0) })?;
        anyhow::ensure!(
            libc::WIFEXITED(status) && libc::WEXITSTATUS(status) == 0,
            "double fork failed in intermediate child"
        );

        return Ok((
            Pipes {
                stdin: stdin_write,
                stdout: stdout_read,
                stderr: stderr_read,
            },
            UnixChild {
                pid: 0,
                exit_status: Some(SandboxExitStatus {
                    imp: UnixSandboxExitStatus(0),
                }),
                wait_task: None,
            },
        ));
    }

    Ok((
        Pipes {
            stdin: stdin_write,
            stdout: stdout_read,
            stderr: stderr_read,
        },
        UnixChild {
            pid: child_pid,
            exit_status: None,
            wait_task: None,
        },
    ))
}

#[cfg(target_os = "linux")]
fn exec(
    program: *const libc::c_char,
    argv: *const *const libc::c_char,
    env: *const *const libc::c_char,
    stdin: Option<RawFd>,
    stdout: Option<RawFd>,
    stderr: Option<RawFd>,
    workdir: Option<*const libc::c_char>,
    pass_fds: &[OwnedFd],
    die_with_parent: bool,
    kernel_fd_limit: libc::rlim_t,
) -> anyhow::Result<()> {
    // `pass_fds` are unmarked again below
    mark_inherited_fds_cloexec(kernel_fd_limit);

    if let Some(mut fd) = stdin {
        if fd > 0 && fd <= libc::STDERR_FILENO {
            fd = cvt_r(|| unsafe { libc::dup(fd) })
                .context("duplicating stdin fd")?;
        }
        cvt_r(|| unsafe { libc::dup2(fd, libc::STDIN_FILENO) })
            .context("setting stdin fd")?;
    }
    if let Some(mut fd) = stdout {
        if fd > 0 && fd <= libc::STDERR_FILENO {
            fd = cvt_r(|| unsafe { libc::dup(fd) })
                .context("duplicating stdout fd")?;
        }
        cvt_r(|| unsafe { libc::dup2(fd, libc::STDOUT_FILENO) })
            .context("setting stdout fd")?;
    }
    if let Some(mut fd) = stderr {
        if fd > 0 && fd <= libc::STDERR_FILENO {
            fd = cvt_r(|| unsafe { libc::dup(fd) })
                .context("duplicating stderr fd")?;
        }
        cvt_r(|| unsafe { libc::dup2(fd, libc::STDERR_FILENO) })
            .context("setting stderr fd")?;
    }

    // Set working directory
    if let Some(workdir) = workdir {
        cvt_r(|| unsafe { libc::chdir(workdir) })
            .context("setting working directory")?;
    }

    // Unset "close on exec" flag so fds are kept after the exec
    for fd in pass_fds {
        cvt_r(|| unsafe { libc::ioctl(fd.as_raw_fd(), libc::FIONCLEX) })
            .context("unsetting close on exec")?;
    }

    // Set PR_SET_PDEATHSIG to SIGKILL
    // This will kill the process when the parent thread or process dies
    if die_with_parent {
        cvt_r(|| unsafe {
            libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL, 0, 0, 0)
        })
        .context("setting PR_SET_PDEATHSIG")?;
    }

    cvt(unsafe { libc::execve(program, argv, env) })?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn read_kernel_fd_limit() -> libc::rlim_t {
    std::fs::read_to_string("/proc/sys/fs/nr_open")
        .ok()
        .and_then(|nr_open| nr_open.trim().parse().ok())
        .unwrap_or(1 << 20)
}

/// Try to mark all inherited fds as cloexec. We can do this in 1 of 3 ways:
/// 1. libc::close_range with CLOSE_RANGE_CLOEXEC (Linux 5.11+)
/// 2. Reading /proc/self/fd (May fail)
/// 3. Looping over every fd up to the limit
#[cfg(target_os = "linux")]
fn mark_inherited_fds_cloexec(kernel_fd_limit: libc::rlim_t) {
    let result = unsafe {
        libc::close_range(
            3,
            libc::c_uint::MAX,
            libc::CLOSE_RANGE_CLOEXEC as libc::c_int,
        )
    };
    if result == 0 || mark_proc_self_fds_cloexec() {
        return;
    }

    let mut limit = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    let max_fd =
        if unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, &raw mut limit) } == 0
        {
            limit.rlim_cur.max(limit.rlim_max)
        } else {
            kernel_fd_limit
        };

    // Clamp between 1024 and the kernel's fd limit
    let max_fd = max_fd.min(kernel_fd_limit).max(1024);
    for fd in 3..max_fd as libc::c_int {
        unsafe { libc::ioctl(fd, libc::FIOCLEX) };
    }
}

#[cfg(target_os = "linux")]
fn mark_proc_self_fds_cloexec() -> bool {
    // Byte offsets into `struct linux_dirent64`
    // The struct is variable-length, so we can't transmute into it
    const RECORD_LEN_OFFSET: usize = 16;
    const NAME_OFFSET: usize = 19;

    #[repr(C, align(8))]
    struct AlignedBuffer([u8; 4096]);

    let dir_fd = unsafe {
        libc::open(
            c"/proc/self/fd".as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC,
        )
    };
    if dir_fd == -1 {
        return false;
    }
    let dir_fd =
        unsafe { <OwnedFd as std::os::fd::FromRawFd>::from_raw_fd(dir_fd) };

    let mut buffer = AlignedBuffer([0; 4096]);
    loop {
        let read = unsafe {
            libc::syscall(
                libc::SYS_getdents64,
                dir_fd.as_raw_fd(),
                buffer.0.as_mut_ptr(),
                buffer.0.len(),
            )
        };
        if read == 0 {
            return true;
        } else if read < 0 {
            return false;
        }

        let entries = &buffer.0[..read as usize];
        let mut offset = 0;
        while offset < entries.len() {
            let Some(record) = entries.get(offset..) else {
                return false;
            };
            let Some(&[low, high]) =
                record.get(RECORD_LEN_OFFSET..RECORD_LEN_OFFSET + 2)
            else {
                return false;
            };
            let record_len = u16::from_ne_bytes([low, high]) as usize;
            let Some(name) = record.get(NAME_OFFSET..record_len) else {
                return false;
            };

            if let Some(fd) = parse_fd(name)
                && fd >= 3
                && fd != dir_fd.as_raw_fd()
            {
                unsafe { libc::ioctl(fd, libc::FIOCLEX) };
            }

            offset += record_len;
        }
    }
}

#[cfg(target_os = "linux")]
fn parse_fd(name: &[u8]) -> Option<libc::c_int> {
    let mut fd: libc::c_int = 0;
    for &byte in name.iter().take_while(|&&byte| byte != 0) {
        if !byte.is_ascii_digit() {
            return None;
        }
        fd = fd.checked_mul(10)?;
        fd = fd.checked_add((byte - b'0') as libc::c_int)?;
    }
    Some(fd)
}

#[cfg(target_os = "macos")]
fn exec(
    program: *const libc::c_char,
    argv: *const *mut libc::c_char,
    env: *const *mut libc::c_char,
    workdir: Option<*const libc::c_char>,
    spawn_config: &macos_posix_spawn::PosixSpawnConfig,
    sandbox_profile: CString,
    sandbox_params: RawStringVec,
) -> anyhow::Result<()> {
    if let Some(workdir) = workdir {
        cvt_r(|| unsafe { libc::chdir(workdir) })
            .context("setting working directory")?;
    }

    let mut error_buf = std::ptr::null_mut();
    let sandbox_profile = sandbox_profile.as_ptr();
    let sandbox_params = sandbox_params.into_null_terminated_ptr().cast();
    let status_code = unsafe {
        sandbox_init_with_parameters(
            sandbox_profile,
            0,
            sandbox_params,
            &raw mut error_buf,
        )
    };

    if !error_buf.is_null() {
        let error = unsafe { std::ffi::CStr::from_ptr(error_buf) };
        let error = error.to_string_lossy().to_string();
        unsafe { sandbox_free_error(error_buf) };
        return Err(anyhow::anyhow!(error).context("setting up sandbox"));
    }

    anyhow::ensure!(status_code == 0, "sandbox setup failed");

    Err(anyhow::anyhow!(spawn_config.exec(program, argv, env))
        .context("executing program"))
}

#[cfg(target_os = "macos")]
mod macos_posix_spawn {
    use std::os::fd::{AsRawFd, OwnedFd, RawFd};

    use super::cvt_r;

    pub struct PosixSpawnConfig {
        attributes: libc::posix_spawnattr_t,
        file_actions: libc::posix_spawn_file_actions_t,
        dup_sources: [Option<RawFd>; 3],
    }

    impl PosixSpawnConfig {
        pub fn new(
            stdin: Option<RawFd>,
            stdout: Option<RawFd>,
            stderr: Option<RawFd>,
            pass_fds: &[OwnedFd],
        ) -> std::io::Result<Self> {
            let mut config = Self {
                attributes: std::ptr::null_mut(),
                file_actions: std::ptr::null_mut(),
                dup_sources: [None; 3],
            };

            unsafe {
                cvt_errno(libc::posix_spawnattr_init(
                    &raw mut config.attributes,
                ))?;
                cvt_errno(libc::posix_spawnattr_setflags(
                    &raw mut config.attributes,
                    (libc::POSIX_SPAWN_SETEXEC
                        | libc::POSIX_SPAWN_CLOEXEC_DEFAULT)
                        as libc::c_short,
                ))?;
                cvt_errno(libc::posix_spawn_file_actions_init(
                    &raw mut config.file_actions,
                ))?;
            }

            config.add_stdio(stdin, libc::STDIN_FILENO)?;
            config.add_stdio(stdout, libc::STDOUT_FILENO)?;
            config.add_stdio(stderr, libc::STDERR_FILENO)?;

            for fd in pass_fds {
                config.add_inherit(fd.as_raw_fd())?;
            }

            Ok(config)
        }

        pub fn exec(
            &self,
            program: *const libc::c_char,
            argv: *const *mut libc::c_char,
            env: *const *mut libc::c_char,
        ) -> std::io::Error {
            // We need to unset cloexec on the source of the dup2, otherwise
            // it'll fail. It'll be closed by POSIX_SPAWN_CLOEXEC_DEFAULT later anyways
            if let Err(error) = self.clear_cloexec_on_dup_sources() {
                return error;
            }

            let error = unsafe {
                libc::posix_spawnp(
                    std::ptr::null_mut(),
                    program,
                    &raw const self.file_actions,
                    &raw const self.attributes,
                    argv,
                    env,
                )
            };
            std::io::Error::from_raw_os_error(error)
        }

        fn add_stdio(
            &mut self,
            source: Option<RawFd>,
            target: RawFd,
        ) -> std::io::Result<()> {
            if let Some(source) = source
                && source != target
            {
                self.dup_sources[target as usize] = Some(source);
                cvt_errno(unsafe {
                    libc::posix_spawn_file_actions_adddup2(
                        &raw mut self.file_actions,
                        source,
                        target,
                    )
                })
            } else {
                self.add_inherit(target)
            }
        }

        fn clear_cloexec_on_dup_sources(&self) -> std::io::Result<()> {
            for source in self.dup_sources.into_iter().flatten() {
                cvt_r(|| unsafe { libc::ioctl(source, libc::FIONCLEX) })?;
            }
            Ok(())
        }

        fn add_inherit(&mut self, fd: RawFd) -> std::io::Result<()> {
            if unsafe { libc::fcntl(fd, libc::F_GETFD) } == -1 {
                // fd is closed, avoid trying to inherit it since
                // otherwise the spawn will fail
                return Ok(());
            }
            cvt_errno(unsafe {
                posix_spawn_file_actions_addinherit_np(
                    &raw mut self.file_actions,
                    fd,
                )
            })
        }
    }

    impl Drop for PosixSpawnConfig {
        fn drop(&mut self) {
            unsafe {
                if !self.attributes.is_null() {
                    libc::posix_spawnattr_destroy(&raw mut self.attributes);
                }
                if !self.file_actions.is_null() {
                    libc::posix_spawn_file_actions_destroy(
                        &raw mut self.file_actions,
                    );
                }
            }
        }
    }

    fn cvt_errno(error: libc::c_int) -> std::io::Result<()> {
        if error == 0 {
            Ok(())
        } else {
            Err(std::io::Error::from_raw_os_error(error))
        }
    }

    unsafe extern "C" {
        fn posix_spawn_file_actions_addinherit_np(
            file_actions: *mut libc::posix_spawn_file_actions_t,
            filedes: libc::c_int,
        ) -> libc::c_int;
    }
}

#[derive(Debug)]
pub struct UnixChild {
    pid: libc::pid_t,
    exit_status: Option<SandboxExitStatus>,
    wait_task: Option<tokio::task::JoinHandle<Result<libc::c_int>>>,
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

        if let Some(task) = self.wait_task.as_mut() {
            let Some(result) = task.now_or_never() else {
                return Ok(None);
            };
            self.wait_task = None;
            self.exit_status = Some(SandboxExitStatus {
                imp: UnixSandboxExitStatus(result??),
            });
            return Ok(self.exit_status);
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

    async fn wait(&mut self) -> Result<SandboxExitStatus> {
        if let Some(exit_status) = self.exit_status {
            return Ok(exit_status);
        }

        let wait_for_pid = self.pid;
        let task = self.wait_task.get_or_insert_with(|| {
            tokio::task::spawn_blocking(move || {
                let mut status = 0 as libc::c_int;
                cvt_r(|| unsafe {
                    libc::waitpid(wait_for_pid, &raw mut status, 0)
                })?;
                anyhow::Ok(status)
            })
        });
        let result = task.await;
        self.wait_task = None;
        let exit_status = SandboxExitStatus {
            imp: UnixSandboxExitStatus(result??),
        };
        self.exit_status = Some(exit_status);
        Ok(exit_status)
    }

    async fn kill(&mut self) -> Result<()> {
        if let Some(task) = self.wait_task.as_mut()
            && let Some(result) = task.now_or_never()
        {
            self.wait_task = None;
            self.exit_status = Some(SandboxExitStatus {
                imp: UnixSandboxExitStatus(result??),
            });
        }

        if self.exit_status.is_some() {
            return Ok(());
        }

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
        let mut debug = f.debug_struct("UnixSandboxExitStatus");
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

pub fn open_dev_null() -> anyhow::Result<libc::c_int> {
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
unsafe extern "C" {
    fn sandbox_init_with_parameters(
        profile: *const libc::c_char,
        flags: u64,
        parameters: *const *const libc::c_char,
        errorbuf: *mut *mut libc::c_char,
    ) -> libc::c_int;

    fn sandbox_free_error(errorbuf: *mut libc::c_char);
}
