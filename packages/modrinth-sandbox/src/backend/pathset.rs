use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct PathSet {
    paths: Vec<PathBuf>,
}

#[derive(Debug, Default)]
pub struct ReadWritePathSet {
    readable: PathSet,
    writeable: PathSet,
}

impl PathSet {
    pub fn insert(&mut self, path: PathBuf) -> bool {
        #[cfg(windows)]
        let path = lower_path(path);

        match self.paths.binary_search(&path) {
            Ok(_) => false,
            Err(index) => {
                while index < self.paths.len() {
                    let potential_child = &self.paths[index];
                    if potential_child.starts_with(&path) {
                        self.paths.remove(index);
                    } else {
                        break;
                    }
                }

                self.paths.insert(index, path);
                true
            }
        }
    }

    pub fn contains(&self, path: PathBuf) -> bool {
        #[cfg(windows)]
        let path = lower_path(path);

        let search_until = match self.paths.binary_search(&path) {
            Ok(_) => {
                return true;
            }
            Err(search_until) => search_until,
        };

        let Some(path_parent) = path.parent() else {
            return false;
        };

        for index in (0..search_until).rev() {
            let candidate = &self.paths[index];
            if path.starts_with(candidate) {
                return true;
            } else if candidate.parent() == Some(path_parent) {
                continue; // Siblings, continue backwards to find potential parent
            } else {
                break;
            }
        }

        false
    }
}

impl ReadWritePathSet {
    pub fn from_vecs(
        mut read_only: Vec<PathBuf>,
        mut read_write: Vec<PathBuf>,
    ) -> Self {
        let mut set = Self::default();

        read_write.sort();
        read_write.dedup();
        for path in read_write {
            set.insert_writeable(path);
        }

        read_only.sort();
        read_only.dedup();
        for path in read_only {
            set.insert_readable(path);
        }

        set
    }

    pub fn insert_readable(&mut self, path: PathBuf) -> bool {
        self.readable.insert(path)
    }

    pub fn insert_writeable(&mut self, path: PathBuf) -> bool {
        self.readable.insert(path.clone());
        self.writeable.insert(path)
    }

    pub fn can_read(&self, path: PathBuf) -> bool {
        self.readable.contains(path.clone())
    }

    pub fn can_write(&self, path: PathBuf) -> bool {
        self.writeable.contains(path)
    }

    #[cfg_attr(not(windows), allow(unused))]
    pub fn iter_both(&self) -> impl Iterator<Item = &PathBuf> {
        self.readable.paths.iter()
    }

    pub fn iter_read_only(&self) -> impl Iterator<Item = &PathBuf> {
        self.readable
            .paths
            .iter()
            .filter(|path| !self.writeable.contains(path.to_path_buf()))
    }

    pub fn iter_read_write(&self) -> impl Iterator<Item = &PathBuf> {
        self.writeable.paths.iter()
    }
}

#[cfg(windows)]
fn lower_path(path: PathBuf) -> PathBuf {
    let mut os_string = path.into_os_string();
    os_string.make_ascii_lowercase();
    PathBuf::from(os_string)
}
