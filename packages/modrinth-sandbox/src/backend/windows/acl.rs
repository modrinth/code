use std::os::windows::io::AsRawHandle;
use std::{io, path::Path};

use serde::{Deserialize, Serialize};
use windows::{
    Win32::{
        Foundation::{WAIT_ABANDONED, WAIT_FAILED, WAIT_OBJECT_0},
        System::Threading::{
            CreateMutexW, INFINITE, ReleaseMutex, WaitForSingleObject,
        },
    },
    core::{PCWSTR, w},
};
use winreg::{RegKey, enums::HKEY_CURRENT_USER};

const REGISTRY_KEY: &str = r"Software\MinecraftSandbox";
const GRANTS_VALUE: &str = "Grants";
const MUTEX_NAME: PCWSTR = w!(r"Local\MinecraftSandboxAclLedger");

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionType {
    /// Allows access through the named directory without inheriting the grant
    /// to any of its children.
    Traverse,
    Read,
    Write,
}

#[derive(Deserialize, Eq, PartialEq, Serialize)]
struct AppliedGrant {
    path: String,
    permission: PermissionType,
}

pub struct AppliedAclLedger {
    key: RegKey,
}

impl AppliedAclLedger {
    pub fn open() -> io::Result<Self> {
        let (key, _) =
            RegKey::predef(HKEY_CURRENT_USER).create_subkey(REGISTRY_KEY)?;
        Ok(Self { key })
    }

    pub fn record(
        &self,
        path: &Path,
        permission: PermissionType,
    ) -> io::Result<()> {
        let _guard = LedgerMutexGuard::acquire()?;
        let mut grants = self.read()?;
        let path = path.to_str().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("ACL path is not valid Unicode: {path:?}"),
            )
        })?;
        let grant = AppliedGrant {
            path: path.to_owned(),
            permission,
        };
        if grants.contains(&grant) {
            return Ok(());
        }

        grants.push(grant);
        let grants = serde_json::to_string(&grants).map_err(|error| {
            io::Error::new(io::ErrorKind::InvalidData, error)
        })?;
        self.key.set_value(GRANTS_VALUE, &grants)
    }

    fn read(&self) -> io::Result<Vec<AppliedGrant>> {
        let grants: String = match self.key.get_value(GRANTS_VALUE) {
            Ok(grants) => grants,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(Vec::new());
            }
            Err(error) => return Err(error),
        };
        serde_json::from_str(&grants)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    }
}

struct LedgerMutexGuard {
    handle: std::os::windows::io::OwnedHandle,
}

impl LedgerMutexGuard {
    fn acquire() -> io::Result<Self> {
        let handle = unsafe { CreateMutexW(None, false, MUTEX_NAME) }?;
        let handle = super::to_owned_handle(handle)
            .map_err(|error| io::Error::other(error.to_string()))?;
        let wait = unsafe {
            WaitForSingleObject(
                windows::Win32::Foundation::HANDLE(handle.as_raw_handle()),
                INFINITE,
            )
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
            let _ = ReleaseMutex(windows::Win32::Foundation::HANDLE(
                self.handle.as_raw_handle(),
            ));
        }
    }
}
