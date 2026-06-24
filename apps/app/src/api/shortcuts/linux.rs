use crate::api::Result;
use std::path::Path;
use url::Url;

pub(super) const SHORTCUT_EXTENSION: &str = "desktop";

pub(super) async fn create_shortcut(
    profile_name: &str,
    launch_url: &Url,
    output_path: &Path,
) -> Result<()> {
    let target_path = std::env::current_exe()?;
    tokio::fs::write(
        output_path,
        format!(
            "[Desktop Entry]\n\
			Type=Application\n\
			Name={}\n\
			Exec={} {}\n\
			Icon=ModrinthApp\n\
			Terminal=false\n\
			Categories=Game;\n",
            escape_desktop_entry_value(&format!("Launch {profile_name}")),
            quote_desktop_exec_arg(&target_path.to_string_lossy()),
            quote_desktop_exec_arg(launch_url.as_str()),
        ),
    )
    .await?;

    use std::os::unix::fs::PermissionsExt;

    let mut permissions = tokio::fs::metadata(output_path).await?.permissions();
    permissions.set_mode(0o755);
    tokio::fs::set_permissions(output_path, permissions).await?;

    Ok(())
}

fn escape_desktop_entry_value(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\r', "")
}

fn quote_desktop_exec_arg(input: &str) -> String {
    format!(
        "\"{}\"",
        input
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('$', "\\$")
            .replace('`', "\\`")
    )
}
