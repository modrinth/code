use std::path::Path;

pub fn link_dir(source: &Path, link: &Path) -> std::io::Result<()> {
    #[cfg(windows)]
    {
        match std::os::windows::fs::symlink_dir(source, link) {
            Ok(()) => Ok(()),
            Err(_) => create_junction_with_cmd(source, link),
        }
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, link)
    }
}

pub fn link_file(source: &Path, link: &Path) -> std::io::Result<()> {
    #[cfg(windows)]
    {
        // Prefer hard links on Windows (no admin/dev mode required).
        // Fall back to symlink if hardlink cannot be created.
        std::fs::hard_link(source, link)
            .or_else(|_| std::os::windows::fs::symlink_file(source, link))
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, link)
    }
}

pub fn is_targeting(source: &Path, link: &Path) -> bool {
    let Ok(link_canonical) = std::fs::canonicalize(link) else {
        return false;
    };
    let Ok(source_canonical) = std::fs::canonicalize(source) else {
        return false;
    };
    link_canonical == source_canonical
}

pub fn is_targeting_file(source: &Path, link: &Path) -> bool {
    #[cfg(windows)]
    {
        // Prefer symlink target validation when available.
        if let Ok(target) = std::fs::read_link(link) {
            let Ok(target_canonical) = std::fs::canonicalize(target) else {
                return false;
            };
            let Ok(source_canonical) = std::fs::canonicalize(source) else {
                return false;
            };
            return target_canonical == source_canonical;
        }

        // Hard links cannot be reliably identified with stable std APIs on all
        // toolchains. Use a conservative fallback to avoid unstable features.
        false
    }
    #[cfg(unix)]
    {
        is_targeting(source, link)
    }
}

#[cfg(windows)]
fn create_junction_with_cmd(source: &Path, link: &Path) -> std::io::Result<()> {
    let source = source.to_string_lossy().to_string();
    let link = link.to_string_lossy().to_string();
    let status = std::process::Command::new("cmd")
        .args(["/C", "mklink", "/J", &link, &source])
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other("failed to create directory junction"))
    }
}
