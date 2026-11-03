//! Handles tracking which file paths we've given the sandbox access to, in a
//! global per-container [`AppliedAclLedger`].
//!
//! For each AppContainer we make, we give it a registry key at
//! `HKCU\Software\MinecraftSandbox\<SID>`, under the name `Grant`. When we give
//! the sandbox permission to access a specific path, we add it to the ACL, and
//! record it in the ledger; when we want to reset the sandbox's file access,
//! we can read the ledger to see what access we previously gave it, and revoke
//! those.
//!
//! The reason we need a ledger is because Windows doesn't let us list all files
//! which we've allowed to an AppContainer via ACL. Instead, it works the other
//! way around, where individual files/directories list what AppContainers are
//! allowed to read/write/traverse them. It would be impractical to scan every
//! single file on the disk to see if our container can access it, so instead we
//! keep track of it ourselves.
//!
//! We record grants in the ledger before adding them to the ACL, and revoke
//! grants from the ACL before removing them from the ledger. This means that:
//! - all ACL grants **must** be in the ledger
//! - all grants in the ledger **may** also exist in the ACL
//!
//! The first one is the important part, since we need to ensure that we can
//! always clean up ACL entries after ourselves; and addition/removal of a
//! grant or ACL entry is idempotent, so it's OK to remove or add a path that's
//! already removed/added.
//!
//! While reading/writing the ledger, we take a global Windows mutex keyed to
//! that container, to make sure that reads/writes won't conflict with another
//! process trying to read/write to the same ledger.
//!
//! When we spawn an instance, we keep track of what permissions it has in the
//! ledger, and key those grants by the instance we've just made[^1]. We
//! also prune the ledger - find any entries keyed by a process which is now
//! dead, and remove it. This lets up clean up the grants list automatically.
//!
//! ACL grants are per-container, **not** per process. So all processes in the
//! same AppContainer sandbox share the same access. If you spawn a new process,
//! and it has less permissions than a process that we spawned before, then we
//! wouldn't want that old process to suddenly stop working since it can no
//! longer access some files; to solve this, the real ACL that we apply is the
//! *union* of all grants of all instances. Since instances get cleaned up
//! later, revoking a grant still works.
//!
//! [^1]: When we're preparing to spawn an instance, we give the ledger entry a
//! UUID; after we've actually spawned the process, *then* we put the PID and
//! process creation time into the ledger (both of these let us uniquely
//! identify the process, since Windows can reuse PIDs).

use std::collections::{BTreeMap, HashSet};
use std::io;
use std::os::windows::io::{AsRawHandle, OwnedHandle};
use std::path::Path;
use std::{ffi::OsStr, os::windows::ffi::OsStrExt};

use eyre::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use windows::{
    Win32::{
        Foundation::{
            CloseHandle, E_INVALIDARG, FILETIME, HANDLE, WAIT_ABANDONED,
            WAIT_FAILED, WAIT_OBJECT_0, WAIT_TIMEOUT,
        },
        System::Threading::{
            CreateMutexW, GetCurrentProcess, GetCurrentProcessId,
            GetProcessTimes, INFINITE, OpenProcess, PROCESS_ACCESS_RIGHTS,
            PROCESS_QUERY_LIMITED_INFORMATION, ReleaseMutex,
            WaitForSingleObject,
        },
    },
    core::PCWSTR,
};
use winreg::{RegKey, enums::HKEY_CURRENT_USER};

