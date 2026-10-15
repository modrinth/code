use std::path::PathBuf;

use infer::MatcherType;
use url::Url;

pub async fn open_url(url: Url) {
    assert!(matches!(url.scheme(), "http" | "https"));

    #[cfg(target_os = "linux")]
    if let Ok(uri) = ashpd::Uri::parse(url.as_str()) {
        let result = ashpd::desktop::open_uri::OpenFileRequest::default()
            .send_uri(&uri)
            .await;
        if let Err(err) = result {
            tracing::error!("SandboxAPI failed to send OpenURI: {err}");
        } else {
            return;
        }
    }

    #[cfg(target_os = "macos")]
    {
        let workspace = objc2_app_kit::NSWorkspace::sharedWorkspace();
        let nsstring = objc2_foundation::NSString::from_str(url.as_str());
        if let Some(path) = objc2_foundation::NSURL::URLWithString(&nsstring) {
            workspace.openURL(&path);
            return;
        }
    }

    if let Err(err) = open::that_detached(url.as_str()) {
        tracing::error!("SandboxAPI failed to open url: {err}");
    }
}

pub async fn open_dir(path: PathBuf) {
    // We have to be very careful here, using something like open::that(path) will work,
    // however it opens us up to a TOCTOU exploit where the folder's path is replaced by an executable file.
    // Therefore, we use platform specific operations that are designed to specifically open a folder
    #[cfg(target_os = "macos")]
    {
        let Some(path_str) = path.to_str() else {
            return;
        };
        let workspace = objc2_app_kit::NSWorkspace::sharedWorkspace();
        let path = objc2_foundation::NSString::from_str(path_str);
        workspace.selectFile_inFileViewerRootedAtPath(None, &path);
    }
    #[cfg(target_os = "windows")]
    {
        let mut command = std::process::Command::new("explorer.exe");
        command.arg(path);
        let result = Self::windows_spawn_detached(command);
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

pub async fn open_file_safe(path: PathBuf) {
    let Some(extension) = path.extension() else {
        return;
    };

    // Ensure extension matches files we allow (images, video, audio)
    match extension.to_ascii_lowercase().as_encoded_bytes() {
        // Image formats
        b"jpg" | b"jpeg" | b"png" | b"gif" | b"webp" | b"cr2" | b"tif" | b"tiff" | b"bmp" => {},
        // Video formats
        b"mp4" | b"m4v" | b"mkv" | b"webm" | b"mov" | b"avi" | b"mpg" => {}
        // Audio formats
        b"mid" | b"mp3" | b"m4a" | b"ogg" | b"flac" | b"wav" | b"amr" | b"aac" | b"aiff" => {}
        // Refuse to open other formats
        _ => {
            return;
        }
    }

    // On Linux, file opening is typically determined by the mime type rather than extension
    // Therefore, for security reasons, we need to copy the file to a safe place before we check the mime type
    // of the file. This is to prevent a TOCTOU exploit where the program could replace the file
    // with a malicious executable after we check the mime.
    #[cfg(target_os = "linux")]
    let path = {
        let mut new_path = std::env::temp_dir().join(Uuid::now_v7().to_string());
        new_path.set_extension(extension);
        if tokio::fs::copy(&path, &new_path).await.is_err() {
            return;
        }
        new_path
    };

    // Ensure mime type is image, video or audio
    let Ok(Some(file_type)) = infer::get_from_path(&path) else {
        return;
    };
    let matcher_type = file_type.matcher_type();
    if !matches!(matcher_type, MatcherType::Image | MatcherType::Video | MatcherType::Audio) {
        return;
    }

    open_file(path).await
}

async fn open_file(path: PathBuf) {
    #[cfg(target_os = "macos")]
    {
        let workspace = objc2_app_kit::NSWorkspace::sharedWorkspace();
        if let Some(path) = objc2_foundation::NSURL::from_file_path(&path) {
            workspace.openURL(&path);
            return;
        }
    }

    #[cfg(target_os = "linux")]
    {
        let Ok(file) = std::fs::File::open(&path) else {
            return;
        };
        let result = ashpd::desktop::open_uri::OpenFileRequest::default()
            .send_file(&std::os::fd::AsFd::as_fd(&file))
            .await;
        if let Err(err) = result {
            tracing::error!("SandboxAPI failed to send OpenFile: {err}");
        }
    }

    if let Err(err) = open::that_detached(path) {
        tracing::error!("SandboxAPI failed to open file: {err}");
    }
}
