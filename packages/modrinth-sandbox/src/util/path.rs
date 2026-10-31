use std::path::{Path, PathBuf};

use eyre::{Context, Result, eyre};

pub fn resolve_path(path: &Path) -> Result<PathBuf> {
    let path = path
        .canonicalize()
        .wrap_err_with(|| eyre!("executable file {path:?} doesn't exist"))?;

    debug_assert!(path.is_absolute());

    #[cfg(windows)]
    {
        // Try to remove the \\?\ verbatim path prefix since it can break some applications
        let encoded_bytes = path.as_os_str().as_encoded_bytes();
        if let Some(rest) = encoded_bytes.strip_prefix(b"\\\\?\\") {
            return Ok(PathBuf::from(unsafe {
                std::ffi::OsStr::from_encoded_bytes_unchecked(&rest)
            }));
        }
    }

    Ok(path)
}
