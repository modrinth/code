use std::path::PathBuf;

#[cfg(any(target_os = "linux", target_os = "windows"))]
use anyhow::Context;
use anyhow::anyhow;

pub struct OpenFileDialogFilter {
    #[cfg_attr(target_os = "macos", allow(unused))]
    pub user_friendly_name: String,
    pub extension: String,
}

#[derive(Default)]
pub struct OpenFileDialogArgs {
    /// Title of the dialog window
    pub title: Option<String>,
    /// Label for the accept button
    pub accept_label: Option<String>,
    /// Whether the dialog should be a save dialog instead of an open dialog
    pub save: bool,
    /// Whether multiple files can be selected
    pub multiple: bool,
    /// Whether folders should be selected instead of files
    pub directory: bool,
    /// List of filters to apply
    pub filters: Vec<OpenFileDialogFilter>,
    /// Default path (may be folder or file)
    pub default_location: Option<PathBuf>,
}

pub async fn open_file_dialog(
    args: OpenFileDialogArgs,
) -> anyhow::Result<Vec<PathBuf>> {
    #[cfg(target_os = "macos")]
    {
        open_file_dialog_macos(args).await
    }
    #[cfg(target_os = "linux")]
    {
        open_file_dialog_linux(args).await
    }
    #[cfg(target_os = "windows")]
    {
        open_file_dialog_windows(args).await
    }
    #[cfg(not(any(
        target_os = "macos",
        target_os = "linux",
        target_os = "windows"
    )))]
    {
        drop(args);
        Err(anyhow::anyhow!("unsupported platform for file dialog"))
    }
}

fn split_default_location(
    default_location: &std::path::Path,
) -> (Option<PathBuf>, Option<std::ffi::OsString>) {
    let bytes = default_location.as_os_str().as_encoded_bytes();
    let has_trailing_separator =
        bytes.ends_with(b"/") || (cfg!(windows) && bytes.ends_with(b"\\"));
    if has_trailing_separator || default_location.is_dir() {
        (Some(default_location.to_path_buf()), None)
    } else {
        (
            default_location
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
                .map(std::path::Path::to_path_buf),
            default_location
                .file_name()
                .map(std::ffi::OsStr::to_os_string),
        )
    }
}

