use std::{collections::{HashMap, HashSet, VecDeque}, ffi::{CString, OsStr}, hash::Hash, os::{fd::{AsFd, BorrowedFd, OwnedFd}, unix::ffi::OsStrExt}, path::PathBuf, sync::{Arc, atomic::{AtomicBool, Ordering}}, time::{Duration, SystemTime}};

use eyre::{Context, bail, eyre};
use indexmap::IndexMap;
use parking_lot::Mutex;
use rand::Rng;
use rustix::fs::{AtFlags, OFlags, RenameFlags};

const TTL: Duration = Duration::ZERO;
const O_DEFAULT: OFlags = OFlags::NOFOLLOW.union(OFlags::CLOEXEC);

pub fn mount(mountpoint: PathBuf, force: bool) -> eyre::Result<FileSharing> {
    if fuser::INodeNo::ROOT.0 != 1 {
        bail!("expected root inode to have value 1");
    }

    if force {
        if let Ok(cstring) = CString::new(mountpoint.as_os_str().as_encoded_bytes()) {
            unsafe { libc::unmount(cstring.as_ptr(), 0) };
        }
    }

    let abort = AbortHandle::default();
    let filesystem = FileSharingFS(Arc::new(Mutex::new(FileSharingFSInner {
        mountpoint: mountpoint.clone(),
        process_uid: unsafe { libc::getuid() },
        process_gid: unsafe { libc::getgid() },
        abort: abort.clone(),
        files: FileSharingFiles {
            abort,
            ..Default::default()
        },
        ..Default::default()
    })));
    let mut cfg = fuser::Config::default();
    cfg.mount_options.push(fuser::MountOption::FSName("modrinth-sandbox-file-sharing".to_string()));
    cfg.mount_options.push(fuser::MountOption::RW);
    cfg.mount_options.push(fuser::MountOption::AutoUnmount);
    cfg.mount_options.push(fuser::MountOption::NoDev);
    cfg.mount_options.push(fuser::MountOption::NoSuid);
    cfg.mount_options.push(fuser::MountOption::DefaultPermissions);
    cfg.acl = fuser::SessionACL::All;
    let session = fuser::spawn_mount(filesystem.clone(), mountpoint, &cfg).context("mounting fuse filesystem")?;

    let (delete_sender, delete_receiver) = std::sync::mpsc::channel::<(fuser::INodeNo, fuser::INodeNo, Arc<OsStr>)>();

    let notifier = session.notifier();
    std::thread::Builder::new()
        .name("INode Delete Notifier Thread".to_string())
        .stack_size(16384)
        .spawn(move || {
            let Ok((parent, child, name)) = delete_receiver.recv() else {
                return;
            };
            _ = notifier.delete(parent, child, &name);
        }).context("spawning thread to handle inode deletion")?;

    filesystem.0.lock().files.delete_notifier = Some(delete_sender);
    Ok(FileSharing {
        filesystem,
        session: Arc::new(session)
    })
}

#[derive(Clone, Debug)]
pub struct FileSharing {
    filesystem: FileSharingFS,
    session: Arc<fuser::BackgroundSession>,
}

impl FileSharing {
    pub fn add(&self, path: PathBuf, mutable: bool) -> eyre::Result<PathBuf> {
        let mut fs = self.filesystem.0.lock();
        if let Some(shutdown_reason) = fs.abort.get_abort_message() {
            bail!(shutdown_reason);
        }

        let Some(parent) = path.parent() else {
            bail!("path must have a parent to be shared");
        };
        let Some(filename) = path.file_name() else {
            bail!("path must have a filename to be shared");
        };
        let filename: Arc<OsStr> = filename.into();

        let dirfd = match rustix::fs::open(parent, OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC, rustix::fs::Mode::empty()) {
            Ok(dirfd) => dirfd,
            Err(errno) => {
                let io_error = std::io::Error::from_raw_os_error(errno.raw_os_error());
                return Err(eyre!("unable to open parent directory").wrap_err(io_error));
            },
        };

        let parent_stat = match rustix::fs::fstat(&dirfd) {
            Ok(stat) => stat,
            Err(errno) => {
                let io_error = std::io::Error::from_raw_os_error(errno.raw_os_error());
                return Err(eyre!("unable to stat parent directory").wrap_err(io_error));
            },
        };
        let child_stat = match rustix::fs::statat(&dirfd, &*filename, AtFlags::SYMLINK_NOFOLLOW) {
            Ok(stat) => {
                Some(stat)
            },
            Err(errno) => {
                if errno == rustix::io::Errno::NOENT {
                    None
                } else {
                    let io_error = std::io::Error::from_raw_os_error(errno.raw_os_error());
                    return Err(eyre!("unable to stat file").wrap_err(io_error));
                }
            },
        };

        let mut identified_parent = IdentifiedFd {
            st_ino: parent_stat.st_ino,
            st_dev: parent_stat.st_dev,
            fd: Arc::new(dirfd),
        };

        let files = &mut fs.files;
        if let Some(parent_data) = files.existing_parents.get_mut(&identified_parent) {
            // Reuse existing parent fd
            identified_parent = parent_data.own_fd.clone();

            // Try find existing bind
            let hexes = if mutable {
                &parent_data.rw
            } else {
                &parent_data.ro
            };

            for hex_bytes in hexes {
                let Some(node_index) = files.hex_to_node_index.get(hex_bytes) else {
                    continue;
                };
                let Some(Some(node)) = files.nodes.get_mut(node_index.0) else {
                    continue;
                };
                let FileNodeInner::RootFile(root) = &node.inner else {
                    continue;
                };
                if node.filename == filename {
                    let hex_bytes = root.hex;

                    let mut path = fs.mountpoint.join(hex::encode(hex_bytes));
                    path.push(&*filename);

                    return Ok(path);
                }
            }
        }

        let file_type = child_stat.map(|stat| to_fuser_filetype(rustix::fs::FileType::from_raw_mode(stat.st_mode)));
        let hex_bytes = fs.files.add_file(identified_parent.clone(), filename.clone(), file_type, mutable);

        if let Some(shutdown_reason) = fs.abort.get_abort_message() {
            bail!(shutdown_reason);
        }

        let parent_data = fs.files.existing_parents.entry(identified_parent.clone()).or_insert_with(|| {
            ExistingParentData {
                own_fd: identified_parent,
                ro: Vec::new(),
                rw: Vec::new()
            }
        });
        if mutable {
            parent_data.rw.push(hex_bytes);
        } else {
            parent_data.ro.push(hex_bytes);
        };

        let mut path = fs.mountpoint.join(hex::encode(hex_bytes));
        path.push(&*filename);

        Ok(path)
    }

