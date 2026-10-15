use std::collections::{BTreeMap, HashSet};
use std::io;
use std::os::windows::io::{AsRawHandle, OwnedHandle};
use std::path::Path;
use std::{ffi::OsStr, os::windows::ffi::OsStrExt};

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
    pub fn from_path(
        path: &Path,
        permission: PermissionType,
    ) -> io::Result<Self> {
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
    pub fn open(app_container_sid: &OsStr) -> io::Result<Self> {
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

    pub fn lock(&self) -> io::Result<RegistryStateGuard<'_>> {
        let mutex = LedgerMutexGuard::acquire(&self.mutex_name)?;
        let state = self.read()?;
        Ok(RegistryStateGuard {
            ledger: self,
            state,
            _mutex: mutex,
        })
    }

    pub fn prepare(
        &self,
        grants: Vec<Grant>,
    ) -> io::Result<PreparedInstance<'_>> {
        let guard = self.lock()?;
        guard.prepare(grants)
    }

    fn read(&self) -> io::Result<RegistryState> {
        let state: String = match self.key.get_value(GRANT_VALUE) {
            Ok(state) => state,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(RegistryState::default());
            }
            Err(error) => return Err(error),
        };
        serde_json::from_str(&state)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    }

    fn write(&self, state: &RegistryState) -> io::Result<()> {
        let state = serde_json::to_string(state).map_err(|error| {
            io::Error::new(io::ErrorKind::InvalidData, error)
        })?;
        self.key.set_value(GRANT_VALUE, &state)
    }
}

pub struct RegistryStateGuard<'a> {
    ledger: &'a AppliedAclLedger,
    state: RegistryState,
    _mutex: LedgerMutexGuard,
}

impl<'a> RegistryStateGuard<'a> {
    pub fn prepare(
        mut self,
        grants: Vec<Grant>,
    ) -> io::Result<PreparedInstance<'a>> {
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
            guard: self,
            id,
            dead_instances,
            old_effective,
            desired_effective,
        })
    }

    fn persist(&self) -> io::Result<()> {
        self.ledger.write(&self.state)
    }
}

pub struct PreparedInstance<'a> {
    guard: RegistryStateGuard<'a>,
    id: Uuid,
    dead_instances: HashSet<Uuid>,
    old_effective: Vec<Grant>,
    desired_effective: Vec<Grant>,
}

impl PreparedInstance<'_> {
    pub fn old_effective(&self) -> &[Grant] {
        &self.old_effective
    }

    pub fn desired_effective(&self) -> &[Grant] {
        &self.desired_effective
    }

    pub fn finalize_pruning(&mut self) -> io::Result<()> {
        self.guard
            .state
            .instances
            .retain(|instance| !self.dead_instances.contains(&instance.id));
        self.dead_instances.clear();
        self.guard.persist()
    }

    pub fn activate(&mut self, process: ProcessIdentity) -> io::Result<()> {
        let instance = self.instance_mut()?;
        instance.state = InstanceState::Running { process };
        self.guard.persist()
    }

    fn instance_mut(&mut self) -> io::Result<&mut Instance> {
        self.guard
            .state
            .instances
            .iter_mut()
            .find(|instance| instance.id == self.id)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "pending ACL instance not found",
                )
            })
    }
}

impl ProcessIdentity {
    fn current() -> io::Result<Self> {
        Self::from_handle(unsafe { GetCurrentProcessId() }, unsafe {
            GetCurrentProcess()
        })
    }

    pub(super) fn from_handle(pid: u32, handle: HANDLE) -> io::Result<Self> {
        Ok(Self {
            pid,
            creation_time: process_creation_time(handle)?,
        })
    }

    fn is_alive(self) -> io::Result<bool> {
        let handle = match unsafe {
            OpenProcess(
                PROCESS_QUERY_LIMITED_INFORMATION | SYNCHRONIZE_PROCESS,
                false,
                self.pid,
            )
        } {
            Ok(handle) => handle,
            Err(error) if error.code() == E_INVALIDARG => return Ok(false),
            Err(error) => return Err(io::Error::other(error.to_string())),
        };
        let handle = ProcessHandle(handle);
        if process_creation_time(handle.0)? != self.creation_time {
            return Ok(false);
        }
        match unsafe { WaitForSingleObject(handle.0, 0) } {
            WAIT_TIMEOUT => Ok(true),
            WAIT_OBJECT_0 => Ok(false),
            WAIT_FAILED => Err(io::Error::last_os_error()),
            wait => Err(io::Error::other(format!(
                "unexpected process wait result: {wait:?}"
            ))),
        }
    }
}

fn process_creation_time(handle: HANDLE) -> io::Result<u64> {
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
    fn acquire(name: &[u16]) -> io::Result<Self> {
        let handle =
            unsafe { CreateMutexW(None, false, PCWSTR(name.as_ptr())) }?;
        let handle = super::to_owned_handle(handle)
            .map_err(|error| io::Error::other(error.to_string()))?;
        let wait = unsafe {
            WaitForSingleObject(HANDLE(handle.as_raw_handle()), INFINITE)
        };
        if wait != WAIT_OBJECT_0 && wait != WAIT_ABANDONED {
            if wait == WAIT_FAILED {
                return Err(io::Error::last_os_error());
            }
            return Err(io::Error::other(format!(
                "unexpected mutex wait result: {wait:?}"
            )));
        }
        Ok(Self { handle })
    }
}

impl Drop for LedgerMutexGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = ReleaseMutex(HANDLE(self.handle.as_raw_handle()));
        }
    }
}

mod uuid_string {
    use serde::{Deserialize, Deserializer, Serializer};
    use uuid::Uuid;

    pub fn serialize<S>(uuid: &Uuid, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&uuid.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Uuid::parse_str(&value).map_err(serde::de::Error::custom)
    }
}
