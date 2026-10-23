use std::path::PathBuf;

use eyre::eyre;

pub struct OpenFileDialogFilter {
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

pub async fn open_file_dialog(args: OpenFileDialogArgs) -> eyre::Result<Vec<PathBuf>> {
    #[cfg(target_os = "macos")]
    {
        open_file_dialog_macos(args).await
    }
}

#[cfg(target_os = "macos")]
pub async fn open_file_dialog_macos(args: OpenFileDialogArgs) -> eyre::Result<Vec<PathBuf>> {
    let (result_sender, result_receiver) = tokio::sync::oneshot::channel::<eyre::Result<Vec<PathBuf>>>();

    dispatch2::run_on_main(move |mtm| {
        if !objc2_app_kit::NSApplication::sharedApplication(mtm).isRunning() {
            _ = result_sender.send(Err(eyre!("cannot open file dialog in headless environment")));
            return;
        }

        let panel = if args.save {
            objc2_app_kit::NSSavePanel::new(mtm)
        } else {
            let Ok(panel) = objc2_app_kit::NSOpenPanel::new(mtm).downcast::<objc2_app_kit::NSSavePanel>() else {
                _ = result_sender.send(Err(eyre!("couldn't cast NSOpenPanel to NSSavePanel")));
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

        if let Some(open_panel) = panel.downcast_ref::<objc2_app_kit::NSOpenPanel>() && !args.save {
            open_panel.setAllowsMultipleSelection(args.multiple);
            open_panel.setCanChooseDirectories(args.directory);
            open_panel.setCanChooseFiles(!args.directory);
        }

        // For consistent behaviour with other platforms
        panel.setAllowsOtherFileTypes(true);

        // Filters
        if !args.filters.is_empty() {
            let filters = args.filters.iter().filter_map(|filter| {
                let extension = objc2_foundation::NSString::from_str(&filter.extension);
                objc2_uniform_type_identifiers::UTType::typeWithFilenameExtension(&extension)
            }).collect::<Vec<_>>();
            let filters = filters.iter().map(|uttype| &**uttype).collect::<Vec<_>>();
            if !filters.is_empty() {
                let array = objc2_foundation::NSArray::from_slice(&filters);
                panel.setAllowedContentTypes(&array);
            }
            if args.save {
                panel.setShowsContentTypes(filters.len() >= 2);
            }
        }

        // Default location
        if let Some(default_location) = &args.default_location {
            if default_location.is_dir() {
                let path = objc2_foundation::NSString::from_str(&default_location.to_string_lossy());
                let url = objc2_foundation::NSURL::fileURLWithPath_isDirectory(&path, true);
                panel.setDirectoryURL(Some(&url));
            } else if let Some(parent) = default_location.parent() {
                let parent_path = objc2_foundation::NSString::from_str(&parent.to_string_lossy());
                let parent_url = objc2_foundation::NSURL::fileURLWithPath_isDirectory(&parent_path, true);
                panel.setDirectoryURL(Some(&parent_url));

                if let Some(file_name) = default_location.file_name() {
                    let name_field = objc2_foundation::NSString::from_str(&file_name.to_string_lossy());
                    panel.setNameFieldStringValue(&name_field);
                }
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

            let urls = if args.multiple && let Some(open_panel) = retained_panel.downcast_ref::<objc2_app_kit::NSOpenPanel>() {
                open_panel.URLs().iter().collect::<Vec<_>>()
            } else if let Some(url) = retained_panel.URL() {
                vec![url]
            } else {
                vec![]
            };

            let mut paths = Vec::new();
            for url in urls {
                if !url.isFileURL() {
                    _ = result_sender.send(Err(eyre!("panel returned a url which was not a file:// url")));
                    return;
                }
                let Some(path) = url.to_file_path() else {
                    _ = result_sender.send(Err(eyre!("failed to convert url into file path")));
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
        .map_err(|_| eyre!("channel closed unexpectedly while waiting for file dialog result"))
        .flatten()
}
