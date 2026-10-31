use std::{
    ffi::CStr,
    os::fd::{AsRawFd, FromRawFd, OwnedFd},
};

use crate::backend::unix::cvt;

use std::{
	env,
	ffi::OsStr,
	path::{Path, PathBuf},
};

use eyre::{ContextCompat, Result, eyre};
use tokio::fs;

pub async fn find_command(
    command: &(impl AsRef<OsStr> + ?Sized),
) -> Result<PathBuf> {
    find_command_(command.as_ref()).await
}

async fn find_command_(command: &OsStr) -> Result<PathBuf> {
    let path = env::var_os("PATH").wrap_err("missing `PATH`")?;
    for mut path in env::split_paths(&path) {
        if !path.is_absolute() {
            continue;
        }
        path.push(command);
        if fs::try_exists(&path).await.unwrap_or(false) {
            return Ok(path);
        }
    }

    Err(eyre!("command not found in `PATH`"))
}

pub struct WritableMemoryFile {
    fd: OwnedFd,
}

impl WritableMemoryFile {
    pub fn open(name: &CStr) -> eyre::Result<Self> {
        let fd = unsafe {
            OwnedFd::from_raw_fd(cvt(libc::memfd_create(
                name.as_ptr(),
                libc::MFD_CLOEXEC | libc::MFD_ALLOW_SEALING,
            ))?)
        };
        Ok(Self { fd })
    }

    pub fn write_filter(
        mut self,
        filter: libseccomp::ScmpFilterContext,
    ) -> eyre::Result<OwnedFd> {
        filter.export_bpf(&self.fd)?;
        self.reset_and_seal()?;
        Ok(self.fd)
    }

    pub fn write(mut self, data: &CStr) -> eyre::Result<OwnedFd> {
        unsafe {
            let fd = self.fd.as_raw_fd();
            cvt(libc::write(
                fd,
                data.as_ptr().cast(),
                data.count_bytes() as libc::size_t,
            ))?;
            self.reset_and_seal()?;
        }
        Ok(self.fd)
    }

    fn reset_and_seal(&mut self) -> eyre::Result<()> {
        unsafe {
            let fd = self.fd.as_raw_fd();
            libc::lseek(fd, 0, libc::SEEK_SET);
            cvt(libc::fcntl(
                fd,
                libc::F_ADD_SEALS,
                libc::F_SEAL_SEAL
                    | libc::F_SEAL_SHRINK
                    | libc::F_SEAL_GROW
                    | libc::F_SEAL_WRITE,
            ))?;
        }
        Ok(())
    }
}

pub fn open_pipe() -> eyre::Result<(OwnedFd, OwnedFd)> {
    let mut fds = [0, 0];
    unsafe {
        super::unix::cvt(libc::pipe2(&raw mut fds as *mut _, libc::O_CLOEXEC))?;
        Ok((OwnedFd::from_raw_fd(fds[0]), OwnedFd::from_raw_fd(fds[1])))
    }
}

pub async fn ensure_valid_command(command: &crate::SandboxCommand) -> Result<()> {
	for path in command.ensure_dirs_exist.iter()
		.chain(&command.read_only_paths)
		.chain(&command.read_write_paths)
		.chain(std::iter::once(&command.executable))
		.chain(command.working_directory.iter())
	{
		ensure_exposable_path(path).await?;
	}
	Ok(())
}

async fn ensure_exposable_path(path: &Path) -> Result<()> {
	let check = |path: &Path| {
		eyre::ensure!(
			!path.starts_with("/tmp"),
			"cannot expose {}: Flatpak's private /tmp cannot be exposed through the host portal; use a persistent or cache directory instead",
			path.display()
		);
		Ok(())
	};
	check(path)?;
	for ancestor in path.ancestors() {
		match tokio::fs::canonicalize(ancestor).await {
			Ok(canonical) => return check(&canonical),
			Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
			Err(_) => break,
		}
	}
	Ok(())
}
