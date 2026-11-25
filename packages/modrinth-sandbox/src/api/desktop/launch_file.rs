use std::path::PathBuf;

use crate::SandboxHandle;

pub async fn launch_file_safe(path: PathBuf, sandbox_handle: SandboxHandle) {
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
        infer::MatcherType::Image
            | infer::MatcherType::Video
            | infer::MatcherType::Audio
    ) {
        return;
    }

    launch_file(path, sandbox_handle).await
}

async fn launch_file(
    path: PathBuf,
    #[cfg_attr(not(target_os = "macos"), allow(unused))]
    sandbox_handle: SandboxHandle,
) {
    #[cfg(target_os = "macos")]
    launch_file_macos(&path, &sandbox_handle);

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
        } else {
            return;
        }
    }

    #[cfg(not(target_os = "macos"))]
    if let Err(err) = open::that_detached(path) {
        tracing::error!("SandboxAPI failed to open file: {err}");
    }
}

#[cfg(target_os = "macos")]
fn launch_file_macos(path: &std::path::Path, sandbox_handle: &SandboxHandle) {
    use objc2_app_kit::NSWorkspace;
    use objc2_foundation::NSURL;

    let Some(url) = NSURL::from_file_path(path) else {
        return;
    };
    let Some(extension) = path.extension().and_then(|ext| ext.to_str()) else {
        return;
    };

    // We explicitly look up the application ourselves to avoid exploits
    // involving replacing the file with a symlink pointing at a different
    // file type.
    // If a malicious mod tries to replace `a.png` with a symlink
    // pointing at `b.sh` then `b.sh` will end up being opened with Preview
    // which will probably fail

    let workspace = NSWorkspace::sharedWorkspace();
    let Some(application) = find_application_for_extension(
        &workspace, extension,
    )
    .and_then(|application| {
        super::resolve_trusted_application(&application, sandbox_handle)
    }) else {
        tracing::warn!(
            "SandboxAPI found no trusted application to open .{extension} files"
        );
        return;
    };

    super::open_with_application(&workspace, &url, &application);
}

#[cfg(target_os = "macos")]
fn find_application_for_extension(
    workspace: &objc2_app_kit::NSWorkspace,
    extension: &str,
) -> Option<objc2::rc::Retained<objc2_foundation::NSURL>> {
    if objc2::available!(macos = 12.0) {
        let content_type = super::content_type_for_extension(extension)?;
        return unsafe {
            objc2::msg_send![
                workspace,
                URLForApplicationToOpenContentType: &*content_type
            ]
        };
    }

    let placeholder = tempfile::Builder::new()
        .suffix(&format!(".{extension}"))
        .tempfile()
        .ok()?;
    let url = objc2_foundation::NSURL::from_file_path(placeholder.path())?;
    workspace.URLForApplicationToOpenURL(&url)
}