    pub fn join(self) -> eyre::Result<()> {
        Arc::into_inner(self.session).unwrap().join()?;
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum INodeType {
    Root,
    HexFolder(FileNodeIndex),
    FileNode(FileNodeIndex),
}

impl INodeType {
    pub fn from_ino(ino: fuser::INodeNo) -> Option<Self> {
        if ino.0 == 0 {
            return None;
        }
        debug_assert_eq!(fuser::INodeNo::ROOT.0, 1);
        if ino == fuser::INodeNo::ROOT {
            return Some(Self::Root);
        }
        let index = (ino.0 >> 1).checked_sub(1)?;
        if (ino.0 & 1) == 1 {
            Some(Self::HexFolder(FileNodeIndex(index as usize)))
        } else {
            Some(Self::FileNode(FileNodeIndex(index as usize)))
        }
    }

    pub fn to_ino(self) -> fuser::INodeNo {
        match self {
            INodeType::Root => fuser::INodeNo::ROOT,
            INodeType::HexFolder(index) => {
                fuser::INodeNo(((index.0.strict_add(1) << 1) | 1) as u64)
            },
            INodeType::FileNode(index) => {
                fuser::INodeNo((index.0.strict_add(1) << 1) as u64)
            },
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
struct FileNodeIndex(usize);

#[derive(Debug)]
struct FileNode {
    own_index: FileNodeIndex,
    generation: fuser::Generation,
    refcnt: usize,
    file_type: Option<fuser::FileType>,
    filename: Arc<OsStr>,
    subfiles: IndexMap<Arc<OsStr>, FileNodeIndex>,
    inner: FileNodeInner,
}

#[derive(Debug)]
enum FileNodeInner {
    RootFile(RootFileNode),
    Subfile(SubfileNode)
}

#[derive(Debug)]
struct RootFileNode {
    hex: [u8; 16],
    // SECURITY: We store the parent FD + filename here instead of a path to prevent exploits
    // If a path was used instead, a parent directory or the file itself could be set to a symlink
    // pointing outside the folder.
    // We use methods like openat, fstatat, etc. with the parent fd + filename and the NOFOLLOW flag
    // to interact with the referenced file in a safe way
    dirfd: IdentifiedFd,
    mutable: bool,
}

#[derive(Debug)]
struct SubfileNode {
    parent: FileNodeIndex,
}

#[derive(Clone, Debug)]
struct IdentifiedFd {
    st_ino: u64,
    st_dev: i32,
    fd: Arc<OwnedFd>
}

impl AsFd for IdentifiedFd {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.fd.as_fd()
    }
}

impl PartialEq for IdentifiedFd {
    fn eq(&self, other: &Self) -> bool {
        self.st_ino == other.st_ino && self.st_dev == other.st_dev
    }
}

impl Eq for IdentifiedFd {
}

impl PartialOrd for IdentifiedFd {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.st_ino.partial_cmp(&other.st_ino) {
            Some(std::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.st_dev.partial_cmp(&other.st_dev) {
            Some(std::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        Some(std::cmp::Ordering::Equal)
    }
}

impl Ord for IdentifiedFd {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.st_ino.cmp(&other.st_ino) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match self.st_dev.cmp(&other.st_dev) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        std::cmp::Ordering::Equal
    }
}

impl Hash for IdentifiedFd {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.st_ino.hash(state);
        self.st_dev.hash(state);
    }
}

#[derive(PartialEq, Eq, Debug)]
enum FileHandleOpenedBy {
    Open,
    Opendir,
}

#[derive(Debug)]
struct FileHandleData {
    opened_by: FileHandleOpenedBy,
    underlying_fd: Option<Arc<OwnedFd>>,
    underlying_dir_entries: Option<UnderlyingDirEntries>,
    mutable: bool,
}

#[derive(Debug)]
enum UnderlyingDirEntries {
    HexFolders {
        hex_folders: Box<[([u8; 16], FileNodeIndex)]>,
    },
    Single {
        filename: Arc<OsStr>,
        file_node_index: FileNodeIndex,
    },
    Backed {
        dir: rustix::fs::Dir,
        last_offset: u64,
        parent: FileNodeIndex,
    },
}

#[derive(Default, Clone, Debug)]
struct AbortHandle(Arc<AbortHandleInner>);

#[derive(Default, Debug)]
struct AbortHandleInner {
    reason: Mutex<&'static str>,
    aborted: AtomicBool,
}

impl AbortHandle {
    pub fn abort(&self, reason: &'static str) {
        if !self.0.aborted.swap(true, Ordering::Relaxed) {
            *self.0.reason.lock() = reason;
        }
        tracing::error!("Aborting file sharing fs due to unexpected error: {reason})");
    }

    pub fn is_aborted(&self) -> bool {
        self.0.aborted.load(Ordering::Relaxed)
    }

    pub fn get_abort_message(&self) -> Option<&'static str> {
        if self.0.aborted.load(Ordering::Relaxed) {
            Some(*self.0.reason.lock())
        } else {
            None
        }
    }
}

#[derive(Default, Debug)]
struct FileSharingFSInner {
    mountpoint: PathBuf,
    process_uid: u32,
    process_gid: u32,
    abort: AbortHandle,
    reuse_buffer: Vec<u8>,

    files: FileSharingFiles,

    last_handle_id: u64,
    file_handle_data: HashMap<u64, FileHandleData>,
}

#[derive(Debug)]
struct ExistingParentData {
    own_fd: IdentifiedFd,
    ro: Vec<[u8; 16]>,
    rw: Vec<[u8; 16]>,
}

#[derive(Default, Debug)]
struct FileSharingFiles {
    abort: AbortHandle,
    nodes: Vec<Option<FileNode>>,
    node_index_pending_forget: HashMap<FileNodeIndex, (usize, fuser::Generation)>,
    node_index_free_list: VecDeque<(FileNodeIndex, fuser::Generation)>,
    hex_to_node_index: IndexMap<[u8; 16], FileNodeIndex>,

    existing_parents: HashMap<IdentifiedFd, ExistingParentData>,

    delete_notifier: Option<std::sync::mpsc::Sender<(fuser::INodeNo, fuser::INodeNo, Arc<OsStr>)>>,
}

impl FileSharingFiles {
    fn get_node_mut(&mut self, file_node_index: FileNodeIndex) -> Result<&mut FileNode, fuser::Errno> {
        let Some(Some(file_node)) = self.nodes.get_mut(file_node_index.0) else {
            return Err(fuser::Errno::ENOENT);
        };
        if file_node.own_index != file_node_index {
            self.abort.abort("get_node_mut: file_node.own_index doesn't match index in fs.nodes");
            Err(fuser::Errno::EIO)
        } else {
            Ok(file_node)
        }
    }

    fn get_node(&self, file_node_index: FileNodeIndex) -> Result<&FileNode, fuser::Errno> {
        let Some(Some(file_node)) = self.nodes.get(file_node_index.0) else {
            return Err(fuser::Errno::ENOENT);
        };
        if file_node.own_index != file_node_index {
            self.abort.abort("get_node: file_node.own_index doesn't match index in fs.nodes");
            Err(fuser::Errno::EIO)
        } else {
            Ok(file_node)
        }
    }

    fn take_node_index(&mut self) -> (FileNodeIndex, fuser::Generation) {
        if let Some((id, generation)) = self.node_index_free_list.pop_front() {
            (id, fuser::Generation(generation.0 + 1))
        } else {
            (FileNodeIndex(self.nodes.len()), fuser::Generation(0))
        }
    }

    fn open_ino(&mut self, ino: fuser::INodeNo, flags: OFlags) -> Result<Arc<OwnedFd>, fuser::Errno> {
        self.open_ino_and_check_mutable(ino, flags, (flags & OFlags::ACCMODE) != OFlags::RDONLY)
    }

    fn open_ino_and_check_mutable(&mut self, ino: fuser::INodeNo, flags: OFlags, check_mutable: bool) -> Result<Arc<OwnedFd>, fuser::Errno> {
        let flags = O_DEFAULT | flags;

        let Some(ino_type) = INodeType::from_ino(ino) else {
            return Err(fuser::Errno::ENOENT);
        };

        let file_node_index = match ino_type {
            INodeType::Root | INodeType::HexFolder(_) => {
                return Err(fuser::Errno::EACCES);
            },
            INodeType::FileNode(file_node_index) => {
                file_node_index
            },
        };

        let node = self.get_node(file_node_index)?;

        match &node.inner {
            FileNodeInner::RootFile(root_file_node) => {
                if check_mutable && !root_file_node.mutable {
                    return Err(fuser::Errno::EACCES);
                }

                let fd = rustix::fs::openat(&root_file_node.dirfd, &*node.filename, flags, rustix::fs::Mode::empty())
                    .map_err(to_fuser_errno)?;

                Ok(Arc::new(fd))

            },
            FileNodeInner::Subfile(_) => {
                let mut visited = HashSet::new();
                let mut filenames = VecDeque::new();

                let mut scan_node_index = file_node_index;
                let parent_fd = loop {
                    let node = self.get_node(scan_node_index)?;

                    match &node.inner {
                        FileNodeInner::RootFile(root_file_node) => {
                            if check_mutable && !root_file_node.mutable {
                                return Err(fuser::Errno::EACCES);
                            }

                            break rustix::fs::openat(
                                &root_file_node.dirfd,
                                &*node.filename,
                                O_DEFAULT | OFlags::RDONLY | OFlags::DIRECTORY,
                                rustix::fs::Mode::empty()
                            ).map_err(to_fuser_errno)?;
                        },
                        FileNodeInner::Subfile(subfile_node) => {
                            filenames.push_front(node.filename.clone());

                            visited.insert(scan_node_index);
                            if visited.contains(&subfile_node.parent) {
                                return Err(fuser::Errno::ELOOP);
                            }
                            if visited.len() > 256 {
                                return Err(fuser::Errno::ENAMETOOLONG);
                            }
                            scan_node_index = subfile_node.parent;
                        },
                    }
                };

                // Try to use OPENAT2 with RESOLVE_BENEATH
                #[cfg(target_os = "linux")]
                {
                    let mut relative_path = PathBuf::new();
                    for filename in filenames {
                        relative_path.push(&*filename);
                    }

                    static SUPPORTS_OPENAT2: AtomicBool = AtomicBool::new(true);
                    if SUPPORTS_OPENAT2.load(Ordering::Relaxed) {
                        for _ in 0..3 { // Retry up to 3 times if receiving EAGAIN
                            let result = rustix::fs::openat2(
                                &parent_fd,
                                &relative_path,
                                flags,
                                rustix::fs::Mode::empty(),
                                rustix::fs::ResolveFlags::BENEATH,
                            ).map_err(to_fuser_errno);
                            match result {
                                Ok(fd) => {
                                    return Ok(Arc::new(fd));
                                }
                                Err(errno) => {
                                    if errno == fuser::Errno::ENOSYS || errno == fuser::Errno::EPERM {
                                        SUPPORTS_OPENAT2.store(false, Ordering::Relaxed);
                                        break;
                                    } else if errno == fuser::Errno::EAGAIN {
                                        continue;
                                    } else {
                                        return Err(errno);
                                    }
                                }
                            }
                        }
                    }
                }

                // When OPENAT2 isn't supported, we fall back to opening each directory in order with O_NOFOLLOW
                let mut last_fd = parent_fd;

                let filenames_len = filenames.len();
                for (index, filename) in filenames.into_iter().enumerate() {
                    let open_flags = if index+1 < filenames_len {
                        // Open directory
                        O_DEFAULT | OFlags::RDONLY | OFlags::DIRECTORY
                    } else {
                        flags
                    };
                    last_fd = rustix::fs::openat(&last_fd, &*filename, open_flags, rustix::fs::Mode::empty())
                        .map_err(to_fuser_errno)?;
                }

                Ok(Arc::new(last_fd))
            },
        }
    }

    fn open_ino_for_child(&mut self, ino: fuser::INodeNo, child: &OsStr, check_mutable: bool) -> Result<Arc<OwnedFd>, fuser::Errno> {
        let Some(inode_type) = INodeType::from_ino(ino) else {
            return Err(fuser::Errno::ENOENT);
        };

        let flags = O_DEFAULT | OFlags::RDONLY | OFlags::DIRECTORY;

        match inode_type {
            INodeType::Root => {
                Err(fuser::Errno::EACCES)
            },
            INodeType::HexFolder(file_node_index) => {
                let node = self.get_node(file_node_index)?;
                match &node.inner {
                    FileNodeInner::RootFile(root_file_node) => {
                        if check_mutable && !root_file_node.mutable {
                            Err(fuser::Errno::EACCES)
                        } else if &*node.filename != child {
                            if check_mutable {
                                Err(fuser::Errno::EACCES)
                            } else {
                                Err(fuser::Errno::ENOENT)
                            }
                        } else {
                            Ok(root_file_node.dirfd.fd.clone())
                        }
                    },
                    FileNodeInner::Subfile(_) => {
                        self.abort.abort("open_ino_for_child: HexFolder inode contained subfile instead of rootfile");
                        Err(fuser::Errno::EIO)
                    },
                }
            },
            INodeType::FileNode(_) => {
                self.open_ino_and_check_mutable(ino, flags, check_mutable)
            },
        }
    }

    fn open_parent_ino(&mut self, ino: fuser::INodeNo, check_mutable: bool) -> Result<(Arc<OwnedFd>, Arc<OsStr>), fuser::Errno> {
        let Some(inode_type) = INodeType::from_ino(ino) else {
            return Err(fuser::Errno::ENOENT);
        };

        let flags = O_DEFAULT | OFlags::RDONLY | OFlags::DIRECTORY;

        match inode_type {
            INodeType::Root => {
                Err(fuser::Errno::EACCES)
            },
            INodeType::HexFolder(_) => {
                Err(fuser::Errno::EACCES)
            },
            INodeType::FileNode(file_node_index) => {
                let node = self.get_node(file_node_index)?;

                match &node.inner {
                    FileNodeInner::RootFile(root_file_node) => {
                        if check_mutable && !root_file_node.mutable {
                            return Err(fuser::Errno::EACCES);
                        }
                        Ok((root_file_node.dirfd.fd.clone(), node.filename.clone()))
                    },
                    FileNodeInner::Subfile(subfile_node) => {
                        let filename = node.filename.clone();
                        let parent_ino = INodeType::FileNode(subfile_node.parent).to_ino();
                        let parent = self.open_ino_and_check_mutable(parent_ino, flags, check_mutable)?;
                        Ok((parent, filename))
                    },
                }
            },
        }
    }

    fn add_file(&mut self, dirfd: IdentifiedFd, filename: Arc<OsStr>, file_type: Option<fuser::FileType>, mutable: bool) -> [u8; 16] {
        let (node_index, node_generation) = self.take_node_index();

        let hex = loop {
            let hex: [u8; 16] = rand::thread_rng().r#gen();
            if !self.hex_to_node_index.contains_key(&hex) {
                break hex;
            }
        };

        let node = FileNode {
            own_index: node_index,
            generation: node_generation,
            refcnt: 0,
            file_type,
            filename,
            subfiles: Default::default(),
            inner: FileNodeInner::RootFile(RootFileNode {
                hex,
                dirfd,
                mutable,
            }),
        };

        if node_index.0 == self.nodes.len() {
            self.nodes.push(Some(node));
        } else {
            if self.nodes[node_index.0].is_some() {
                self.abort.abort("add_file: reused file_index already had data at index");
            }
            self.nodes[node_index.0] = Some(node);
        }

        self.hex_to_node_index.insert(hex, node_index);
        hex
    }

    fn add_subfile(&mut self, parent: FileNodeIndex, filename: Arc<OsStr>, refcnt: usize, file_type: fuser::FileType) -> (FileNodeIndex, fuser::Generation) {
        let (node_index, node_generation) = self.take_node_index();

        let node = FileNode {
            own_index: node_index,
            generation: node_generation,
            refcnt,
            file_type: Some(file_type),
            filename,
            subfiles: Default::default(),
            inner: FileNodeInner::Subfile(SubfileNode { parent }),
        };

        if node_index.0 == self.nodes.len() {
            self.nodes.push(Some(node));
        } else {
            if self.nodes[node_index.0].is_some() {
                self.abort.abort("add_subfile: reused file_index already had data at index");
            }
            self.nodes[node_index.0] = Some(node);
        }

        (node_index, node_generation)
    }

    fn try_remove_ino_child(&mut self, parent: fuser::INodeNo, name: &OsStr) -> Result<(), fuser::Errno> {
        let Some(inode_type) = INodeType::from_ino(parent) else {
            return Err(fuser::Errno::ENOENT);
        };

        match inode_type {
            INodeType::Root => {
                Err(fuser::Errno::EACCES)
            },
            INodeType::HexFolder(file_node_index) => {
                let node = self.get_node(file_node_index)?;

                if &*node.filename == name {
                    self.remove_node(file_node_index)
                } else {
                    Ok(())
                }
            },
            INodeType::FileNode(file_node_index) => {
                let node = self.get_node_mut(file_node_index)?;

                if let Some(removed) = node.subfiles.shift_remove(&*name) {
                    self.remove_node(removed)
                } else {
                    Ok(())
                }
            },
        }
    }

    fn remove_node(&mut self, file_node_index: FileNodeIndex) -> Result<(), fuser::Errno> {
        if file_node_index.0 >= self.nodes.len() {
            return Err(fuser::Errno::ENOENT);
        }

        let existing = &mut self.nodes[file_node_index.0 as usize];
        let Some(mut node) = std::mem::replace(existing, None) else {
            self.abort.abort("remove_node: file that was being removed was missing data");
            return Err(fuser::Errno::EIO);
        };

        if node.own_index != file_node_index {
            self.abort.abort("remove_node: own_index of removed node doesn't match index in fs.nodes");
            return Err(fuser::Errno::EIO);
        }

        if node.refcnt == 0 {
            self.node_index_free_list.push_back((node.own_index, node.generation));
        } else {
            self.node_index_pending_forget.insert(node.own_index, (node.refcnt, node.generation));
        }

        let subfiles = std::mem::take(&mut node.subfiles);
        for (_, subfile_index) in subfiles {
            self.remove_node(subfile_index)?;
        }

        match node.inner {
            FileNodeInner::RootFile(root) => {
                // Remove node from hex map
                if self.hex_to_node_index.shift_remove(&root.hex) != Some(file_node_index) {
                    self.abort.abort("remove_node: hex_to_file_index[file.hex] != Some(file_index)");
                    return Err(fuser::Errno::EIO);
                }

                // Remove info from real parent data
                let empty = if let Some(real_parent_data) = self.existing_parents.get_mut(&root.dirfd) {
                    real_parent_data.ro.retain(|ro| ro != &root.hex);
                    real_parent_data.rw.retain(|rw| rw != &root.hex);
                    real_parent_data.ro.is_empty() && real_parent_data.rw.is_empty()
                } else {
                    false
                };
                if empty {
                    self.existing_parents.remove(&root.dirfd);
                }

                if let Some(delete_notifier) = &self.delete_notifier {
                    // File itself
                    let hex_folder = INodeType::HexFolder(file_node_index).to_ino();
                    let root_file = INodeType::FileNode(file_node_index).to_ino();

                    _ = delete_notifier.send((hex_folder, root_file, node.filename.clone()));

                    // Hex folder
                    let hex_name = hex::encode(root.hex);
                    _ = delete_notifier.send((fuser::INodeNo::ROOT, hex_folder, OsStr::new(&hex_name).into()));
                }
            },
            FileNodeInner::Subfile(subfile_node) => {
                if let Some(delete_notifier) = &self.delete_notifier {
                    _ = delete_notifier.send((
                        INodeType::FileNode(subfile_node.parent).to_ino(),
                        INodeType::FileNode(file_node_index).to_ino(),
                        node.filename.clone()
                    ));
                }
            },
        }

        Ok(())
    }

    fn get_or_create_child(&mut self, parent: FileNodeIndex, name: &OsStr, increase_refcnt: bool) -> Result<(FileNodeIndex, fuser::Generation, rustix::fs::Stat), fuser::Errno> {
        let parent_ino = INodeType::FileNode(parent).to_ino();
        let dirfd = self.open_ino_for_child(parent_ino, name, false)?;

        let stat = rustix::fs::statat(dirfd, name, AtFlags::SYMLINK_NOFOLLOW).map_err(to_fuser_errno);
        let stat = match stat {
            Ok(stat) => stat,
            Err(errno) => {
                if should_remove_on_error(errno) {
                    self.try_remove_ino_child(parent_ino, name)?;
                }
                return Err(errno);
            },
        };

        let parent_node = self.get_node_mut(parent)?;

        if let Some(subfile_node_index) = parent_node.subfiles.get(name).copied() {
            let subfile_node = self.get_node_mut(subfile_node_index)?;
            if increase_refcnt {
                subfile_node.refcnt += 1;
            }
            subfile_node.file_type = Some(to_fuser_filetype(rustix::fs::FileType::from_raw_mode(stat.st_mode)));
            Ok((subfile_node_index, subfile_node.generation, stat))
        } else {
            let name: Arc<OsStr> = name.into();
            let initial_refcnt = if increase_refcnt {
                1
            } else {
                0
            };
            let file_type = to_fuser_filetype(rustix::fs::FileType::from_raw_mode(stat.st_mode));
            let (subfile_node_index, subfile_generation) = self.add_subfile(parent,
                name.clone(), initial_refcnt, file_type);

            let parent_node = self.get_node_mut(parent)?;
            parent_node.subfiles.insert(name, subfile_node_index);
            Ok((subfile_node_index, subfile_generation, stat))
        }
    }
}

impl FileSharingFSInner {
    pub fn abort(&self, reason: &'static str) {
        self.abort.abort(reason);
    }

    fn create_dummy_directory_attr(&self, ino: fuser::INodeNo) -> fuser::FileAttr {
        fuser::FileAttr {
            ino,
            size: 0,
            blocks: 0,
            atime: SystemTime::UNIX_EPOCH,
            mtime: SystemTime::UNIX_EPOCH,
            ctime: SystemTime::UNIX_EPOCH,
            crtime: SystemTime::UNIX_EPOCH,
            kind: fuser::FileType::Directory,
            perm: 0o755,
            nlink: 1,
            uid: self.process_uid,
            gid: self.process_gid,
            rdev: 0,
            blksize: 512,
            flags: 0,
        }
    }

    fn create_root_attr(&self) -> fuser::FileAttr {
        self.create_dummy_directory_attr(fuser::INodeNo::ROOT)
    }

    fn create_hex_folder_attr(&self, node_index: FileNodeIndex) -> fuser::FileAttr {
        self.create_dummy_directory_attr(INodeType::HexFolder(node_index).to_ino())
    }

    fn create_file_attr(&self, ino: fuser::INodeNo, stat: rustix::fs::Stat) -> fuser::FileAttr {
        fuser::FileAttr {
            ino,
            size: stat.st_size as _,
            blocks: stat.st_blocks as _,
            atime: i64_to_systemtime(stat.st_atime),
            mtime: i64_to_systemtime(stat.st_mtime),
            ctime: i64_to_systemtime(stat.st_ctime),
            #[cfg(target_vendor = "apple")]
            crtime: i64_to_systemtime(stat.st_birthtime),
            #[cfg(not(target_vendor = "apple"))]
            crtime: SystemTime::UNIX_EPOCH,
            kind: to_fuser_filetype(rustix::fs::FileType::from_raw_mode(stat.st_mode)),
            perm: stat.st_mode,
            nlink: stat.st_nlink as _,
            uid: stat.st_uid as _,
            gid: stat.st_gid as _,
            rdev: stat.st_rdev as _,
            blksize: stat.st_blksize as _,
            #[cfg(target_vendor = "apple")]
            flags: stat.st_flags,
            #[cfg(not(target_vendor = "apple"))]
            flags: 0,
        }
    }

    fn push_file_handle_data(&mut self, data: FileHandleData) -> fuser::FileHandle {
        self.last_handle_id += 1;
        while self.file_handle_data.contains_key(&self.last_handle_id) || self.last_handle_id == 0 {
            self.last_handle_id += 1;
        }
        self.file_handle_data.insert(self.last_handle_id, data);
        fuser::FileHandle(self.last_handle_id)
    }

    fn do_lookup(&mut self, parent: fuser::INodeNo, name: &OsStr, increase_refcnt: bool) -> Result<(fuser::FileAttr, fuser::Generation), fuser::Errno> {
        let Some(inode_type) = INodeType::from_ino(parent) else {
            return Err(fuser::Errno::ENOENT);
        };

        match inode_type {
            INodeType::Root => {
                let mut id = [0_u8; 16];
                if hex::decode_to_slice(name.as_encoded_bytes(), &mut id).is_err() {
                    return Err(fuser::Errno::ENOENT);
                }

                let Some(file_node_index) = self.files.hex_to_node_index.get(&id).copied() else {
                    return Err(fuser::Errno::ENOENT);
                };

                let attr = self.create_hex_folder_attr(file_node_index);
                let node = self.files.get_node_mut(file_node_index)?;
                let generation = node.generation;

                if increase_refcnt {
                    node.refcnt += 1;
                }

                Ok((attr, generation))
            },
            INodeType::HexFolder(file_node_index) => {
                let node = self.files.get_node_mut(file_node_index)?;

                let FileNodeInner::RootFile(root_file_node) = &mut node.inner else {
                    self.abort("do_lookup: HexFolder inode contained subfile instead of rootfile");
                    return Err(fuser::Errno::EIO);
                };

                if &*node.filename != name {
                    return Err(fuser::Errno::ENOENT);
                }

                let stat = match rustix::fs::statat(&root_file_node.dirfd, name, AtFlags::SYMLINK_NOFOLLOW) {
                    Ok(stat) => stat,
                    Err(errno) => {
                        let errno = to_fuser_errno(errno);

                        if node.file_type.is_some() && should_remove_on_error(errno) {
                            self.files.remove_node(file_node_index)?;
                        }

                        return Err(errno);
                    },
                };

                node.file_type = Some(to_fuser_filetype(rustix::fs::FileType::from_raw_mode(stat.st_mode)));

                let generation = node.generation;

                if increase_refcnt {
                    node.refcnt += 1;
                }

                let attr = self.create_file_attr(INodeType::FileNode(file_node_index).to_ino(), stat);
                Ok((attr, generation))
            },
            INodeType::FileNode(parent_file_node_index) => {
                let (subfile_node_index, subfile_generation, stat) = self.files.get_or_create_child(parent_file_node_index,
                    name, increase_refcnt)?;

                let attr = self.create_file_attr(INodeType::FileNode(subfile_node_index).to_ino(), stat);
                Ok((attr, subfile_generation))
            },
        }
    }

    fn do_read_dir(&mut self, folder_ino: fuser::INodeNo, fh: fuser::FileHandle, mut offset: u64, reply: &mut fuser::ReplyDirectory) -> Result<(), fuser::Errno> {
        let Some(inode_type) = INodeType::from_ino(folder_ino) else {
            return Err(fuser::Errno::ENOENT);
        };

        if offset == 0 {
            if reply.add(folder_ino, 1, fuser::FileType::Directory, ".") {
                return Ok(());
            }
        } else {
            offset -= 1;
        }

        if offset == 0 {
            let parent_ino = match inode_type {
                INodeType::Root => {
                    // For parent inode on the root, just return the inode of the root
                    folder_ino
                },
                INodeType::HexFolder(_) => {
                    fuser::INodeNo::ROOT
                },
                INodeType::FileNode(file_node_index) => {
                    let node = self.files.get_node(file_node_index)?;
                    match &node.inner {
                        FileNodeInner::RootFile(_) => INodeType::HexFolder(file_node_index),
                        FileNodeInner::Subfile(subfile_node) => INodeType::FileNode(subfile_node.parent),
                    }.to_ino()
                },
            };
            if reply.add(parent_ino, 2, fuser::FileType::Directory, "..") {
                return Ok(());
            }
        } else {
            offset -= 1;
        }

        let Some(file_handle_data) = self.file_handle_data.get_mut(&fh.0) else {
            return Err(fuser::Errno::EBADF);
        };

        let Some(underlying_dir_entries) = &mut file_handle_data.underlying_dir_entries else {
            return Err(fuser::Errno::EBADF);
        };

        const BASE_OFFSET: u64 = 3;
        match underlying_dir_entries {
            UnderlyingDirEntries::HexFolders { hex_folders } => {
                let mut next_index = BASE_OFFSET + offset;
                for (hex_bytes, file_node_index) in hex_folders.iter().skip(offset as _) {
                    let name = hex::encode(hex_bytes);
                    let ino = INodeType::HexFolder(*file_node_index).to_ino();

                    if reply.add(ino, next_index, fuser::FileType::Directory, name) {
                        return Ok(());
                    }

                    next_index += 1;
                }
            },
            UnderlyingDirEntries::Single { filename, file_node_index } => {
                if offset == 0 {
                    let filename = filename.clone();
                    let file_node_index = file_node_index.clone();

                    let Ok((attr, _)) = self.do_lookup(INodeType::HexFolder(file_node_index).to_ino(), &filename, false) else {
                        return Ok(());
                    };

                    let ino = INodeType::FileNode(file_node_index).to_ino();

                    if reply.add(ino, BASE_OFFSET, attr.kind, filename) {
                        return Ok(());
                    }
                }
            },
            UnderlyingDirEntries::Backed { dir, last_offset, parent } => {
                let Some(Some(parent_node)) = self.files.nodes.get_mut(parent.0) else {
                    return Err(fuser::Errno::ENOENT);
                };
                let mut parent_node = parent_node;
                if parent_node.own_index != *parent {
                    self.abort.abort("do_read_dir: parent_node.own_index doesn't match index in fs.nodes");
                    return Err(fuser::Errno::EIO);
                }

                if offset != *last_offset {
                    if offset == 0 {
                        dir.rewind();
                    } else {
                        dir.seek(offset as _).map_err(to_fuser_errno)?;
                    }
                    *last_offset = offset;
                }
                while let Some(entry) = dir.next() {
                    let entry = entry.map_err(to_fuser_errno)?;
                    *last_offset += 1;

                    let file_name = OsStr::from_bytes(entry.file_name().to_bytes());

                    if file_name == "." || file_name == ".." {
                        continue;
                    }

                    let file_type = to_fuser_filetype(entry.file_type());

                    let subfile_node_index = if let Some(subfile_node_index) = parent_node.subfiles.get(file_name).copied() {
                        subfile_node_index
                    } else {
                        let name: Arc<OsStr> = file_name.into();
                        let initial_refcnt = 0;
                        let (subfile_node_index, _) = self.files.add_subfile(*parent, name.clone(), initial_refcnt, file_type);

                        let Some(Some(reborrowed_parent_node)) = self.files.nodes.get_mut(parent.0) else {
                            return Err(fuser::Errno::ENOENT);
                        };
                        parent_node = reborrowed_parent_node;
                        parent_node.subfiles.insert(name, subfile_node_index);
                        subfile_node_index
                    };

                    let ino = INodeType::FileNode(subfile_node_index).to_ino();
                    let reply_offset = BASE_OFFSET + *last_offset - 1; // need -1 because we already incremented last_offset
                    if reply.add(ino, reply_offset, fuser::FileType::Directory, file_name) {
                        return Ok(());
                    }
                }
            },
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
struct FileSharingFS(Arc<Mutex<FileSharingFSInner>>);

impl fuser::Filesystem for FileSharingFS {
    fn init(&mut self, _req: &fuser::Request, _config: &mut fuser::KernelConfig) -> std::io::Result<()> {
        Ok(())
    }

    fn lookup(&self, _req: &fuser::Request, parent: fuser::INodeNo, name: &std::ffi::OsStr, reply: fuser::ReplyEntry) {
        let mut fs = self.0.lock();
        if fs.abort.is_aborted() {
            return reply.error(fuser::Errno::EIO);
        }

        tracing::debug!("lookup(parent: {parent:x?}, name: {name:?})");

        match fs.do_lookup(parent, name, true) {
            Ok((attr, generation)) => reply.entry(&TTL, &attr, generation),
            Err(errno) => reply.error(errno),
        }
    }

    fn forget(&self, _req: &fuser::Request, ino: fuser::INodeNo, nlookup: u64) {
        let mut fs = self.0.lock();
        if fs.abort.is_aborted() {
            return;
        }

        // todo: check that this forget logic actually works
        // macfuse seems to have an issue and never issues a forget
        // so i couldn't properly test this

        tracing::debug!("forget(ino: {ino:x?}, nlookup: {nlookup:?})");

        let Some(inode_type) = INodeType::from_ino(ino) else {
            return;
        };

        let file_node_index = match inode_type {
            INodeType::Root => {
                return;
            },
            INodeType::HexFolder(file_node_index) => {
                file_node_index
            },
            INodeType::FileNode(file_node_index) => {
                file_node_index
            },
        };

        match fs.files.get_node_mut(file_node_index) {
            Ok(node) => {
                node.refcnt = node.refcnt.saturating_sub(nlookup as usize);
            },
            Err(_) => {
                if let Some((refcnt, generation)) = fs.files.node_index_pending_forget.get_mut(&file_node_index) {
                    *refcnt = refcnt.saturating_sub(nlookup as usize);
                    if *refcnt == 0 {
                        let generation = *generation;
                        _ = fs.files.node_index_pending_forget.remove(&file_node_index);
                        fs.files.node_index_free_list.push_back((file_node_index, generation));
                    }
                };
            },
        }

    }

    fn getattr(&self, _req: &fuser::Request, ino: fuser::INodeNo, fh: Option<fuser::FileHandle>, reply: fuser::ReplyAttr) {
        let mut fs = self.0.lock();
        if fs.abort.is_aborted() {
            return reply.error(fuser::Errno::EIO);
        }

        tracing::debug!("getattr(ino: {ino:x?}, fh: {fh:?})");

        // Handle existing fd
        if let Some(fh) = fh && let Some(data) = fs.file_handle_data.get(&fh.0) && let Some(fd) = &data.underlying_fd {
            let stat = match rustix::fs::fstat(fd) {
                Ok(stat) => stat,
                Err(errno) => return reply.error(to_fuser_errno(errno)),
            };

            let attr = fs.create_file_attr(ino, stat);
            return reply.attr(&TTL, &attr);
        }

        // Handle without fd
        let Some(inode_type) = INodeType::from_ino(ino) else {
            return reply.error(fuser::Errno::ENOENT);
        };

        match inode_type {
            INodeType::Root => {
                let attr = fs.create_root_attr();
                return reply.attr(&TTL, &attr);
            },
            INodeType::HexFolder(file_node_index) => {
                let attr = fs.create_hex_folder_attr(file_node_index);
                return reply.attr(&TTL, &attr);
            },
            INodeType::FileNode(_) => {
                let (dirfd, name) = match fs.files.open_parent_ino(ino, false) {
                    Ok(res) => res,
                    Err(errno) => return reply.error(errno),
                };

                let stat = match rustix::fs::statat(dirfd, &*name, AtFlags::SYMLINK_NOFOLLOW) {
                    Ok(stat) => stat,
                    Err(errno) => return reply.error(to_fuser_errno(errno)),
                };

                let attr = fs.create_file_attr(ino, stat);
                return reply.attr(&TTL, &attr);
            },
        };
    }

    fn setattr(
        &self,
        _req: &fuser::Request,
        ino: fuser::INodeNo,
        mode: Option<u32>,
        _uid: Option<u32>,
        _gid: Option<u32>,
        size: Option<u64>,
        atime: Option<fuser::TimeOrNow>,
        mtime: Option<fuser::TimeOrNow>,
        _ctime: Option<std::time::SystemTime>,
        fh: Option<fuser::FileHandle>,
        _crtime: Option<std::time::SystemTime>,
        _chgtime: Option<std::time::SystemTime>,
        _bkuptime: Option<std::time::SystemTime>,
        _flags: Option<fuser::BsdFileFlags>,
        reply: fuser::ReplyAttr,
    ) {
        let mut fs = self.0.lock();
        if fs.abort.is_aborted() {
            return reply.error(fuser::Errno::EIO);
        }

        tracing::debug!("setattr(ino: {ino:x?}, mode: {mode:?}, size: {size:?}, fh: {fh:?})");

        let file = if let Some(fh) = fh && let Some(data) = fs.file_handle_data.get(&fh.0) && let Some(fd) = &data.underlying_fd {
            if !data.mutable {
                return reply.error(fuser::Errno::EACCES);
            }
            fd.clone()
        } else {
            match fs.files.open_ino(ino, OFlags::RDWR) {
                Ok(file) => file,
                Err(errno) => return reply.error(errno),
            }
        };

        // Mode
        if let Some(mode) = mode {
            if let Err(errno) = rustix::fs::fchmod(&file, rustix::fs::Mode::from_raw_mode(mode as _)) {
                return reply.error(to_fuser_errno(errno));
            }
        }

        // uid and gid are ignored

        // Size
        if let Some(size) = size {
            if let Err(errno) = rustix::fs::ftruncate(&file, size) {
                return reply.error(to_fuser_errno(errno));
            }
        }

        // Times
        if atime.is_some() || mtime.is_some() {
            let times = rustix::fs::Timestamps {
                last_access: time_to_timespec(atime),
                last_modification: time_to_timespec(mtime),
            };
            if let Err(errno) = rustix::fs::futimens(&file, &times) {
                return reply.error(to_fuser_errno(errno));
            }
        }

        let stat = match rustix::fs::fstat(file) {
            Ok(stat) => stat,
            Err(errno) => return reply.error(to_fuser_errno(errno)),
        };

        let attr = &fs.create_file_attr(ino, stat);
        return reply.attr(&TTL, attr);
    }

    fn readlink(&self, _req: &fuser::Request, ino: fuser::INodeNo, reply: fuser::ReplyData) {
        let mut fs = self.0.lock();
        if fs.abort.is_aborted() {
            return reply.error(fuser::Errno::EIO);
        }

        tracing::debug!("readlink(ino: {ino:x?})");

        let (parent, name) = match fs.files.open_parent_ino(ino, false) {
            Ok(parent) => parent,
            Err(errno) => return reply.error(errno),
        };

        match rustix::fs::readlinkat(parent, &*name, std::mem::take(&mut fs.reuse_buffer)) {
            Ok(target) => {
                reply.data(target.as_bytes_with_nul());
                fs.reuse_buffer = target.into_bytes_with_nul();
                return;
            },
            Err(errno) => return reply.error(to_fuser_errno(errno)),
        }
    }

    fn mknod(
        &self,
        _req: &fuser::Request,
        parent: fuser::INodeNo,
        name: &std::ffi::OsStr,
        mode: u32,
        _umask: u32,
        _rdev: u32,
        reply: fuser::ReplyEntry,
    ) {
        let mode = check_mode(mode);

        let mut fs = self.0.lock();
        if fs.abort.is_aborted() {
            return reply.error(fuser::Errno::EIO);
        }

        tracing::debug!("mknod(parent: {parent:x?}, name: {name:?}, mode: {mode:#o})");

        let file_type = rustix::fs::FileType::from_raw_mode(mode as _);
        let mode = rustix::fs::Mode::from_raw_mode(mode as _);

        if file_type == rustix::fs::FileType::Unknown {
            return reply.error(fuser::Errno::EINVAL);
        }

        let parent_fd = match fs.files.open_ino_for_child(parent, name, true) {
            Ok(parent) => parent,
            Err(errno) => return reply.error(errno),
        };

        let result = match file_type {
            rustix::fs::FileType::RegularFile => {
                let flags = O_DEFAULT | OFlags::RDONLY | OFlags::EXCL | OFlags::CREATE;
                rustix::fs::openat(parent_fd, name, flags, mode).map(|_| ())
            },
            rustix::fs::FileType::Directory => {
                rustix::fs::mkdirat(parent_fd, name, mode)
            },
            _ => {
                #[cfg(target_vendor = "apple")]
                return reply.error(fuser::Errno::EINVAL);
                #[cfg(not(target_vendor = "apple"))]
                rustix::fs::mknodat(parent, name, file_type, mode, rdev as _)
            }
        };

        if let Err(errno) = result {
            return reply.error(fuser::Errno::from_i32(errno.raw_os_error()));
        }

        match fs.do_lookup(parent, name, true) {
            Ok((attr, generation)) => reply.entry(&TTL, &attr, generation),
            Err(errno) => reply.error(errno),
        }
    }

    fn mkdir(
        &self,
        _req: &fuser::Request,
        parent: fuser::INodeNo,
        name: &std::ffi::OsStr,
        mode: u32,
        _umask: u32,
        reply: fuser::ReplyEntry,
    ) {
        let mode = check_mode(mode);

        let mut fs = self.0.lock();
        if fs.abort.is_aborted() {
            return reply.error(fuser::Errno::EIO);
        }

        tracing::debug!("mkdir(parent: {parent:x?}, name: {name:?}, mode: {mode:#o})");

        let file_type = rustix::fs::FileType::from_raw_mode(mode as _);
        let mode = rustix::fs::Mode::from_raw_mode(mode as _);

        if file_type != rustix::fs::FileType::Directory {
            return reply.error(fuser::Errno::EINVAL);
        }

        let parent_fd = match fs.files.open_ino_for_child(parent, name, true) {
            Ok(parent) => parent,
            Err(errno) => return reply.error(errno),
        };

        if let Err(errno) = rustix::fs::mkdirat(parent_fd, name, mode) {
            return reply.error(to_fuser_errno(errno));
        }

        match fs.do_lookup(parent, name, true) {
            Ok((attr, generation)) => reply.entry(&TTL, &attr, generation),
            Err(errno) => reply.error(errno),
        }
    }

    fn unlink(&self, _req: &fuser::Request, parent: fuser::INodeNo, name: &std::ffi::OsStr, reply: fuser::ReplyEmpty) {
        let mut fs = self.0.lock();
        if fs.abort.is_aborted() {
            return reply.error(fuser::Errno::EIO);
        }

        tracing::debug!("unlink(parent: {parent:x?}, name: {name:?})");

        let parent_fd = match fs.files.open_ino_for_child(parent, name, true) {
            Ok(parent) => parent,
            Err(errno) => return reply.error(errno),
        };

        // SYMLINK_NOFOLLOW isn't needed, unlinkat will delete symlinks by default
        if let Err(errno) = rustix::fs::unlinkat(parent_fd, name, AtFlags::empty()) {
            return reply.error(to_fuser_errno(errno));
        }

        match fs.files.try_remove_ino_child(parent, name) {
            Ok(_) => return reply.ok(),
            Err(errno) => return reply.error(errno),
        }
    }

    fn rmdir(&self, _req: &fuser::Request, parent: fuser::INodeNo, name: &std::ffi::OsStr, reply: fuser::ReplyEmpty) {
        let mut fs = self.0.lock();
        if fs.abort.is_aborted() {
            return reply.error(fuser::Errno::EIO);
        }

        tracing::debug!("rmdir(parent: {parent:x?}, name: {name:?})");

        let parent_fd = match fs.files.open_ino_for_child(parent, name, true) {
            Ok(parent) => parent,
            Err(errno) => return reply.error(errno),
        };

        // SYMLINK_NOFOLLOW isn't needed, unlinkat will delete symlinks by default
        if let Err(errno) = rustix::fs::unlinkat(parent_fd, name, AtFlags::REMOVEDIR) {
            return reply.error(to_fuser_errno(errno));
        }

        match fs.files.try_remove_ino_child(parent, name) {
            Ok(_) => return reply.ok(),
            Err(errno) => return reply.error(errno),
        }
    }

    fn symlink(
        &self,
        _req: &fuser::Request,
        parent: fuser::INodeNo,
        link_name: &std::ffi::OsStr,
        target: &std::path::Path,
        reply: fuser::ReplyEntry,
    ) {
        let mut fs = self.0.lock();
        if fs.abort.is_aborted() {
            return reply.error(fuser::Errno::EIO);
        }

        tracing::debug!("symlink(parent: {parent:x?}, link_name: {link_name:?}, target: {target:?})");

        let parent_fd = match fs.files.open_ino_for_child(parent, link_name, true) {
            Ok(parent) => parent,
            Err(errno) => return reply.error(errno),
        };

        if let Err(errno) = rustix::fs::symlinkat(target, parent_fd, link_name) {
            return reply.error(to_fuser_errno(errno));
        }

        match fs.do_lookup(parent, link_name, true) {
            Ok((attr, generation)) => reply.entry(&TTL, &attr, generation),
            Err(errno) => reply.error(errno),
        }
    }

    fn rename(
        &self,
        _req: &fuser::Request,
        parent: fuser::INodeNo,
        name: &std::ffi::OsStr,
        newparent: fuser::INodeNo,
        newname: &std::ffi::OsStr,
        fuser_flags: fuser::RenameFlags,
        reply: fuser::ReplyEmpty,
    ) {
        let mut fs = self.0.lock();
        if fs.abort.is_aborted() {
            return reply.error(fuser::Errno::EIO);
        }

        tracing::debug!("rename(parent: {parent:x?}, name: {name:?}, newparent: {newparent:x?}, newname: {newname:?}, flags: {fuser_flags:?})");

        if newname == "." || newname == ".." {
            // This should already be checked at the kernel level, but we
            // do it here as well just to be sure
            return reply.error(fuser::Errno::EINVAL);
        }

        if parent == newparent && name == newname {
            return reply.ok();
        }

        let Some(old_parent_node_type) = INodeType::from_ino(parent) else {
            return reply.error(fuser::Errno::ENOENT);
        };
        let Some(new_parent_node_type) = INodeType::from_ino(newparent) else {
            return reply.error(fuser::Errno::ENOENT);
        };

        let mut rustix_flags = RenameFlags::from_bits_truncate(fuser_flags.bits());

        // If renaming to a root file, make sure we don't clobber any sibling
        // files that we don't have access to
        if matches!(new_parent_node_type, INodeType::HexFolder(_)) {
            rustix_flags |= RenameFlags::NOREPLACE;
        }

        let old_parent_fd = match fs.files.open_ino_for_child(parent, name, true) {
            Ok(old_parent_fd) => old_parent_fd,
            Err(errno) => return reply.error(errno),
        };

        // Get from_node_index before the rename
        let from_node_index = match old_parent_node_type {
            INodeType::Root => return,
            INodeType::HexFolder(file_node_index) => { // child of hex folder must be file itself
                file_node_index
            },
            INodeType::FileNode(parent_file_node_index) => {
                match fs.files.get_or_create_child(parent_file_node_index, name, false) {
                    Ok((from_node_index, _, _)) => from_node_index,
                    Err(errno) => return reply.error(errno)
                }
            },
        };

        // Handle simple case for renaming a file within the same directory
        if parent == newparent {
            let node = match fs.files.get_node_mut(from_node_index) {
                Ok(node) => node,
                Err(errno) => return reply.error(errno),
            };

            let result = rustix::fs::renameat_with(&old_parent_fd, name, &old_parent_fd, newname, rustix_flags);
            if let Err(errno) = result {
                return reply.error(to_fuser_errno(errno));
            }

            node.filename = newname.into();

            return reply.ok();
        }

        let new_parent_fd = match fs.files.open_ino_for_child(newparent, newname, true) {
            Ok(new_parent_fd) => new_parent_fd,
            Err(errno) => return reply.error(errno),
        };

        let result = rustix::fs::renameat_with(old_parent_fd, name, new_parent_fd, newname, rustix_flags);
        if let Err(errno) = result {
            return reply.error(to_fuser_errno(errno));
        }

        // Get to_node_index after the rename
        let to_node_index = match new_parent_node_type {
            INodeType::Root => {
                fs.abort("rename: new parent was the root node, this shouldn't be possible");
                return reply.error(fuser::Errno::EIO);
            },
            INodeType::HexFolder(to_node_index) => { // child of hex folder must be file itself
                to_node_index
            },
            INodeType::FileNode(to_parent_node_index) => {
                match fs.files.get_or_create_child(to_parent_node_index, newname, false) {
                    Ok((to_node_index, _, _)) => to_node_index,
                    Err(errno) => {
                        // Error encountered, let's just remove everything so we don't
                        // leave ourselves in an inconsistent state
                        _ = fs.files.try_remove_ino_child(parent, name);
                        _ = fs.files.try_remove_ino_child(newparent, newname);
                        return reply.error(errno);
                    }
                }
            },
        };

        // Apply side effects of the rename to our data structures
        if from_node_index == to_node_index {
            // Technically this shouldn't be possible since the move
            // should result in to_node_index being a new node, but
            // we handle it here anyways
            if let Ok(node) = fs.files.get_node_mut(to_node_index) {
                node.filename = newname.into();
            };
        } else {
            if from_node_index.0 >= fs.files.nodes.len() {
                fs.abort("rename: old_file_node_index was larger than node vec");
                return reply.error(fuser::Errno::EIO);
            }
            if to_node_index.0 >= fs.files.nodes.len() {
                fs.abort("rename: new_file_node_index was larger than node vec");
                return reply.error(fuser::Errno::EIO);
            }

            if let Err(errno) = fs.files.try_remove_ino_child(parent, name) {
                fs.abort("rename: unable to remove old node");
                return reply.error(errno);
            }

            // Move the new node to the previous node to maintain inode
            let Ok([previous_node, new_node]) = fs.files.nodes.get_disjoint_mut([from_node_index.0, to_node_index.0]) else {
                fs.abort("rename: from_node_index == to_node_index even though we should have checked this earlier");
                return reply.error(fuser::Errno::EIO);
            };

            if previous_node.is_some() {
                fs.abort("rename: previous_node was still some even after calling remove_node");
                return reply.error(fuser::Errno::EIO);
            }

            // Move new_node to previous_node, and replace new_node with None
            std::mem::swap(previous_node, new_node);

            let Some(node) = previous_node else { // previous_node now points to the new node
                fs.abort("rename: node was none after being swapped with new_node");
                return reply.error(fuser::Errno::EIO);
            };

            node.own_index = from_node_index;
            match &mut node.inner {
                FileNodeInner::RootFile(root_file_node) => {
                    let hex = root_file_node.hex;
                    let Some(index_in_parent_hex) = fs.files.hex_to_node_index.get_mut(&hex) else {
                        fs.abort("rename: missing index in hex map");
                        return reply.error(fuser::Errno::EIO);
                    };
                    if *index_in_parent_hex != to_node_index {
                        fs.abort("rename: wrong node index in hex map");
                        return reply.error(fuser::Errno::EIO);
                    }
                    *index_in_parent_hex = from_node_index;
                },
                FileNodeInner::Subfile(subfile_node) => {
                    let parent_index = subfile_node.parent;
                    let Ok(parent_node) = fs.files.get_node_mut(parent_index) else {
                        fs.abort("rename: unable to find parent");
                        return reply.error(fuser::Errno::EIO);
                    };
                    let Some(index_in_parent_subfiles) = parent_node.subfiles.get_mut(newname) else {
                        fs.abort("rename: missing entry in parent's subfiles map");
                        return reply.error(fuser::Errno::EIO);
                    };
                    if *index_in_parent_subfiles != to_node_index {
                        fs.abort("rename: wrong node index in parent's subfile map");
                        return reply.error(fuser::Errno::EIO);
                    }
                    *index_in_parent_subfiles = from_node_index;
                },
            }
        }

        reply.ok();
    }

    fn link(
        &self,
        _req: &fuser::Request,
        ino: fuser::INodeNo,
        newparent: fuser::INodeNo,
        newname: &std::ffi::OsStr,
        reply: fuser::ReplyEntry,
    ) {
        let mut fs = self.0.lock();
        if fs.abort.is_aborted() {
            return reply.error(fuser::Errno::EIO);
        }

        tracing::debug!("link(ino: {ino:x?}, newparent: {newparent:x?}, newname: {newname:?})");

        let (old_dirfd, old_path) = match fs.files.open_parent_ino(ino, true) {
            Ok(parent) => parent,
            Err(errno) => return reply.error(errno),
        };

        let new_dirfd = match fs.files.open_ino_for_child(ino, newname, true) {
            Ok(new_dirfd) => new_dirfd,
            Err(errno) => return reply.error(errno),
        };

        if let Err(errno) = rustix::fs::linkat(old_dirfd, &*old_path, new_dirfd, newname, AtFlags::SYMLINK_NOFOLLOW) {
            return reply.error(fuser::Errno::from_i32(errno.raw_os_error()));
        }

        match fs.do_lookup(newparent, newname, true) {
            Ok((attr, generation)) => reply.entry(&TTL, &attr, generation),
            Err(errno) => reply.error(errno),
        }
    }

    fn open(&self, _req: &fuser::Request, ino: fuser::INodeNo, flags: fuser::OpenFlags, reply: fuser::ReplyOpen) {
        let mut fs = self.0.lock();
        if fs.abort.is_aborted() {
            return reply.error(fuser::Errno::EIO);
        }

        tracing::debug!("open(ino: {ino:x?}, flags: {flags:?})");

        let Some(inode_type) = INodeType::from_ino(ino) else {
            return reply.error(fuser::Errno::ENOENT);
        };

        let underlying_fd = match inode_type {
            INodeType::Root | INodeType::HexFolder(_) => {
                None
            },
            INodeType::FileNode(_) => {
                let fd = match fs.files.open_ino(ino, OFlags::from_bits_truncate(flags.0 as _)) {
                    Ok(fd) => fd,
                    Err(errno) => return reply.error(errno),
                };

                Some(fd)
            },
        };

        let file_handle_data = FileHandleData {
            opened_by: FileHandleOpenedBy::Open,
            underlying_fd,
            underlying_dir_entries: None,
            mutable: flags.acc_mode() != fuser::OpenAccMode::O_RDONLY
        };

        let file_handle = fs.push_file_handle_data(file_handle_data);
        reply.opened(file_handle, fuser::FopenFlags::empty());
    }

    fn read(
        &self,
        _req: &fuser::Request,
        ino: fuser::INodeNo,
        fh: fuser::FileHandle,
        offset: u64,
        size: u32,
        flags: fuser::OpenFlags,
        lock_owner: Option<fuser::LockOwner>,
        reply: fuser::ReplyData,
    ) {
        let mut fs = self.0.lock();
        if fs.abort.is_aborted() {
            return reply.error(fuser::Errno::EIO);
        }

        tracing::debug!(
            "read(ino: {ino:x?}, fh: {fh}, offset: {offset}, \
            size: {size}, flags: {flags:#x?}, lock_owner: {lock_owner:?})"
        );

        let Some(data) = fs.file_handle_data.get(&fh.0) else {
            return reply.error(fuser::Errno::EBADF);
        };

        let Some(fd) = data.underlying_fd.clone() else {
            return reply.error(fuser::Errno::EBADF);
        };

        if fs.reuse_buffer.len() < size as usize + 1 {
            fs.reuse_buffer.resize(size as usize + 1, 0);
        }

        let num_bytes = match rustix::io::pread(fd, fs.reuse_buffer.as_mut_slice(), offset) {
            Ok(num_bytes) => num_bytes,
            Err(errno) => return reply.error(to_fuser_errno(errno)),
        };

        reply.data(&fs.reuse_buffer[..num_bytes]);
    }

    fn write(
        &self,
        _req: &fuser::Request,
        ino: fuser::INodeNo,
        fh: fuser::FileHandle,
        offset: u64,
        data: &[u8],
        write_flags: fuser::WriteFlags,
        flags: fuser::OpenFlags,
        lock_owner: Option<fuser::LockOwner>,
        reply: fuser::ReplyWrite,
    ) {
        let fs = self.0.lock();
        if fs.abort.is_aborted() {
            return reply.error(fuser::Errno::EIO);
        }

        tracing::debug!(
            "write(ino: {ino:x?}, fh: {fh}, offset: {offset}, \
            data.len(): {}, write_flags: {write_flags:#x?}, flags: {flags:#x?}, \
            lock_owner: {lock_owner:?})",
            data.len()
        );

        let Some(fh_data) = fs.file_handle_data.get(&fh.0) else {
            return reply.error(fuser::Errno::EBADF);
        };

        let Some(fd) = fh_data.underlying_fd.clone() else {
            return reply.error(fuser::Errno::EBADF);
        };

        let num_bytes = match rustix::io::pwrite(fd, data, offset) {
            Ok(num_bytes) => num_bytes,
            Err(errno) => return reply.error(to_fuser_errno(errno)),
        };

        reply.written(num_bytes as u32);
    }

    fn flush(
        &self,
        _req: &fuser::Request,
        ino: fuser::INodeNo,
        fh: fuser::FileHandle,
        lock_owner: fuser::LockOwner,
        reply: fuser::ReplyEmpty,
    ) {
        tracing::debug!("flush(ino: {ino:x?}, fh: {fh}, lock_owner: {lock_owner:?})");

        // locking isn't supported, so we don't have anything to do
        reply.ok();
    }

    fn release(
        &self,
        _req: &fuser::Request,
        ino: fuser::INodeNo,
        fh: fuser::FileHandle,
        flags: fuser::OpenFlags,
        _lock_owner: Option<fuser::LockOwner>,
        _flush: bool,
        reply: fuser::ReplyEmpty,
    ) {
        let mut fs = self.0.lock();
        if fs.abort.is_aborted() {
            return reply.error(fuser::Errno::EIO);
        }

        tracing::debug!("release(ino: {ino:x?}, fh: {fh}, flags: {flags:?})");

        if let Some(data) = fs.file_handle_data.get(&fh.0) {
            if data.opened_by == FileHandleOpenedBy::Open {
                fs.file_handle_data.remove(&fh.0);
            } else {
                tracing::warn!("release was called on a file handle opened by opendir()");
                return;
            }
        }

        reply.ok();
    }

    fn fsync(
        &self,
        _req: &fuser::Request,
        ino: fuser::INodeNo,
        fh: fuser::FileHandle,
        datasync: bool,
        reply: fuser::ReplyEmpty,
    ) {
        let fs = self.0.lock();
        if fs.abort.is_aborted() {
            return reply.error(fuser::Errno::EIO);
        }

        tracing::debug!("fsync(ino: {ino:x?}, fh: {fh}, datasync: {datasync})");

        let Some(fh_data) = fs.file_handle_data.get(&fh.0) else {
            return reply.error(fuser::Errno::EBADF);
        };

        let Some(fd) = fh_data.underlying_fd.clone() else {
            return reply.error(fuser::Errno::EBADF);
        };

        if fh_data.opened_by != FileHandleOpenedBy::Open {
            return reply.error(fuser::Errno::EBADF);
        }

        #[cfg(target_vendor = "apple")]
        let result = rustix::fs::fsync(fd);
        #[cfg(not(target_vendor = "apple"))]
        let result = if datasync {
            rustix::fs::fdatasync(fd)
        } else {
            rustix::fs::fsync(fd)
        };

        match result {
            Ok(_) => reply.ok(),
            Err(errno) => reply.error(to_fuser_errno(errno)),
        }
    }

    fn opendir(&self, _req: &fuser::Request, ino: fuser::INodeNo, flags: fuser::OpenFlags, reply: fuser::ReplyOpen) {
        let mut fs = self.0.lock();
        if fs.abort.is_aborted() {
            return reply.error(fuser::Errno::EIO);
        }

        tracing::debug!("opendir(ino: {ino:x?}, flags: {flags:?})");

        let Some(inode_type) = INodeType::from_ino(ino) else {
            return reply.error(fuser::Errno::ENOENT);
        };

        if flags.acc_mode() != fuser::OpenAccMode::O_RDONLY {
            return reply.error(fuser::Errno::EISDIR);
        }

        let (underlying_fd, underlying_dir_entries) = match inode_type {
            INodeType::Root => {
                let dir_entries = UnderlyingDirEntries::HexFolders {
                    hex_folders: fs.files.hex_to_node_index.iter().map(|(k, v)| (*k, *v)).collect()
                };
                (None, dir_entries)
            },
            INodeType::HexFolder(file_node_index) => {
                let node = match fs.files.get_node(file_node_index) {
                    Ok(node) => node,
                    Err(errno) => return reply.error(errno),
                };
                let dir_entries = UnderlyingDirEntries::Single {
                    filename: node.filename.clone(),
                    file_node_index
                };
                (None, dir_entries)
            },
            INodeType::FileNode(file_node_index) => {
                let fd = match fs.files.open_ino(ino, OFlags::from_bits_truncate(flags.0 as _)) {
                    Ok(fd) => fd,
                    Err(errno) => return reply.error(errno),
                };

                let dir = match rustix::fs::Dir::read_from(&fd) {
                    Ok(dir) => dir,
                    Err(errno) => return reply.error(to_fuser_errno(errno)),
                };

                let dir_entries = UnderlyingDirEntries::Backed {
                    dir,
                    last_offset: 0,
                    parent: file_node_index
                };

                (Some(fd), dir_entries)
            },
        };

        let file_handle_data = FileHandleData {
            opened_by: FileHandleOpenedBy::Opendir,
            underlying_fd,
            underlying_dir_entries: Some(underlying_dir_entries),
            mutable: false,
        };

        let file_handle = fs.push_file_handle_data(file_handle_data);
        reply.opened(file_handle, fuser::FopenFlags::empty());
    }

    fn readdir(
        &self,
        _req: &fuser::Request,
        ino: fuser::INodeNo,
        fh: fuser::FileHandle,
        offset: u64,
        mut reply: fuser::ReplyDirectory,
    ) {
        let mut fs = self.0.lock();
        if fs.abort.is_aborted() {
            return reply.error(fuser::Errno::EIO);
        }

        tracing::debug!("readdir(ino: {ino:x?}, fh: {fh:?}, offset: {offset})");

        if let Err(errno) = fs.do_read_dir(ino, fh, offset, &mut reply) {
            return reply.error(errno);
        } else {
            return reply.ok();
        }
    }

    fn readdirplus(
        &self,
        _req: &fuser::Request,
        ino: fuser::INodeNo,
        fh: fuser::FileHandle,
        offset: u64,
        reply: fuser::ReplyDirectoryPlus,
    ) {
        tracing::warn!("[Not Implemented] readdirplus(ino: {ino:x?}, fh: {fh}, offset: {offset})");
        reply.error(fuser::Errno::ENOSYS);
    }

    fn releasedir(
        &self,
        _req: &fuser::Request,
        ino: fuser::INodeNo,
        fh: fuser::FileHandle,
        flags: fuser::OpenFlags,
        reply: fuser::ReplyEmpty,
    ) {
        let mut fs = self.0.lock();
        if fs.abort.is_aborted() {
            return reply.error(fuser::Errno::EIO);
        }

        tracing::debug!("releasedir(ino: {ino:x?}, fh: {fh}, flags: {flags:?})");

        if let Some(data) = fs.file_handle_data.get(&fh.0) {
            if data.opened_by == FileHandleOpenedBy::Opendir {
                fs.file_handle_data.remove(&fh.0);
            } else {
                tracing::warn!("releasedir was called on a file handle opened by open()");
            }
        }

        reply.ok();
    }

    fn fsyncdir(
        &self,
        _req: &fuser::Request,
        ino: fuser::INodeNo,
        fh: fuser::FileHandle,
        datasync: bool,
        reply: fuser::ReplyEmpty,
    ) {
        let fs = self.0.lock();
        if fs.abort.is_aborted() {
            return reply.error(fuser::Errno::EIO);
        }

        tracing::debug!("fsyncdir(ino: {ino:x?}, fh: {fh}, datasync: {datasync})");

        let Some(fh_data) = fs.file_handle_data.get(&fh.0) else {
            return reply.error(fuser::Errno::EBADF);
        };

        let Some(fd) = fh_data.underlying_fd.clone() else {
            // Nothing to sync for our synthetic directories
            return reply.ok();
        };

        if fh_data.opened_by != FileHandleOpenedBy::Opendir {
            return reply.error(fuser::Errno::EBADF);
        }

        #[cfg(target_vendor = "apple")]
        let result = rustix::fs::fsync(fd);
        #[cfg(not(target_vendor = "apple"))]
        let result = if datasync {
            rustix::fs::fdatasync(fd)
        } else {
            rustix::fs::fsync(fd)
        };

        match result {
            Ok(_) => reply.ok(),
            Err(errno) => reply.error(to_fuser_errno(errno)),
        }
    }

    fn statfs(&self, _req: &fuser::Request, _ino: fuser::INodeNo, reply: fuser::ReplyStatfs) {
        reply.statfs(0, 0, 0, 0, 0, 512, 255, 0);
    }

    fn setxattr(
        &self,
        _req: &fuser::Request,
        ino: fuser::INodeNo,
        name: &std::ffi::OsStr,
        _value: &[u8],
        flags: i32,
        position: u32,
        reply: fuser::ReplyEmpty,
    ) {
        tracing::warn!(
            "[Not Implemented] setxattr(ino: {ino:x?}, name: {name:?}, \
            flags: {flags:#x?}, position: {position})"
        );
        reply.error(fuser::Errno::ENOSYS);
    }

    fn getxattr(&self, _req: &fuser::Request, ino: fuser::INodeNo, name: &std::ffi::OsStr, size: u32, reply: fuser::ReplyXattr) {
        tracing::warn!("[Not Implemented] getxattr(ino: {ino:x?}, name: {name:?}, size: {size})");
        reply.error(fuser::Errno::ENOSYS);
    }

    fn listxattr(&self, _req: &fuser::Request, ino: fuser::INodeNo, size: u32, reply: fuser::ReplyXattr) {
        tracing::warn!("[Not Implemented] listxattr(ino: {ino:x?}, size: {size})");
        reply.error(fuser::Errno::ENOSYS);
    }

    fn removexattr(&self, _req: &fuser::Request, ino: fuser::INodeNo, name: &std::ffi::OsStr, reply: fuser::ReplyEmpty) {
        tracing::warn!("[Not Implemented] removexattr(ino: {ino:x?}, name: {name:?})");
        reply.error(fuser::Errno::ENOSYS);
    }

    fn access(&self, _req: &fuser::Request, _ino: fuser::INodeNo, _mask: fuser::AccessFlags, reply: fuser::ReplyEmpty) {
        // We use fuser::MountOption::DefaultPermissions so this isn't necessary
        return reply.error(fuser::Errno::ENOSYS);
    }

    fn create(
        &self,
        _req: &fuser::Request,
        parent: fuser::INodeNo,
        name: &std::ffi::OsStr,
        mode: u32,
        umask: u32,
        flags: i32,
        reply: fuser::ReplyCreate,
    ) {
        let mode = check_mode(mode);

        let mut fs = self.0.lock();
        if fs.abort.is_aborted() {
            return reply.error(fuser::Errno::EIO);
        }

        tracing::debug!(
            "create(parent: {parent:x?}, name: {name:?}, mode: {mode}, \
            umask: {umask:#x?}, flags: {flags:#x?})"
        );

        let file_type = rustix::fs::FileType::from_raw_mode(mode as _);
        let mode = rustix::fs::Mode::from_raw_mode(mode as _);

        if file_type != rustix::fs::FileType::RegularFile {
            return reply.error(fuser::Errno::EINVAL);
        }

        let parent_fd = match fs.files.open_ino_for_child(parent, name, true) {
            Ok(parent) => parent,
            Err(errno) => return reply.error(errno),
        };

        let flags = O_DEFAULT | OFlags::CREATE | OFlags::from_bits_truncate(flags as _);
        let fh = match rustix::fs::openat(parent_fd, name, flags, mode) {
            Ok(fd) => fs.push_file_handle_data(FileHandleData {
                opened_by: FileHandleOpenedBy::Open,
                underlying_fd: Some(Arc::new(fd)),
                underlying_dir_entries: None,
                mutable: (flags & OFlags::ACCMODE) != OFlags::RDONLY
            }),
            Err(errno) => return reply.error(to_fuser_errno(errno)),
        };

        match fs.do_lookup(parent, name, true) {
            Ok((attr, generation)) => reply.created(&TTL, &attr, generation, fh, fuser::FopenFlags::empty()),
            Err(errno) => reply.error(errno),
        }
    }

    fn getlk(
        &self,
        _req: &fuser::Request,
        _ino: fuser::INodeNo,
        _fh: fuser::FileHandle,
        _lock_owner: fuser::LockOwner,
        _start: u64,
        _end: u64,
        _typ: i32,
        _pid: u32,
        reply: fuser::ReplyLock,
    ) {
        reply.error(fuser::Errno::ENOSYS);
    }

    fn setlk(
        &self,
        _req: &fuser::Request,
        _ino: fuser::INodeNo,
        _fh: fuser::FileHandle,
        _lock_owner: fuser::LockOwner,
        _start: u64,
        _end: u64,
        _typ: i32,
        _pid: u32,
        _sleep: bool,
        reply: fuser::ReplyEmpty,
    ) {
        reply.error(fuser::Errno::ENOSYS);
    }

    fn bmap(&self, _req: &fuser::Request, _ino: fuser::INodeNo, _blocksize: u32, _idx: u64, reply: fuser::ReplyBmap) {
        reply.error(fuser::Errno::ENOSYS);
    }

    fn ioctl(
        &self,
        _req: &fuser::Request,
        _ino: fuser::INodeNo,
        _fh: fuser::FileHandle,
        _flags: fuser::IoctlFlags,
        _cmd: u32,
        _in_data: &[u8],
        _out_size: u32,
        reply: fuser::ReplyIoctl,
    ) {
        reply.error(fuser::Errno::ENOSYS);
    }

    fn poll(
        &self,
        _req: &fuser::Request,
        _ino: fuser::INodeNo,
        _fh: fuser::FileHandle,
        _ph: fuser::PollNotifier,
        _events: fuser::PollEvents,
        _flags: fuser::PollFlags,
        reply: fuser::ReplyPoll,
    ) {
        reply.error(fuser::Errno::ENOSYS);
    }

    fn fallocate(
        &self,
        _req: &fuser::Request,
        _ino: fuser::INodeNo,
        _fh: fuser::FileHandle,
        _offset: u64,
        _length: u64,
        _mode: i32,
        reply: fuser::ReplyEmpty,
    ) {
        reply.error(fuser::Errno::ENOSYS);
    }

    fn lseek(
        &self,
        _req: &fuser::Request,
        ino: fuser::INodeNo,
        fh: fuser::FileHandle,
        offset: i64,
        whence: i32,
        reply: fuser::ReplyLseek,
    ) {
        let fs = self.0.lock();
        if fs.abort.is_aborted() {
            return reply.error(fuser::Errno::EIO);
        }

        tracing::debug!(
            "lseek(ino: {ino:x?}, fh: {fh}, \
            offset: {offset}, whence: {whence})"
        );

        let Some(data) = fs.file_handle_data.get(&fh.0) else {
            return reply.error(fuser::Errno::EBADF);
        };

        let Some(fd) = data.underlying_fd.clone() else {
            return reply.error(fuser::Errno::EBADF);
        };

        let seek_from = match whence {
            libc::SEEK_SET => rustix::fs::SeekFrom::Start(offset as _),
            libc::SEEK_CUR => rustix::fs::SeekFrom::Current(offset as _),
            libc::SEEK_END => rustix::fs::SeekFrom::End(offset as _),
            libc::SEEK_DATA => rustix::fs::SeekFrom::Data(offset as _),
            libc::SEEK_HOLE => rustix::fs::SeekFrom::Hole(offset as _),
            _ => {
                return reply.error(fuser::Errno::EINVAL);
            }
        };

        let new_offset = match rustix::fs::seek(fd, seek_from) {
            Ok(num_bytes) => num_bytes,
            Err(errno) => return reply.error(to_fuser_errno(errno)),
        };

        reply.offset(new_offset as _);
    }

    #[cfg(not(target_os = "linux"))]
    fn copy_file_range(
        &self,
        _req: &fuser::Request,
        _ino_in: fuser::INodeNo,
        _fh_in: fuser::FileHandle,
        _offset_in: u64,
        _ino_out: fuser::INodeNo,
        _fh_out: fuser::FileHandle,
        _offset_out: u64,
        _len: u64,
        _flags: fuser::CopyFileRangeFlags,
        reply: fuser::ReplyWrite,
    ) {
        reply.error(fuser::Errno::ENOSYS);
    }

    #[cfg(target_os = "linux")]
    fn copy_file_range(
        &self,
        _req: &fuser::Request,
        ino_in: fuser::INodeNo,
        fh_in: fuser::FileHandle,
        offset_in: u64,
        ino_out: fuser::INodeNo,
        fh_out: fuser::FileHandle,
        offset_out: u64,
        len: u64,
        flags: fuser::CopyFileRangeFlags,
        reply: fuser::ReplyWrite,
    ) {
        let fs = self.0.lock();
        if fs.abort.is_aborted() {
            return reply.error(fuser::Errno::EIO);
        }

        tracing::debug!(
            "copy_file_range(ino_in: {ino_in:x?}, fh_in: {fh_in}, \
            offset_in: {offset_in}, ino_out: {ino_out:x?}, fh_out: {fh_out}, \
            offset_out: {offset_out}, len: {len}, flags: {flags:?})"
        );

        let Some(fh_data_in) = fs.file_handle_data.get(&fh_in.0) else {
            return reply.error(fuser::Errno::EBADF);
        };

        let Some(fd_in) = fh_data_in.underlying_fd.clone() else {
            return reply.error(fuser::Errno::EBADF);
        };

        let Some(fh_data_out) = fs.file_handle_data.get(&fh_out.0) else {
            return reply.error(fuser::Errno::EBADF);
        };

        let Some(fd_out) = fh_data_out.underlying_fd.clone() else {
            return reply.error(fuser::Errno::EBADF);
        };

        let result = rustix::fs::copy_file_range(
            fd_in,
            Some(&mut offset_in),
            fd_out,
            Some(&mut offset_out),
            len
        );
        let num_bytes = match result {
            Ok(num_bytes) => num_bytes,
            Err(errno) => return reply.error(to_fuser_errno(errno)),
        };

        reply.written(num_bytes as _);
    }

    fn setvolname(&self, _req: &fuser::Request, _name: &std::ffi::OsStr, reply: fuser::ReplyEmpty) {
        reply.error(fuser::Errno::ENOSYS);
    }

    fn exchange(
        &self,
        _req: &fuser::Request,
        _parent: fuser::INodeNo,
        _name: &std::ffi::OsStr,
        _newparent: fuser::INodeNo,
        _newname: &std::ffi::OsStr,
        _options: u64,
        reply: fuser::ReplyEmpty,
    ) {
        reply.error(fuser::Errno::ENOSYS);
    }

    fn getxtimes(&self, _req: &fuser::Request, _ino: fuser::INodeNo, reply: fuser::ReplyXTimes) {
        reply.error(fuser::Errno::ENOSYS);
    }
}

fn i64_to_systemtime(time: i64) -> SystemTime {
    if time > 0 {
        SystemTime::UNIX_EPOCH + Duration::from_secs(time.abs().max(i64::MIN + 1) as u64)
    } else {
        SystemTime::UNIX_EPOCH - Duration::from_secs(time.abs().max(i64::MIN + 1) as u64)
    }
}

fn time_to_timespec(time_or_now: Option<fuser::TimeOrNow>) -> rustix::fs::Timespec {
    match time_or_now {
        Some(fuser::TimeOrNow::SpecificTime(system_time)) => {
            let Ok(since) = system_time.duration_since(SystemTime::UNIX_EPOCH) else {
                return rustix::fs::Timespec { tv_sec: 0, tv_nsec: rustix::fs::UTIME_OMIT };
            };
            rustix::fs::Timespec { tv_sec: since.as_secs() as _, tv_nsec: since.subsec_nanos() as _ }
        },
        Some(fuser::TimeOrNow::Now) => rustix::fs::Timespec { tv_sec: 0, tv_nsec: rustix::fs::UTIME_NOW },
        None => rustix::fs::Timespec { tv_sec: 0, tv_nsec: rustix::fs::UTIME_OMIT },
    }
}

fn should_remove_on_error(err: fuser::Errno) -> bool {
    err == fuser::Errno::ENOENT || err == fuser::Errno::ENOTDIR || err == fuser::Errno::ENAMETOOLONG
}

fn to_fuser_errno(errno: rustix::io::Errno) -> fuser::Errno {
    fuser::Errno::from_i32(errno.raw_os_error())
}

fn to_fuser_filetype(file_type: rustix::fs::FileType) -> fuser::FileType {
    match file_type {
        rustix::fs::FileType::RegularFile => fuser::FileType::RegularFile,
        rustix::fs::FileType::Directory => fuser::FileType::Directory,
        rustix::fs::FileType::Symlink => fuser::FileType::Symlink,
        rustix::fs::FileType::Fifo => fuser::FileType::NamedPipe,
        rustix::fs::FileType::Socket => fuser::FileType::Socket,
        rustix::fs::FileType::CharacterDevice => fuser::FileType::CharDevice,
        rustix::fs::FileType::BlockDevice => fuser::FileType::BlockDevice,
        rustix::fs::FileType::Unknown => fuser::FileType::RegularFile, // Pretend any unknown file is a regular file
    }
}

fn check_mode(mut mode: u32) -> u32 {
    mode &= !(libc::S_ISUID as u32);
    mode &= !(libc::S_ISGID as u32);
    mode
}
