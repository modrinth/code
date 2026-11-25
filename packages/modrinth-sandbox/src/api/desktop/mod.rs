mod open_url;
pub use open_url::*;

mod open_dir;
pub use open_dir::*;

mod launch_file;
pub use launch_file::*;

mod open_file_dialog;
pub use open_file_dialog::*;

/// Looks up the `UTType` for a file extension. Requires macOS 11.
///
/// The class is looked up at runtime because linking to the
/// `UniformTypeIdentifiers` framework would prevent the launcher from starting
/// on macOS 10.15.
#[cfg(target_os = "macos")]
fn content_type_for_extension(
    extension: &str,
) -> Option<objc2::rc::Retained<objc2::runtime::AnyObject>> {
    let class = objc2::runtime::AnyClass::get(c"UTType")?;
    let extension = objc2_foundation::NSString::from_str(extension);
    unsafe { objc2::msg_send![class, typeWithFilenameExtension: &*extension] }
}

#[cfg(target_os = "macos")]
const TRUSTED_APPLICATION_FOLDERS: &[&str] = &[
    "/Applications",
    "/System/Applications",
    "/System/Library/CoreServices",
    "/System/Volumes/Preboot/Cryptexes",
];

#[cfg(target_os = "macos")]
fn resolve_trusted_application(
    application: &objc2_foundation::NSURL,
    sandbox_handle: &crate::SandboxHandle,
) -> Option<objc2::rc::Retained<objc2_foundation::NSURL>> {
    let path = application.to_file_path()?.canonicalize().ok()?;

    let user_applications = std::env::home_dir()
        .and_then(|home| home.canonicalize().ok())
        .map(|home| home.join("Applications"));
    let in_trusted_folder = TRUSTED_APPLICATION_FOLDERS
        .iter()
        .map(std::path::Path::new)
        .chain(user_applications.as_deref())
        .any(|folder| path.starts_with(folder));

    if !in_trusted_folder || sandbox_handle.is_path_writeable(&path) {
        tracing::warn!(
            path = %path.display(),
            "SandboxAPI refusing to use untrusted application"
        );
        return None;
    }
    objc2_foundation::NSURL::from_file_path(path)
}

#[cfg(target_os = "macos")]
fn open_with_application(
    workspace: &objc2_app_kit::NSWorkspace,
    url: &objc2_foundation::NSURL,
    application: &objc2_foundation::NSURL,
) {
    let configuration =
        objc2_app_kit::NSWorkspaceOpenConfiguration::configuration();
    configuration.setPromptsUserIfNeeded(false);

    let completion_handler = block2::RcBlock::new(
        |_: *mut objc2_app_kit::NSRunningApplication,
         error: *mut objc2_foundation::NSError| {
            if let Some(error) = unsafe { error.as_ref() } {
                tracing::error!("SandboxAPI failed to open url: {error}");
            }
        },
    );
    workspace.openURLs_withApplicationAtURL_configuration_completionHandler(
        &objc2_foundation::NSArray::from_slice(&[url]),
        application,
        &configuration,
        Some(&completion_handler),
    );
}
