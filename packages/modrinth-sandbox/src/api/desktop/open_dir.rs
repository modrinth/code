use std::path::PathBuf;

pub async fn open_dir(path: PathBuf) {
    // We have to be very careful here, using something like open::that(path) will work,
    // however it opens us up to a TOCTOU exploit where the folder's path is replaced by an executable file.
    // Therefore, we use platform specific operations that are designed to specifically open a folder
    #[cfg(target_os = "macos")]
    {
        let workspace = objc2_app_kit::NSWorkspace::sharedWorkspace();
        let path =
            objc2_foundation::NSString::from_str(&path.to_string_lossy());
        workspace.selectFile_inFileViewerRootedAtPath(None, &path);
    }
    #[cfg(target_os = "windows")]
    {
        // ShellExecuteExW requires COM to be initialized, which the spawn thread has done
        let result = crate::backend::run_on_spawn_thread(move || {
            use std::os::windows::ffi::OsStrExt;
            use windows::Win32::UI::Shell::SHELLEXECUTEINFOW;

            let folder = path
                .as_os_str()
                .encode_wide()
                .chain([0])
                .collect::<Vec<_>>();

            let mut sei: SHELLEXECUTEINFOW = SHELLEXECUTEINFOW::default();
            sei.cbSize = std::mem::size_of::<SHELLEXECUTEINFOW>() as _;
            sei.fMask = windows::Win32::UI::Shell::SEE_MASK_DEFAULT;
            sei.lpVerb = windows::core::w!("explore");
            sei.lpFile = windows::core::PCWSTR(folder.as_ptr());
            sei.nShow =
                windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL.0;

            unsafe { windows::Win32::UI::Shell::ShellExecuteExW(&mut sei) }
        })
        .await;

        match result {
            Ok(Ok(())) => {}
            Ok(Err(err)) => tracing::error!(
                "SandboxAPI failed to ShellExecuteExW with explore verb: {err}"
            ),
            Err(err) => tracing::error!(
                "SandboxAPI failed to run ShellExecuteExW on spawn thread: {err}"
            ),
        }
    }
    #[cfg(target_os = "linux")]
    {
        let Ok(directory) = std::fs::File::open(path) else {
            return;
        };
        let result = ashpd::desktop::open_uri::OpenDirectoryRequest::default()
            .send(&std::os::fd::AsFd::as_fd(&directory))
            .await;
        if let Err(err) = result {
            tracing::error!("SandboxAPI failed to send OpenDirectory: {err}");
        }
    }
}
