use url::Url;

pub async fn open_url(url: Url) {
    assert!(matches!(url.scheme(), "http" | "https"));

    #[cfg(target_os = "linux")]
    {
        let result = ashpd::desktop::open_uri::OpenFileRequest::default()
            .send_uri(&url)
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
