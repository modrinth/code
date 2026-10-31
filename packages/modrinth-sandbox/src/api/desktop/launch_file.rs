use std::path::PathBuf;

pub async fn launch_file_safe(path: PathBuf) {
    let Some(extension) = path.extension() else {
        return;
    };

    // Ensure extension matches files we allow (images, video, audio)
    match extension.to_ascii_lowercase().as_encoded_bytes() {
        // Image formats
        b"jpg" | b"jpeg" | b"png" | b"gif" | b"webp" | b"cr2" | b"tif"
        | b"tiff" | b"bmp" => {}
        // Video formats
        b"mp4" | b"m4v" | b"mkv" | b"webm" | b"mov" | b"avi" | b"mpg" => {}
        // Audio formats
        b"mid" | b"mp3" | b"m4a" | b"ogg" | b"flac" | b"wav" | b"amr"
        | b"aac" | b"aiff" => {}
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
        let mut new_path =
            std::env::temp_dir().join(uuid::Uuid::now_v7().to_string());
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
    if !matches!(
        matcher_type,
        infer::MatcherType::Image | infer::MatcherType::Video | infer::MatcherType::Audio
    ) {
        return;
    }

    launch_file(path).await
}

async fn launch_file(path: PathBuf) {
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
