use std::path::PathBuf;

pub async fn open_dir(path: PathBuf) {
    // We have to be very careful here, using something like open::that(path) will work,
    // however it opens us up to a TOCTOU exploit where the folder's path is replaced by an executable file.
    // Therefore, we use platform specific operations that are designed to specifically open a folder
    #[cfg(target_os = "macos")]
    {
        // Finder can take a long time to respond
        let result =
            tokio::task::spawn_blocking(move || reveal_in_finder(&path)).await;
        if let Err(err) = result {
            tracing::error!("SandboxAPI failed to reveal directory: {err}");
        }
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

#[cfg(target_os = "macos")]
fn reveal_in_finder(directory: &std::path::Path) {
    use std::{fs::File, io::ErrorKind};

    // Unfortunately selectFile_inFileViewerRootedAtPath will execute
    // a .app bundle instead of opening it in Finder.
    // Even explicitly passing Finder as the application doesn't work,
    // Finder just executes the .app
    //
    // Instead, we use activateFileViewerSelectingURLs which has the
    // unfortunate side effect of requiring a child to select.
    // We use the hidden .DS_Store as the child, creating it if needed, so that
    // empty folders work and nothing visible is selected. If we can't create
    // it, we fall back to opening the parent with the folder selected

    let ds_store = directory.join(".DS_Store");
    // This must be exclusive to avoid truncating the target of a symlink
    let exists = match File::create_new(&ds_store) {
        Ok(_) => true,
        Err(err) => err.kind() == ErrorKind::AlreadyExists,
    };

    let path = if exists { &ds_store } else { directory };
    let Some(url) = objc2_foundation::NSURL::from_file_path(path) else {
        return;
    };
    let workspace = objc2_app_kit::NSWorkspace::sharedWorkspace();
    workspace.activateFileViewerSelectingURLs(
        &objc2_foundation::NSArray::from_slice(&[&*url]),
    );

    // Revealing leaves the window behind the game, so we also need to bring
    // Finder to the front
    let finder = objc2_foundation::NSURL::fileURLWithPath_isDirectory(
        objc2_foundation::ns_string!("/System/Library/CoreServices/Finder.app"),
        true,
    );
    let configuration =
        objc2_app_kit::NSWorkspaceOpenConfiguration::configuration();
    configuration.setPromptsUserIfNeeded(false);
    workspace.openApplicationAtURL_configuration_completionHandler(
        &finder,
        &configuration,
        None,
    );
}