const REGISTRY_BASE: &str = r"Software\MinecraftSandbox";
const GRANT_VALUE: &str = "Grant";
const SYNCHRONIZE_PROCESS: PROCESS_ACCESS_RIGHTS =
    PROCESS_ACCESS_RIGHTS(0x00100000);

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionType {
    /// Allows access through the named directory without inheriting the grant
    /// to any of its children.
    Traverse,
    Read,
    Write,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Grant {
    pub path: String,
    pub permission: PermissionType,
}

impl Grant {
    pub fn from_path(path: &Path, permission: PermissionType) -> Result<Self> {
        let path = path.canonicalize()?;
        let path = path.to_str().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("ACL path is not valid Unicode: {path:?}"),
            )
        })?;
        Ok(Self {
            path: path.to_owned(),
            permission,
        })
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(super) struct ProcessIdentity {
    pid: u32,
    creation_time: u64,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct RegistryState {
    #[serde(default)]
    instances: Vec<Instance>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Instance {
    #[serde(with = "uuid_string")]
    id: Uuid,
    #[serde(flatten)]
    state: InstanceState,
    grants: Vec<Grant>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
enum InstanceState {
    Pending { process: ProcessIdentity },
    Running { process: ProcessIdentity },
}

impl InstanceState {
    fn process(&self) -> ProcessIdentity {
        match self {
            Self::Pending { process } | Self::Running { process } => *process,
        }
    }
}

pub struct AppliedAclLedger {
    key: RegKey,
    mutex_name: Vec<u16>,
}

impl AppliedAclLedger {
    pub fn open(app_container_sid: &OsStr) -> Result<Self> {
        let sid = app_container_sid.to_str().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "AppContainer SID is not valid Unicode",
            )
        })?;
        let key_path = format!(r"{REGISTRY_BASE}\{sid}");
        let (key, _) =
            RegKey::predef(HKEY_CURRENT_USER).create_subkey(key_path)?;
        let mutex_name =
            OsStr::new(&format!(r"Global\MinecraftSandboxAclLedger-{sid}"))
                .encode_wide()
                .chain([0])
                .collect();
        Ok(Self { key, mutex_name })
    }

    /// Locks the global Windows mutex, allowing safe access to the sandbox ACL
    /// ledger.
    pub fn lock(&self) -> Result<LockedAclLedger<'_>> {
        let mutex = LedgerMutexGuard::acquire(&self.mutex_name)?;
        LockedAclLedger::open(&self.key, mutex)
    }

    pub fn add_grants(
        &self,
        instance_id: Uuid,
        grants: Vec<Grant>,
    ) -> Result<Vec<Grant>> {
        let mut ledger = self.lock()?;
        let old_effective = effective_union(&ledger.state, |_| true);
        let instance = ledger
            .state
            .instances
            .iter_mut()
            .find(|instance| instance.id == instance_id)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "active ACL instance not found",
                )
            })?;
        instance.grants.extend(grants);
        instance.grants = normalize_grants(instance.grants.iter());
        let desired_effective = effective_union(&ledger.state, |_| true);
        let changed = desired_effective
            .iter()
            .filter(|desired| {
                old_effective
                    .iter()
                    .find(|old| paths_equal(&old.path, &desired.path))
                    .is_none_or(|old| {
                        permission_strength(desired.permission)
                            > permission_strength(old.permission)
                    })
            })
            .cloned()
            .collect();
        ledger.persist()?;
        Ok(changed)
    }
}

pub struct LockedAclLedger<'a> {
    key: &'a RegKey,
    state: RegistryState,
    _mutex: LedgerMutexGuard,
}

impl<'a> LockedAclLedger<'a> {
    fn open(key: &'a RegKey, mutex: LedgerMutexGuard) -> Result<Self> {
        let state: String = match key.get_value(GRANT_VALUE) {
            Ok(state) => state,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(Self {
                    key,
                    state: RegistryState::default(),
                    _mutex: mutex,
                });
            }
            Err(error) => return Err(error.into()),
        };
        let state = serde_json::from_str(&state).map_err(|error| {
            io::Error::new(io::ErrorKind::InvalidData, error)
        })?;
        Ok(Self {
            key,
            state,
            _mutex: mutex,
        })
    }

    /// Removes stale instances, and prepares the ledger to add a new instance.
    pub fn prepare(
        mut self,
        grants: Vec<Grant>,
    ) -> Result<PreparedInstance<'a>> {
        let process = ProcessIdentity::current()?;
        let grants = normalize_grants(grants.iter());
        let old_effective = effective_union(&self.state, |_| true);
        let mut dead_instances = HashSet::new();
        for instance in &self.state.instances {
            if !instance.state.process().is_alive()? {
                dead_instances.insert(instance.id);
            }
        }
        let id = Uuid::now_v7();
        self.state.instances.push(Instance {
            id,
            state: InstanceState::Pending { process },
            grants,
        });
        let desired_effective = effective_union(&self.state, |instance| {
            !dead_instances.contains(&instance.id)
        });
        self.persist()?;
        Ok(PreparedInstance {
            ledger: self,
            id,
            dead_instances,
            old_effective,
            desired_effective,
        })
    }

    fn persist(&self) -> Result<()> {
        let state = serde_json::to_string(&self.state).map_err(|error| {
            io::Error::new(io::ErrorKind::InvalidData, error)
        })?;
        self.key.set_value(GRANT_VALUE, &state)?;
        Ok(())
    }
}

pub struct PreparedInstance<'a> {
    ledger: LockedAclLedger<'a>,
    id: Uuid,
    dead_instances: HashSet<Uuid>,
    old_effective: Vec<Grant>,
    desired_effective: Vec<Grant>,
}

impl PreparedInstance<'_> {
    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn old_effective(&self) -> &[Grant] {
        &self.old_effective
    }

    pub fn desired_effective(&self) -> &[Grant] {
        &self.desired_effective
    }

    pub fn finalize_pruning(&mut self) -> Result<()> {
        self.ledger
            .state
            .instances
            .retain(|instance| !self.dead_instances.contains(&instance.id));
        self.dead_instances.clear();
        self.ledger.persist()
    }

    pub fn activate(&mut self, process: ProcessIdentity) -> Result<()> {
        let instance = self.instance_mut()?;
        instance.state = InstanceState::Running { process };
        self.ledger.persist()
    }

    fn instance_mut(&mut self) -> Result<&mut Instance> {
        Ok(self
            .ledger
            .state
            .instances
            .iter_mut()
            .find(|instance| instance.id == self.id)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "pending ACL instance not found",
                )
            })?)
    }
}