#[cfg(target_os = "macos")]
pub async fn open_file_dialog_macos(
    args: OpenFileDialogArgs,
) -> anyhow::Result<Vec<PathBuf>> {
    let (result_sender, result_receiver) =
        tokio::sync::oneshot::channel::<anyhow::Result<Vec<PathBuf>>>();

    dispatch2::run_on_main(move |mtm| {
        if !objc2_app_kit::NSApplication::sharedApplication(mtm).isRunning() {
            _ = result_sender.send(Err(anyhow!(
                "cannot open file dialog in headless environment"
            )));
            return;
        }

        let panel = if args.save {
            objc2_app_kit::NSSavePanel::new(mtm)
        } else {
            let Ok(panel) = objc2_app_kit::NSOpenPanel::new(mtm)
                .downcast::<objc2_app_kit::NSSavePanel>()
            else {
                _ = result_sender.send(Err(anyhow!(
                    "couldn't cast NSOpenPanel to NSSavePanel"
                )));
                return;
            };
            panel
        };

        if let Some(title) = &args.title {
            let title = objc2_foundation::NSString::from_str(title);
            panel.setTitle(Some(&title));
        }
        if let Some(accept_label) = &args.accept_label {
            let title = objc2_foundation::NSString::from_str(accept_label);
            panel.setPrompt(Some(&title));
        }

        if let Some(open_panel) =
            panel.downcast_ref::<objc2_app_kit::NSOpenPanel>()
            && !args.save
        {
            open_panel.setAllowsMultipleSelection(args.multiple);
            open_panel.setCanChooseDirectories(args.directory);
            open_panel.setCanChooseFiles(!args.directory);
        }

        // For consistent behaviour with other platforms
        panel.setAllowsOtherFileTypes(true);

        // Filters
        if !args.filters.is_empty() {
            set_panel_filters(&panel, &args.filters, args.save);
        }

        // Default location
        if let Some(default_location) = &args.default_location {
            let (folder, file_name) = split_default_location(default_location);
            if let Some(folder) = folder {
                let folder = objc2_foundation::NSString::from_str(
                    &folder.to_string_lossy(),
                );
                let url = objc2_foundation::NSURL::fileURLWithPath_isDirectory(
                    &folder, true,
                );
                panel.setDirectoryURL(Some(&url));
            }
            if let Some(file_name) = file_name {
                let name_field = objc2_foundation::NSString::from_str(
                    &file_name.to_string_lossy(),
                );
                panel.setNameFieldStringValue(&name_field);
            }
        }

        let retained_panel = panel.clone();
        let sender = std::cell::Cell::new(Some(result_sender));
        let closure = move |response: objc2_app_kit::NSModalResponse| {
            let Some(result_sender) = sender.take() else {
                return;
            };
            if response != objc2_app_kit::NSModalResponseOK {
                _ = result_sender.send(Ok(Vec::new()));
                return;
            }

            let urls = if args.multiple
                && let Some(open_panel) =
                    retained_panel.downcast_ref::<objc2_app_kit::NSOpenPanel>()
            {
                open_panel.URLs().iter().collect::<Vec<_>>()
            } else if let Some(url) = retained_panel.URL() {
                vec![url]
            } else {
                vec![]
            };

            let mut paths = Vec::new();
            for url in urls {
                if !url.isFileURL() {
                    _ = result_sender.send(Err(anyhow!(
                        "panel returned a url which was not a file:// url"
                    )));
                    return;
                }
                let Some(path) = url.to_file_path() else {
                    _ = result_sender.send(Err(anyhow!(
                        "failed to convert url into file path"
                    )));
                    return;
                };
                paths.push(path);
            }

            _ = result_sender.send(Ok(paths));
        };

        let block = block2::RcBlock::new(closure);
        panel.beginWithCompletionHandler(&block);
    });

    result_receiver.await
        .map_err(|_| anyhow!("channel closed unexpectedly while waiting for file dialog result"))
        .flatten()
}

#[cfg(target_os = "macos")]
fn set_panel_filters(
    panel: &objc2_app_kit::NSSavePanel,
    filters: &[OpenFileDialogFilter],
    save: bool,
) {
    use objc2_foundation::{NSArray, NSString};

    if !objc2::available!(macos = 11.0) {
        let file_types = filters
            .iter()
            .map(|filter| NSString::from_str(&filter.extension))
            .collect::<Vec<_>>();
        #[expect(
            deprecated,
            reason = "setAllowedContentTypes requires macOS 11"
        )]
        panel.setAllowedFileTypes(Some(&NSArray::from_retained_slice(
            &file_types,
        )));
        return;
    }

    let content_types = filters
        .iter()
        .filter_map(|filter| {
            super::content_type_for_extension(&filter.extension)
        })
        .collect::<Vec<_>>();
    if !content_types.is_empty() {
        let content_types = NSArray::from_retained_slice(&content_types);
        let _: () = unsafe {
            objc2::msg_send![panel, setAllowedContentTypes: &*content_types]
        };
    }
    if save && objc2::available!(macos = 15.0) {
        panel.setShowsContentTypes(content_types.len() >= 2);
    }
}

