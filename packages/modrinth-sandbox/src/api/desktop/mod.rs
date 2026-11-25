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
