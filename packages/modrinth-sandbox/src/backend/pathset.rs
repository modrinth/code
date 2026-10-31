use std::path::PathBuf;

#[derive(Debug)]
pub struct ReadWritePathSet {
    // e.g. on bwrap, a read-only subpath cannot be written to even if there's a writeable parent
    exclusive: bool,
    readable: Vec<PathBuf>,
    writeable: Vec<PathBuf>,
}

impl ReadWritePathSet {
    pub fn new(exclusive: bool) -> Self {
        Self {
            exclusive,
            readable: Vec::new(),
            writeable: Vec::new(),
        }
    }

    pub fn add_read_only(&mut self, path: PathBuf) {
        #[cfg(windows)]
        let path = lower_path(path);

        if let Ok(existing) = self.writeable.binary_search(&path) {
            self.writeable.remove(existing);
        }

        match self.readable.binary_search(&path) {
            Ok(_) => {}
            Err(index) => {
                self.readable.insert(index, path);
            }
        }
    }

    pub fn add_read_write(&mut self, path: PathBuf) {
        #[cfg(windows)]
        let path = lower_path(path);

        match self.writeable.binary_search(&path) {
            Ok(_) => {}
            Err(index) => {
                self.writeable.insert(index, path.clone());
            }
        }
        match self.readable.binary_search(&path) {
            Ok(_) => {}
            Err(index) => {
                self.readable.insert(index, path);
            }
        }
    }

    pub fn can_read(&self, path: PathBuf) -> bool {
        #[cfg(windows)]
        let path = lower_path(path);

        let search_until = match self.readable.binary_search(&path) {
            Ok(_) => {
                return true;
            }
            Err(search_until) => search_until,
        };

        let Some(path_parent) = path.parent() else {
            return false;
        };

        for index in (0..search_until).rev() {
            let readable_candidate = &self.readable[index];
            if path.starts_with(readable_candidate) {
                return true;
            } else if readable_candidate.parent() == Some(path_parent) {
                continue; // Siblings, continue backwards to find potential parent
            } else {
                break;
            }
        }

        false
    }

    pub fn can_write(&self, path: PathBuf) -> bool {
        #[cfg(windows)]
        let path = lower_path(path);

        let search_write_until = match self.writeable.binary_search(&path) {
            Ok(_) => {
                return true;
            }
            Err(search_write_until) => search_write_until,
        };

        let Some(path_parent) = path.parent() else {
            return false;
        };

        for index in (0..search_write_until).rev() {
            let writeable_candidate = &self.writeable[index];
            if path.starts_with(writeable_candidate) {
                // If exclusive, we need to check if there's a read-only subpath
                if self.exclusive {
                    let search_read_until =
                        match self.readable.binary_search(&path) {
                            Ok(_) => {
                                return false; // Exact read-only path
                            }
                            Err(search_read_until) => search_read_until,
                        };

                    for index in (0..search_read_until).rev() {
                        let readable_candidate = &self.readable[index];
                        if path.starts_with(readable_candidate) {
                            // Found read-only parent, compare length
                            // If readable is longer, it means its a read-only subpath
                            // that takes priority over our writeable parent
                            return writeable_candidate.as_os_str().len()
                                >= readable_candidate.as_os_str().len();
                        } else if readable_candidate.parent()
                            == Some(path_parent)
                        {
                            continue; // Siblings, continue backwards to find potential parent
                        } else {
                            break;
                        }
                    }
                }

                return true;
            } else if writeable_candidate.parent() == Some(path_parent) {
                continue; // Siblings, continue backwards to find potential parent
            } else {
                break;
            }
        }
        false
    }
}

#[cfg(windows)]
fn lower_path(path: PathBuf) -> PathBuf {
    let mut os_string = path.into_os_string();
    os_string.make_ascii_lowercase();
    PathBuf::from(os_string)
}