#[cfg(target_os = "linux")]
pub async fn open_file_dialog_linux(
    args: OpenFileDialogArgs,
) -> anyhow::Result<Vec<PathBuf>> {
    use ashpd::desktop::{ResponseError, file_chooser::SelectedFiles};

    let filters = linux_file_filters(&args.filters);
    let (folder, file_name) = match &args.default_location {
        Some(default_location) => split_default_location(default_location),
        None => (None, None),
    };

    let request = if args.save {
        let file_name = file_name.as_ref().map(|name| name.to_string_lossy());
        SelectedFiles::save_file()
            .title(args.title.as_deref())
            .accept_label(args.accept_label.as_deref())
            .modal(true)
            .current_name(file_name.as_deref())
            .current_folder::<&std::path::Path>(folder.as_deref())
            .context("setting default folder")?
            .filters(filters)
            .send()
            .await
    } else {
        SelectedFiles::open_file()
            .title(args.title.as_deref())
            .accept_label(args.accept_label.as_deref())
            .modal(true)
            .multiple(args.multiple)
            .directory(args.directory)
            .current_folder::<&std::path::Path>(folder.as_deref())
            .context("setting default folder")?
            .filters(if args.directory { Vec::new() } else { filters })
            .send()
            .await
    }
    .context("sending FileChooser portal request")?;

    let selected = match request.response() {
        Ok(selected) => selected,
        Err(ashpd::Error::Response(ResponseError::Cancelled)) => {
            return Ok(Vec::new());
        }
        Err(err) => {
            return Err(
                anyhow!(err).context("FileChooser portal request failed")
            );
        }
    };

    let mut paths = Vec::with_capacity(selected.uris().len());
    for uri in selected.uris() {
        if uri.scheme() != "file" {
            return Err(anyhow!(
                "portal returned a url which was not a file:// url"
            ));
        }
        let path = uri
            .to_file_path()
            .map_err(|_| anyhow!("failed to convert url into file path"))?;
        paths.push(path);
    }

    if !args.multiple {
        paths.truncate(1);
    }

    Ok(paths)
}

#[cfg(target_os = "linux")]
fn linux_file_filters(
    filters: &[OpenFileDialogFilter],
) -> Vec<ashpd::desktop::file_chooser::FileFilter> {
    use ashpd::desktop::file_chooser::FileFilter;

    let mut grouped: Vec<(&str, Vec<String>)> = Vec::new();
    for filter in filters {
        let extension = filter.extension.trim().trim_start_matches('.');
        if extension.is_empty() {
            continue;
        }
        let glob = case_insensitive_extension_glob(extension);
        let label = filter.user_friendly_name.as_str();
        match grouped.iter_mut().find(|(existing, _)| *existing == label) {
            Some((_, globs)) => {
                if !globs.contains(&glob) {
                    globs.push(glob);
                }
            }
            None => grouped.push((label, vec![glob])),
        }
    }

    if grouped.is_empty() {
        return Vec::new();
    }

    let mut portal_filters = grouped
        .into_iter()
        .map(|(label, globs)| {
            globs
                .iter()
                .fold(FileFilter::new(label), |filter, glob| filter.glob(glob))
        })
        .collect::<Vec<_>>();
    portal_filters.push(FileFilter::new("All files").glob("*"));
    portal_filters
}

/// Portal globs are case-sensitive, so `png` becomes `*.[pP][nN][gG]`.
#[cfg(target_os = "linux")]
fn case_insensitive_extension_glob(extension: &str) -> String {
    let mut glob = String::from("*.");
    for c in extension.chars() {
        let lower = c.to_ascii_lowercase();
        let upper = c.to_ascii_uppercase();
        if lower != upper {
            glob.push('[');
            glob.push(lower);
            glob.push(upper);
            glob.push(']');
        } else if matches!(c, '*' | '?' | '[' | ']') {
            glob.push('[');
            glob.push(c);
            glob.push(']');
        } else {
            glob.push(c);
        }
    }
    glob
}

#[cfg(target_os = "windows")]
pub async fn open_file_dialog_windows(
    args: OpenFileDialogArgs,
) -> anyhow::Result<Vec<PathBuf>> {
    let (result_sender, result_receiver) =
        tokio::sync::oneshot::channel::<anyhow::Result<Vec<PathBuf>>>();

    std::thread::Builder::new()
        .name("Modrinth Sandbox File Dialog".to_string())
        .spawn(move || {
            _ = result_sender.send(show_windows_file_dialog(&args));
        })
        .context("spawning file dialog thread")?;

    result_receiver.await.map_err(|_| {
        anyhow!(
            "channel closed unexpectedly while waiting for file dialog result"
        )
    })?
}

