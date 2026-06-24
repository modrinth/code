use crate::api::Result;
use std::{
    os::windows::ffi::OsStrExt,
    path::{Path, PathBuf},
};
use windows::{
    Win32::{
        System::Com::{
            CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
            COINIT_DISABLE_OLE1DDE, CoCreateInstance, CoInitializeEx,
            CoUninitialize, IPersistFile,
        },
        UI::Shell::{IShellLinkW, ShellLink},
    },
    core::{Interface, PCWSTR},
};

pub(super) const SHORTCUT_EXTENSION: &str = "lnk";

pub(super) async fn create_shortcut(
    _profile_name: &str,
    launch_url: &str,
    output_path: &Path,
) -> Result<()> {
    let target_path = std::env::current_exe()?;
    let working_dir = target_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default();
    let output_path = output_path.to_path_buf();
    let launch_url = launch_url.to_string();

    tokio::task::spawn_blocking(move || {
        create_windows_shortcut(
            output_path,
            target_path,
            working_dir,
            launch_url,
        )
    })
    .await
    .map_err(|error| {
        std::io::Error::other(format!(
            "failed to join shortcut creation task: {error}"
        ))
    })??;

    Ok(())
}

fn create_windows_shortcut(
    output_path: PathBuf,
    target_path: PathBuf,
    working_dir: PathBuf,
    launch_url: String,
) -> std::io::Result<()> {
    let output_path = windows_wide_path(&output_path);
    let target_path = windows_wide_path(&target_path);
    let working_dir = windows_wide_path(&working_dir);
    let launch_url = windows_wide_string(&launch_url);

    unsafe {
        let init_result = CoInitializeEx(
            None,
            COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE,
        );
        windows_result(init_result.ok())?;
        let _com = WindowsComGuard;

        let shortcut: IShellLinkW = windows_result(CoCreateInstance(
            &ShellLink,
            None,
            CLSCTX_INPROC_SERVER,
        ))?;
        windows_result(shortcut.SetPath(windows_pcwstr(&target_path)))?;
        windows_result(shortcut.SetArguments(windows_pcwstr(&launch_url)))?;
        windows_result(
            shortcut.SetWorkingDirectory(windows_pcwstr(&working_dir)),
        )?;
        windows_result(
            shortcut.SetIconLocation(windows_pcwstr(&target_path), 0),
        )?;

        let persist_file: IPersistFile = windows_result(shortcut.cast())?;
        windows_result(persist_file.Save(windows_pcwstr(&output_path), true))?;
    }

    Ok(())
}

fn windows_result<T>(result: windows::core::Result<T>) -> std::io::Result<T> {
    result.map_err(std::io::Error::other)
}

struct WindowsComGuard;

impl Drop for WindowsComGuard {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}

fn windows_wide_path(path: &Path) -> Vec<u16> {
    path.as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

fn windows_wide_string(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn windows_pcwstr(value: &[u16]) -> PCWSTR {
    PCWSTR::from_raw(value.as_ptr())
}
