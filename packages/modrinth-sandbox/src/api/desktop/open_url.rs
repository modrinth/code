use url::Url;

use crate::SandboxHandle;

pub async fn open_url(
    url: Url,
    #[cfg_attr(not(target_os = "macos"), allow(unused))] sandbox_handle: Option<
        SandboxHandle,
    >,
) {
    assert!(matches!(
        url.scheme(),
        "http" | "https" | "steam" | "signalrgb"
    ));

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
    open_url_macos(&url, sandbox_handle.as_ref());

    #[cfg(not(target_os = "macos"))]
    if let Err(err) = open::that_detached(url.as_str()) {
        tracing::error!("SandboxAPI failed to open url: {err}");
    }
}

#[cfg(target_os = "macos")]
fn open_url_macos(url: &Url, sandbox_handle: Option<&SandboxHandle>) {
    let workspace = objc2_app_kit::NSWorkspace::sharedWorkspace();
    let nsstring = objc2_foundation::NSString::from_str(url.as_str());
    let Some(nsurl) = objc2_foundation::NSURL::URLWithString(&nsstring) else {
        tracing::error!("SandboxAPI failed to convert url");
        return;
    };

    if matches!(url.scheme(), "http" | "https") {
        workspace.openURL(&nsurl);
        return;
    }

    let Some(sandbox_handle) = sandbox_handle else {
        tracing::error!(
            "SandboxAPI refusing to open {} url without a sandbox handle",
            url.scheme()
        );
        return;
    };
    let Some(application) = workspace
        .URLForApplicationToOpenURL(&nsurl)
        .and_then(|application| {
            super::resolve_trusted_application(&application, sandbox_handle)
        })
    else {
        tracing::warn!(
            "SandboxAPI found no trusted application to open {} urls",
            url.scheme()
        );
        return;
    };
    super::open_with_application(&workspace, &nsurl, &application);
}
