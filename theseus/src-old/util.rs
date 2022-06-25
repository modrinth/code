use std::path::{Path, PathBuf};
use std::{env, io};

use path_clean::PathClean;

// https://stackoverflow.com/a/54817755
pub fn absolute_path(path: impl AsRef<Path>) -> io::Result<PathBuf> {
    let path = path.as_ref();

    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()?.join(path)
    }
    .clean();

    Ok(absolute_path)
}