#[cfg(target_os = "windows")]
fn show_windows_file_dialog(
    args: &OpenFileDialogArgs,
) -> anyhow::Result<Vec<PathBuf>> {
    use windows::{
        Win32::{
            Foundation::ERROR_CANCELLED,
            System::Com::{
                CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
                COINIT_DISABLE_OLE1DDE, CoCreateInstance, CoInitializeEx,
                CoUninitialize,
            },
            UI::Shell::{
                Common::COMDLG_FILTERSPEC, FOS_ALLOWMULTISELECT,
                FOS_FILEMUSTEXIST, FOS_FORCEFILESYSTEM, FOS_NOCHANGEDIR,
                FOS_OVERWRITEPROMPT, FOS_PATHMUSTEXIST, FOS_PICKFOLDERS,
                FileOpenDialog, FileSaveDialog, IFileDialog, IFileOpenDialog,
                IFileSaveDialog, IShellItem, SHCreateItemFromParsingName,
            },
        },
        core::{HRESULT, Interface, PCWSTR},
    };

    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE)
    }
    .ok()
    .map_err(com_error("initializing COM"))?;

    scopeguard::defer! {
        unsafe { CoUninitialize() };
    }

    let (dialog, open_dialog): (IFileDialog, Option<IFileOpenDialog>) =
        if args.save {
            let dialog: IFileSaveDialog = unsafe {
                CoCreateInstance(&FileSaveDialog, None, CLSCTX_INPROC_SERVER)
            }
            .map_err(com_error("creating FileSaveDialog"))?;
            (
                dialog.cast().map_err(com_error("casting FileSaveDialog"))?,
                None,
            )
        } else {
            let dialog: IFileOpenDialog = unsafe {
                CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)
            }
            .map_err(com_error("creating FileOpenDialog"))?;
            (
                dialog.cast().map_err(com_error("casting FileOpenDialog"))?,
                Some(dialog),
            )
        };

    let mut options = unsafe { dialog.GetOptions() }
        .map_err(com_error("getting dialog options"))?
        | FOS_FORCEFILESYSTEM
        | FOS_NOCHANGEDIR
        | FOS_PATHMUSTEXIST;
    if args.save {
        options |= FOS_OVERWRITEPROMPT;
    } else {
        options |= FOS_FILEMUSTEXIST;
        if args.multiple {
            options |= FOS_ALLOWMULTISELECT;
        }
        if args.directory {
            options |= FOS_PICKFOLDERS;
        }
    }
    unsafe { dialog.SetOptions(options) }
        .map_err(com_error("setting dialog options"))?;

    if let Some(title) = &args.title {
        let title = to_wide(title);
        unsafe { dialog.SetTitle(PCWSTR(title.as_ptr())) }
            .map_err(com_error("setting dialog title"))?;
    }
    if let Some(accept_label) = &args.accept_label {
        let accept_label = to_wide(accept_label);
        unsafe { dialog.SetOkButtonLabel(PCWSTR(accept_label.as_ptr())) }
            .map_err(com_error("setting dialog accept label"))?;
    }

    // The wide strings must outlive the call to Show
    let filters = if args.directory {
        Vec::new()
    } else {
        windows_file_filters(&args.filters)
    };
    if !filters.is_empty() {
        let filter_specs = filters
            .iter()
            .map(|(name, spec)| COMDLG_FILTERSPEC {
                pszName: PCWSTR(name.as_ptr()),
                pszSpec: PCWSTR(spec.as_ptr()),
            })
            .collect::<Vec<_>>();
        unsafe { dialog.SetFileTypes(&filter_specs) }
            .map_err(com_error("setting dialog file types"))?;
    }
    if args.save
        && let Some(extension) = args
            .filters
            .iter()
            .map(|filter| filter.extension.trim().trim_start_matches('.'))
            .find(|extension| is_valid_windows_extension(extension))
    {
        let extension = to_wide(extension);
        unsafe { dialog.SetDefaultExtension(PCWSTR(extension.as_ptr())) }
            .map_err(com_error("setting dialog default extension"))?;
    }

    if let Some(default_location) = &args.default_location {
        let (folder, file_name) = split_default_location(default_location);
        if let Some(folder) = folder {
            let folder = to_wide(folder.as_os_str());
            let folder_item: windows::core::Result<IShellItem> = unsafe {
                SHCreateItemFromParsingName(PCWSTR(folder.as_ptr()), None)
            };
            if let Ok(folder_item) = folder_item {
                _ = unsafe { dialog.SetFolder(&folder_item) };
            }
        }
        if let Some(file_name) = file_name {
            let file_name = to_wide(file_name.as_os_str());
            unsafe { dialog.SetFileName(PCWSTR(file_name.as_ptr())) }
                .map_err(com_error("setting dialog file name"))?;
        }
    }

    if let Err(err) = unsafe { dialog.Show(None) } {
        if err.code() == HRESULT::from_win32(ERROR_CANCELLED.0) {
            return Ok(Vec::new());
        }
        return Err(com_error("showing file dialog")(err));
    }

    let mut paths = Vec::new();
    if let Some(open_dialog) = &open_dialog
        && args.multiple
    {
        let items = unsafe { open_dialog.GetResults() }
            .map_err(com_error("getting dialog results"))?;
        let count = unsafe { items.GetCount() }
            .map_err(com_error("getting result count"))?;
        for index in 0..count {
            let item = unsafe { items.GetItemAt(index) }
                .map_err(com_error("getting dialog result"))?;
            paths.push(windows_shell_item_path(&item)?);
        }
    } else {
        let item = unsafe { dialog.GetResult() }
            .map_err(com_error("getting dialog result"))?;
        paths.push(windows_shell_item_path(&item)?);
    }

    Ok(paths)
}

