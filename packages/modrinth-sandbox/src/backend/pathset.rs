use std::path::PathBuf;

#[cfg(windows)]
use crate::path_access::normalize_path;

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
        let path = normalize_path(path);

        match self.paths.binary_search(&path) {
            Ok(_) => false,
            Err(index) => {
                if index > 0 && path.starts_with(&self.paths[index - 1]) {
                    return false;
                }

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
        let path = normalize_path(path);

        match self.paths.binary_search(&path) {
            Ok(_) => true,
            Err(index) => index > 0 && path.starts_with(&self.paths[index - 1]),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &PathBuf> {
        self.paths.iter()
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
        self.readable.contains(path)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn set(read_only: &[&str], read_write: &[&str]) -> ReadWritePathSet {
        ReadWritePathSet::from_vecs(
            read_only.iter().map(PathBuf::from).collect(),
            read_write.iter().map(PathBuf::from).collect(),
        )
    }

    fn paths<'a>(iter: impl Iterator<Item = &'a PathBuf>) -> Vec<PathBuf> {
        iter.cloned().collect()
    }

    #[test]
    fn children_resolve_against_parent() {
        let set = set(&["/jre"], &["/inst"]);

        assert!(set.can_read("/inst".into()));
        assert!(set.can_write("/inst".into()));
        assert!(set.can_read("/inst/saves/world".into()));
        assert!(set.can_write("/inst/saves/world".into()));

        assert!(set.can_read("/jre".into()));
        assert!(set.can_read("/jre/bin/java".into()));
        assert!(!set.can_write("/jre".into()));
        assert!(!set.can_write("/jre/bin/java".into()));
    }

    #[test]
    fn unrelated_and_partial_name_matches_are_rejected() {
        let set = set(&["/jre"], &["/inst"]);

        for path in ["/", "/in", "/instance", "/inst-2/file", "/zzz", "/aaa"] {
            assert!(!set.can_read(path.into()), "{path} should be unreadable");
            assert!(
                !set.can_write(path.into()),
                "{path} should be unwriteable"
            );
        }
    }

    #[test]
    fn sibling_sorting_between_parent_and_child_does_not_hide_parent() {
        let set = set(&["/a-b"], &["/a"]);

        assert!(set.can_write("/a/x".into()));
        assert!(set.can_read("/a-b/x".into()));
        assert!(!set.can_write("/a-b/x".into()));
        assert!(!set.can_read("/a-c".into()));
    }

    #[test]
    fn read_only_child_of_writeable_parent_is_writeable() {
        let set = set(&["/inst/mods"], &["/inst"]);

        assert!(set.can_write("/inst/mods/mod.jar".into()));
        assert!(set.can_write("/inst/saves".into()));
        assert_eq!(paths(set.iter_read_only()), Vec::<PathBuf>::new());
        assert_eq!(paths(set.iter_read_write()), [PathBuf::from("/inst")]);
        assert_eq!(paths(set.iter_both()), [PathBuf::from("/inst")]);
    }

    #[test]
    fn writeable_child_of_read_only_parent_stays_writeable() {
        let set = set(&["/ro"], &["/ro/rw"]);

        assert!(set.can_read("/ro/file".into()));
        assert!(!set.can_write("/ro/file".into()));
        assert!(set.can_read("/ro/rw/file".into()));
        assert!(set.can_write("/ro/rw/file".into()));
        assert_eq!(paths(set.iter_read_only()), [PathBuf::from("/ro")]);
        assert_eq!(paths(set.iter_read_write()), [PathBuf::from("/ro/rw")]);
    }

    #[test]
    fn insert_drops_nested_and_duplicate_entries() {
        let mut set = PathSet::default();

        assert!(set.insert("/a/b".into()));
        assert!(set.insert("/a/c".into()));
        assert!(!set.insert("/a/b".into()));
        assert!(!set.insert("/a/b/deeper".into()));

        assert!(set.insert("/a".into()));
        assert_eq!(set.paths, [PathBuf::from("/a")]);

        assert!(!set.insert("/a/d".into()));
        assert!(set.contains("/a/b".into()));
        assert!(set.contains("/a/d".into()));
    }

    #[test]
    fn empty_set_contains_nothing() {
        let set = ReadWritePathSet::default();

        assert!(!set.can_read("/".into()));
        assert!(!set.can_write("/anything".into()));
    }
}
