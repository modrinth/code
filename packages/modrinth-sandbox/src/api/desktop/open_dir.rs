use std::path::PathBuf;

pub async fn open_dir(path: PathBuf) {
    // We have to be very careful here, using something like open::that(path) will work,
    // however it opens us up to a TOCTOU exploit where the folder's path is replaced by an executable file.
    // Therefore, we use platform specific operations that are designed to specifically open a folder
    #[cfg(target_os = "macos")]
    {
        let workspace = objc2_app_kit::NSWorkspace::sharedWorkspace();
        let path = objc2_foundation::NSString::from_str(&path.to_string_lossy());
        workspace.selectFile_inFileViewerRootedAtPath(None, &path);
    }
    #[cfg(target_os = "windows")]
    {
        let mut command = std::process::Command::new("explorer.exe");
        command.arg(path);
        let result = windows_spawn_detached(command);
        if let Err(err) = result {
            tracing::error!("SandboxAPI failed to spawn explorer.exe: {err}");
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

#[cfg(target_os = "windows")]
fn windows_spawn_detached(
    mut command: std::process::Command,
) -> std::io::Result<()> {
    command.stdin(std::process::Stdio::null());
    command.stdout(std::process::Stdio::null());
    command.stderr(std::process::Stdio::null());

    use std::os::windows::process::CommandExt;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    command.creation_flags(CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW);

    command.spawn().map(|_| ())
}