#[cfg(target_os = "windows")]
fn com_error(
    context: &'static str,
) -> impl FnOnce(windows::core::Error) -> anyhow::Error {
    move |err| anyhow!("{err}").context(context)
}

#[cfg(target_os = "windows")]
fn is_valid_windows_extension(extension: &str) -> bool {
    !extension.is_empty() && !extension.contains(['*', '?', ';', '\0'])
}

#[cfg(target_os = "windows")]
fn windows_shell_item_path(
    item: &windows::Win32::UI::Shell::IShellItem,
) -> anyhow::Result<PathBuf> {
    use std::os::windows::ffi::OsStringExt;
    use windows::Win32::{
        System::Com::CoTaskMemFree, UI::Shell::SIGDN_FILESYSPATH,
    };

    let name = unsafe { item.GetDisplayName(SIGDN_FILESYSPATH) }
        .map_err(com_error("getting file system path of dialog result"))?;
    let path = std::ffi::OsString::from_wide(unsafe { name.as_wide() });
    unsafe { CoTaskMemFree(Some(name.0 as *const _)) };
    Ok(PathBuf::from(path))
}

/// Groups extensions by label into `(name, "*.a;*.b")` pairs as
/// null-terminated wide strings, followed by an "All files" entry.
#[cfg(target_os = "windows")]
fn windows_file_filters(
    filters: &[OpenFileDialogFilter],
) -> Vec<(Vec<u16>, Vec<u16>)> {
    let mut grouped: Vec<(&str, Vec<String>)> = Vec::new();
    for filter in filters {
        let extension = filter.extension.trim().trim_start_matches('.');
        if !is_valid_windows_extension(&extension) {
            continue;
        }
        let pattern = format!("*.{extension}");
        let label = filter.user_friendly_name.as_str();
        match grouped.iter_mut().find(|(existing, _)| *existing == label) {
            Some((_, patterns)) => {
                if !patterns.contains(&pattern) {
                    patterns.push(pattern);
                }
            }
            None => grouped.push((label, vec![pattern])),
        }
    }

    if grouped.is_empty() {
        return Vec::new();
    }

    let mut specs = grouped
        .into_iter()
        .map(|(label, patterns)| (to_wide(label), to_wide(patterns.join(";"))))
        .collect::<Vec<_>>();
    specs.push((to_wide("All files"), to_wide("*.*")));
    specs
}

#[cfg(target_os = "windows")]
fn to_wide(value: impl AsRef<std::ffi::OsStr>) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    value.as_ref().encode_wide().chain([0]).collect()
}