impl ProcessIdentity {
    fn current() -> Result<Self> {
        Self::from_handle(unsafe { GetCurrentProcessId() }, unsafe {
            GetCurrentProcess()
        })
    }

    pub(super) fn from_handle(pid: u32, handle: HANDLE) -> Result<Self> {
        Ok(Self {
            pid,
            creation_time: process_creation_time(handle)?,
        })
    }

    fn is_alive(self) -> Result<bool> {
        let handle = match unsafe {
            OpenProcess(
                PROCESS_QUERY_LIMITED_INFORMATION | SYNCHRONIZE_PROCESS,
                false,
                self.pid,
            )
        } {
            Ok(handle) => handle,
            Err(error) if error.code() == E_INVALIDARG => return Ok(false),
            Err(error) => {
                return Err(io::Error::other(error.to_string()).into());
            }
        };
        let handle = ProcessHandle(handle);
        if process_creation_time(handle.0)? != self.creation_time {
            return Ok(false);
        }
        match unsafe { WaitForSingleObject(handle.0, 0) } {
            WAIT_TIMEOUT => Ok(true),
            WAIT_OBJECT_0 => Ok(false),
            WAIT_FAILED => Err(io::Error::last_os_error().into()),
            wait => Err(io::Error::other(format!(
                "unexpected process wait result: {wait:?}"
            ))
            .into()),
        }
    }
}

fn process_creation_time(handle: HANDLE) -> Result<u64> {
    let mut creation = FILETIME::default();
    let mut exit = FILETIME::default();
    let mut kernel = FILETIME::default();
    let mut user = FILETIME::default();
    unsafe {
        GetProcessTimes(
            handle,
            &mut creation,
            &mut exit,
            &mut kernel,
            &mut user,
        )?;
    }
    Ok((u64::from(creation.dwHighDateTime) << 32)
        | u64::from(creation.dwLowDateTime))
}

fn effective_union<'a>(
    state: &'a RegistryState,
    include_instance: impl Fn(&Instance) -> bool,
) -> Vec<Grant> {
    normalize_grants(
        state
            .instances
            .iter()
            .filter(|instance| include_instance(instance))
            .flat_map(|instance| instance.grants.iter()),
    )
}

pub fn paths_equal(first: &str, second: &str) -> bool {
    path_key(first) == path_key(second)
}

fn path_key(path: &str) -> String {
    path.to_lowercase()
}

fn normalize_grants<'a>(
    grants: impl IntoIterator<Item = &'a Grant>,
) -> Vec<Grant> {
    let mut normalized = BTreeMap::<String, Grant>::new();
    for grant in grants {
        let key = path_key(&grant.path);
        match normalized.get_mut(&key) {
            Some(existing)
                if permission_strength(grant.permission)
                    > permission_strength(existing.permission) =>
            {
                existing.permission = grant.permission;
            }
            Some(_) => {}
            None => {
                normalized.insert(key, grant.clone());
            }
        }
    }
    normalized.into_values().collect()
}

fn permission_strength(permission: PermissionType) -> u8 {
    match permission {
        PermissionType::Traverse => 0,
        PermissionType::Read => 1,
        PermissionType::Write => 2,
    }
}

struct ProcessHandle(HANDLE);

impl Drop for ProcessHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

struct LedgerMutexGuard {
    handle: OwnedHandle,
}

impl LedgerMutexGuard {
    fn acquire(name: &[u16]) -> Result<Self> {
        let handle =
            unsafe { CreateMutexW(None, false, PCWSTR(name.as_ptr())) }?;
        let handle = super::to_owned_handle(handle)
            .map_err(|error| io::Error::other(error.to_string()))?;
        let wait = unsafe {
            WaitForSingleObject(HANDLE(handle.as_raw_handle()), INFINITE)
        };
        if wait != WAIT_OBJECT_0 && wait != WAIT_ABANDONED {
            if wait == WAIT_FAILED {
                return Err(io::Error::last_os_error().into());
            }
            return Err(io::Error::other(format!(
                "unexpected mutex wait result: {wait:?}"
            ))
            .into());
        }
        Ok(Self { handle })
    }
}

impl Drop for LedgerMutexGuard {
    fn drop(&mut self) {
        let _ = unsafe { ReleaseMutex(HANDLE(self.handle.as_raw_handle())) };
    }
}

mod uuid_string {
    use serde::{Deserialize, Deserializer, Serializer};
    use uuid::Uuid;

    pub fn serialize<S>(
        uuid: &Uuid,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&uuid.to_string())
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> std::result::Result<Uuid, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Uuid::parse_str(&value).map_err(serde::de::Error::custom)
    }
}
