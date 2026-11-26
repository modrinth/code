use std::{
    collections::HashSet,
    ffi::{OsStr, OsString},
    fs::OpenOptions,
    io::{Error, ErrorKind},
    os::windows::{
        ffi::{OsStrExt, OsStringExt},
        fs::OpenOptionsExt,
    },
    path::{Path, PathBuf},
    sync::Arc,
};

use tracing::info;
use windows::{
    Win32::{
        Foundation::{
            ERROR_ACCESS_DENIED, ERROR_ALREADY_EXISTS, ERROR_FILE_NOT_FOUND,
            ERROR_INSUFFICIENT_BUFFER, ERROR_PATH_NOT_FOUND, ERROR_SUCCESS,
            GetLastError, HANDLE, LocalFree,
        },
        NetworkManagement::WindowsFirewall::{
            NetworkIsolationGetAppContainerConfig,
            NetworkIsolationSetAppContainerConfig,
        },
        Security::{
            ACCESS_ALLOWED_ACE, ACCESS_DENIED_ACE, ACE_HEADER, ACL,
            Authorization::{
                ConvertSidToStringSidW, ConvertStringSidToSidW,
                EXPLICIT_ACCESS_W, GRANT_ACCESS, GetNamedSecurityInfoW,
                GetSecurityInfo, NO_MULTIPLE_TRUSTEE, REVOKE_ACCESS,
                SE_FILE_OBJECT, SE_OBJECT_TYPE, SE_REGISTRY_KEY,
                SE_WINDOW_OBJECT, SET_ACCESS, SetEntriesInAclW,
                SetNamedSecurityInfoW, SetSecurityInfo, TRUSTEE_IS_GROUP,
                TRUSTEE_IS_SID, TRUSTEE_W,
            },
            CONTAINER_INHERIT_ACE, CreateWellKnownSid,
            DACL_SECURITY_INFORMATION, DeriveCapabilitySidsFromName, EqualSid,
            FreeSid, GetAce, GetSecurityDescriptorControl, INHERITED_ACE,
            InitializeSecurityDescriptor,
            Isolation::{
                CreateAppContainerProfile,
                DeriveAppContainerSidFromAppContainerName,
                GetAppContainerFolderPath,
            },
            NO_INHERITANCE, OBJECT_INHERIT_ACE, PSECURITY_DESCRIPTOR, PSID,
            SE_DACL_AUTO_INHERIT_REQ, SE_DACL_AUTO_INHERITED,
            SE_DACL_PROTECTED, SECURITY_CAPABILITIES,
            SECURITY_DESCRIPTOR_CONTROL, SID_AND_ATTRIBUTES, SetFileSecurityW,
            SetSecurityDescriptorControl, SetSecurityDescriptorDacl,
            WELL_KNOWN_SID_TYPE, WinCapabilityInternetClientServerSid,
            WinCapabilityInternetClientSid,
            WinCapabilityPrivateNetworkClientServerSid,
        },
        Storage::FileSystem::{
            DELETE, FILE_FLAG_BACKUP_SEMANTICS, FILE_GENERIC_EXECUTE,
            FILE_GENERIC_READ, FILE_GENERIC_WRITE, FILE_TRAVERSE, READ_CONTROL,
            WRITE_DAC,
        },
        System::{
            Com::CoTaskMemFree,
            Memory::{GetProcessHeap, HEAP_FLAGS, HeapFree},
            Registry::KEY_READ,
            StationsAndDesktops::OpenWindowStationW,
            SystemServices::{
                ACCESS_ALLOWED_ACE_TYPE, ACCESS_DENIED_ACE_TYPE,
                SE_GROUP_ENABLED, SECURITY_DESCRIPTOR_REVISION,
            },
            Threading::PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES,
        },
        UI::WindowsAndMessaging::WINSTA_WRITEATTRIBUTES,
    },
    core::{HRESULT, PCWSTR, PWSTR},
};

use anyhow::{Context, Result, anyhow, bail, ensure};
use async_trait::async_trait;
use derive_more::Debug;
use parking_lot::Mutex;
use uuid::Uuid;

use crate::{
    SandboxArg, SandboxCommand, SandboxExitStatus,
    backend::{
        Backend, ExposedPath, SandboxChild, SandboxChildOp, SandboxEnv,
        SandboxHandle, SandboxHandleOp, pathset::ReadWritePathSet,
    },
    helper::MakeHelper,
    path_access::{PathPermission, PathRule, PathRules},
};

use super::{
    WindowsChild,
    acl::{AppliedAclLedger, Grant, GrantKind, PermissionType, paths_equal},
    drag_intercept::DragInterceptHandle,
    home::link_into_sandbox_home,
};

#[derive(Debug)]
pub struct AppContainer;

#[async_trait]
impl Backend for AppContainer {
    async fn init(make_helper: MakeHelper) -> Result<Box<dyn SandboxEnv>> {
        Ok(Box::new(AppContainerEnv {
            make_helper,
            job_handle: super::create_job_object()
                .context("error creating job object")?,
            null_device: super::open_null_device()
                .context("error opening null device")?,
        }))
    }
}

#[derive(Clone, Debug)]
#[debug("AppContainerEnv")]
pub struct AppContainerEnv {
    make_helper: MakeHelper,
    job_handle: HANDLE,
    null_device: HANDLE,
}

// HANDLE can be sent across threads despite containing *mut c_void
unsafe impl Send for AppContainerEnv {}
unsafe impl Sync for AppContainerEnv {}

#[async_trait]
impl SandboxEnv for AppContainerEnv {
    async fn spawn(
        &self,
        command: SandboxCommand,
    ) -> Result<crate::SandboxChild> {
        let env = self.clone();
        crate::backend::run_on_spawn_thread(move || spawn(&env, command))
            .await?
    }
}

fn spawn(
    env: &AppContainerEnv,
    mut command: SandboxCommand,
) -> Result<crate::SandboxChild> {
    let app_container_sid = create_app_container(
        command.app_container_name.as_os_str(),
        command.app_container_description.as_os_str(),
    )?;
    scopeguard::defer! {
        unsafe { FreeSid(app_container_sid) };
    }

    let app_container_sid_string =
        app_container_sid_to_string(&app_container_sid)?;
    let acl_ledger = AppliedAclLedger::open(&app_container_sid_string)
        .context("opening AppContainer ACL ledger")?;
    let app_container_folder =
        get_app_container_folder(&app_container_sid_string)?;
    let local_state = app_container_folder.join("LocalState");
    let roaming_state = app_container_folder.join("RoamingState");
    let temp_state = app_container_folder.join("TempState");
    let sandbox_home = local_state.join("Home");
    for path in [&local_state, &roaming_state, &temp_state, &sandbox_home] {
        std::fs::create_dir_all(path).with_context(|| {
            format!("creating AppContainer storage directory {path:?}")
        })?;
    }

    // The sandbox always has access to this
    command
        .path_rules
        .push(PathRule::Unblock(app_container_folder.clone()));
    let (path_set, path_rules) = command.take_paths()?;
    let real_home = std::env::home_dir();
    let home = real_home
        .clone()
        .filter(|home| path_set.can_write(home.clone()))
        .unwrap_or_else(|| sandbox_home.clone());

    command.extra_environment.extend([
        ("APPDATA".into(), roaming_state.into()),
        ("LOCALAPPDATA".into(), local_state.into()),
        ("HOME".into(), home.clone().into()),
        ("USERPROFILE".into(), home.clone().into()),
        ("HOMEDRIVE".into(), "".into()),
        ("HOMEPATH".into(), home.clone().into()),
        ("TEMP".into(), temp_state.clone().into()),
        ("TMP".into(), temp_state.into()),
    ]);
    if command.is_jvm {
        let mut user_home = OsString::from("-Duser.home=");
        user_home.push(&home);
        command.args.insert(0, user_home.into());
    }

    let environment = command.take_environment();

    let mut owned_capabilities: Vec<OwnedCapability> = Vec::new();

    if command.allow_network {
        owned_capabilities
            .push(OwnedCapability::new(WinCapabilityInternetClientSid)?);
        owned_capabilities
            .push(OwnedCapability::new(WinCapabilityInternetClientServerSid)?);
        owned_capabilities.push(OwnedCapability::new(
            WinCapabilityPrivateNetworkClientServerSid,
        )?);
    }

    let mut capabilities = owned_capabilities
        .iter_mut()
        .map(|cap| SID_AND_ATTRIBUTES {
            Sid: cap.as_psid(),
            Attributes: SE_GROUP_ENABLED as u32,
        })
        .collect::<Vec<_>>();

    // Ideally we could just add the appcontainer's sid to winsta0 directly,
    // but for some reason this triggers a bug in chrome.
    // See https://github.com/Moulberry/PandoraLauncher/issues/415

    let mut group_sids = std::ptr::null_mut();
    let mut group_sid_count = 0;
    let mut capability_sids = std::ptr::null_mut();
    let mut capability_sid_count = 0;
    unsafe {
        DeriveCapabilitySidsFromName(
            windows::core::w!(
                "modrinth.grantWinstaWriteAttributesCapability_ikU1gY09aHdb3PsX"
            ),
            &mut group_sids,
            &mut group_sid_count,
            &mut capability_sids,
            &mut capability_sid_count,
        )?
    };

    if capability_sid_count == 0 {
        tracing::error!(
            "DeriveCapabilitySidsFromName returned 0 for capabilitysidcount"
        );
    } else if capability_sids.is_null() {
        tracing::error!(
            "DeriveCapabilitySidsFromName returned null for capability_sids"
        );
    } else {
        let derived_sid = unsafe { capability_sids.read() };
        if let Err(err) = add_writeattributes_to_winsta(&derived_sid) {
            tracing::error!("Unable to set WINSTA_WRITEATTRIBUTES: {err}");
        } else {
            capabilities.push(SID_AND_ATTRIBUTES {
                Sid: derived_sid,
                Attributes: SE_GROUP_ENABLED as u32,
            });
        }
    }

    scopeguard::defer! {
        // https://github.com/microsoft/Windows-universal-samples/blob/082195895276903b6630d5cb4d03c9d365ec210c/Samples/CustomCapability/Service/Server/RpcServer.cpp#L33
        if !group_sids.is_null() {
            for i in 0..group_sid_count {
                let group_sid = unsafe { group_sids.offset(i as isize).read() };
                unsafe { LocalFree(Some(windows::Win32::Foundation::HLOCAL(group_sid.0))) };
            }
            unsafe { LocalFree(Some(windows::Win32::Foundation::HLOCAL(group_sids.cast()))) };
        }
        if !capability_sids.is_null() {
            for i in 0..capability_sid_count {
                let group_sid = unsafe { capability_sids.offset(i as isize).read() };
                unsafe { LocalFree(Some(windows::Win32::Foundation::HLOCAL(group_sid.0))) };
            }
            unsafe { LocalFree(Some(windows::Win32::Foundation::HLOCAL(capability_sids.cast()))) };
        }
    }

    let mut security_capabilities = SECURITY_CAPABILITIES::default();
    security_capabilities.CapabilityCount = capabilities.len() as u32;
    security_capabilities.Capabilities = capabilities.as_mut_ptr();
    security_capabilities.AppContainerSid = app_container_sid;

    let mut attribute_list = Vec::new();
    attribute_list.push(super::ProcThreadAttribute {
        attribute: PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES as usize,
        value: &security_capabilities as *const SECURITY_CAPABILITIES
            as *const _,
        size: size_of::<SECURITY_CAPABILITIES>(),
    });

    for path in &command.ensure_dirs_exist {
        std::fs::create_dir_all(path)
            .with_context(|| anyhow!("creating directory {path:?}"))?;
    }

    if let Some(real_home) = &real_home
        && home == sandbox_home
    {
        for path in path_set.iter_both() {
            if let Err(err) =
                link_into_sandbox_home(path, real_home, &sandbox_home)
            {
                tracing::warn!(
                    "unable to link {path:?} into sandbox home: {err:#}"
                );
            }
        }
    }

    let mut parents_to_add: HashSet<std::path::PathBuf> = HashSet::default();
    for path in path_set.iter_both() {
        if let Some(parent) = path.parent() {
            parents_to_add.insert(parent.to_path_buf());
        }
    }

    let mut parents: HashSet<std::path::PathBuf> = HashSet::default();
    loop {
        let taken = std::mem::take(&mut parents_to_add);
        if taken.is_empty() {
            break;
        }
        for to_add in taken {
            if path_set.can_read(to_add.clone()) {
                continue;
            }
            let parent = to_add.parent().map(Path::to_path_buf);
            if parents.insert(to_add)
                && let Some(parent) = parent
            {
                parents_to_add.insert(parent);
            }
        }
    }

    // Read-only directories can contain a read-write, but not vice-versa. So we bind read then write
    let mut instance_grants = Vec::new();
    for path in path_set.iter_read_only() {
        instance_grants.extend(
            Grant::from_path(path, PermissionType::Read)
                .with_context(|| anyhow!("creating read grant for {path:?}"))?,
        );
    }
    for path in path_set.iter_read_write() {
        instance_grants.extend(
            Grant::from_path(path, PermissionType::Write).with_context(
                || anyhow!("creating write grant for {path:?}"),
            )?,
        );
    }
    for path in parents {
        instance_grants.extend(
            Grant::from_path(&path, PermissionType::Traverse).with_context(
                || anyhow!("creating traverse grant for {path:?}"),
            )?,
        );
    }

    instance_grants.extend(
        command
            .registry_read_keys
            .iter()
            .filter_map(|key| Grant::from_registry_key(key)),
    );

    let locked_acl_ledger = acl_ledger
        .lock()
        .context("locking AppContainer ACL ledger")?;
    let mut prepared = locked_acl_ledger
        .prepare(instance_grants)
        .context("preparing AppContainer ACL instance")?;
    let mut parents_to_elevate = Vec::new();
    let mut read_paths_to_elevate = Vec::new();
    let mut write_paths_to_elevate = Vec::new();
    let mut remove_paths_to_elevate = Vec::new();
    let mut not_revoked = Vec::new();

    for grant in prepared.old_effective() {
        if !prepared
            .desired_effective()
            .iter()
            .any(|desired| paths_equal(&desired.path, &grant.path))
        {
            let path = PathBuf::from(&grant.path);
            // This has to be checked before revoking, since a failure could remove it
            // from the path but not from everything inside of it
            let was_listed =
                is_grant_listed(&app_container_sid, grant).unwrap_or(true);
            if let Err(err) = revoke_grant(&app_container_sid, grant)
                && was_listed
            {
                if grant.kind == GrantKind::File && is_access_denied(&err) {
                    remove_paths_to_elevate.push(path);
                } else {
                    tracing::warn!(
                        "unable to revoke sandbox access from {path:?}: {err:#}"
                    );
                    not_revoked.push(grant.clone());
                }
            }
        }
    }

    for grant in prepared.desired_effective() {
        let path = PathBuf::from(&grant.path);
        if let Err(err) = apply_grant(&app_container_sid, grant) {
            if grant.kind == GrantKind::RegistryKey {
                tracing::warn!(
                    "unable to grant sandbox access to registry key {path:?}: {err:#}"
                );
            } else if is_access_denied(&err) {
                let mut permission = grant.permission;
                if permission == PermissionType::Write
                    && !can_access_ourselves(&path, permission)
                {
                    tracing::warn!(
                        "unable to grant sandbox write access to {path:?}, since we can't write to it ourselves, falling back to read access"
                    );
                    permission = PermissionType::Read;
                }
                if !can_access_ourselves(&path, permission) {
                    tracing::warn!(
                        "unable to grant sandbox {permission:?} access to {path:?}, since we don't have access ourselves"
                    );
                    continue;
                }
                // Otherwise the user would be prompted for every launch
                if is_already_granted(&app_container_sid, &path, permission)
                    .unwrap_or(false)
                {
                    continue;
                }
                match permission {
                    PermissionType::Traverse => parents_to_elevate.push(path),
                    PermissionType::Read => read_paths_to_elevate.push(path),
                    PermissionType::Write => write_paths_to_elevate.push(path),
                }
            } else {
                return Err(err).with_context(|| {
                    format!("granting sandbox access to {path:?}")
                });
            }
        }
    }

    let mut need_set_network_isolation_elevated = false;
    if command.allow_network {
        if let Err(err) = try_set_network_isolation(&app_container_sid) {
            if is_access_denied(&err) {
                info!(
                    "lacking permission to allow localhost access, elevating"
                );
                need_set_network_isolation_elevated = true;
            } else {
                return Err(err)
                    .context("configuring AppContainer network isolation");
            }
        }
    }

    if !parents_to_elevate.is_empty()
        || !read_paths_to_elevate.is_empty()
        || !write_paths_to_elevate.is_empty()
        || !remove_paths_to_elevate.is_empty()
        || need_set_network_isolation_elevated
    {
        let mut arguments = Vec::<SandboxArg>::new();
        arguments.push(command.app_container_name.clone());
        arguments.push(if need_set_network_isolation_elevated {
            "true".into()
        } else {
            "false".into()
        });
        arguments.push(std::env::join_paths(parents_to_elevate)?.into());
        arguments.push(std::env::join_paths(read_paths_to_elevate)?.into());
        arguments.push(std::env::join_paths(write_paths_to_elevate)?.into());
        arguments.push(std::env::join_paths(remove_paths_to_elevate)?.into());

        info!(?arguments, "spawning elevated helper command");
        let mut elevated = super::runas::spawn(env.make_helper, arguments)
            .context("spawning elevated helper command")?;
        let elevated_status = elevated
            .blocking_wait()
            .context("running elevated helper")?;
        info!("elevated helper command finished: {elevated_status}");
        anyhow::ensure!(
            elevated_status.success(),
            "elevated helper failed with {elevated_status}"
        );
    }

    prepared
        .finalize_pruning(&not_revoked)
        .context("finalizing AppContainer ACL pruning")?;

    let (pipes, child) = super::spawn(
        command.executable.clone().into(),
        std::mem::take(&mut command.args),
        environment,
        command.stdin,
        command.stdout,
        command.stderr,
        command.working_directory,
        env.job_handle.clone(),
        env.null_device.clone(),
        attribute_list,
    )
    .context("spawning child")?;
    let process_identity = child
        .process_identity()
        .context("reading spawned child process identity")?;
    let acl_instance_id = prepared.id();
    prepared
        .activate(process_identity)
        .context("activating AppContainer ACL instance")?;
    drop(prepared);

    let handle = WindowsSandboxHandle {
        path_set: Arc::new(Mutex::new(path_set)),
        path_rules: Arc::new(path_rules),
        app_container_sid: Arc::new(app_container_sid_string),
        acl_instance_id,
        expose_lock: Arc::new(Mutex::new(())),
    };
    let drag_intercept = start_drag_intercept(
        &child,
        &command.app_container_name.as_os_str().to_string_lossy(),
        handle.clone(),
    );

    let mut child = crate::SandboxChild {
        stdin: pipes.stdin,
        stdout: pipes.stdout,
        stderr: pipes.stderr,
        on_finish: command.on_finish,
        handle: SandboxHandle::AppContainer(handle),
        imp: SandboxChild::AppContainer(AppContainerChild {
            _drag_intercept: drag_intercept,
            child,
        }),
    };

    for on_start in command.on_start {
        (on_start)(&mut child);
    }

    Ok(child)
}

fn start_drag_intercept(
    child: &WindowsChild,
    name: &str,
    sandbox: WindowsSandboxHandle,
) -> Option<DragInterceptHandle> {
    match DragInterceptHandle::start(child.id()?, name, sandbox) {
        Ok(handle) => Some(handle),
        Err(err) => {
            tracing::error!("unable to start drag intercept: {err:#}");
            None
        }
    }
}

#[derive(Debug)]
pub(crate) struct AppContainerChild {
    /// Must be dropped before `child`, as its process ID can be reused after that.
    _drag_intercept: Option<DragInterceptHandle>,
    child: WindowsChild,
}

#[async_trait]
impl SandboxChildOp for AppContainerChild {
    fn id(&self) -> Option<u32> {
        self.child.id()
    }

    fn try_wait(&mut self) -> Result<Option<SandboxExitStatus>> {
        self.child.try_wait()
    }

    async fn wait(&mut self) -> Result<SandboxExitStatus> {
        self.child.wait().await
    }

    async fn kill(&mut self) -> Result<()> {
        self.child.kill().await
    }
}

#[derive(Clone)]
pub struct WindowsSandboxHandle {
    path_set: Arc<Mutex<ReadWritePathSet>>,
    path_rules: Arc<PathRules>,
    app_container_sid: Arc<OsString>,
    acl_instance_id: Uuid,
    expose_lock: Arc<Mutex<()>>,
}

impl std::fmt::Debug for WindowsSandboxHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindowsSandboxHandle")
            .finish_non_exhaustive()
    }
}

impl SandboxHandleOp for WindowsSandboxHandle {
    fn expose_path(
        &self,
        path: &Path,
        permission: PathPermission,
    ) -> anyhow::Result<ExposedPath> {
        let is_accessible = |path: &Path, permission| match permission {
            PathPermission::ReadOnly => self.is_path_readable(path),
            PathPermission::ReadWrite => self.is_path_writeable(path),
        };

        if is_accessible(path, permission) {
            return Ok(ExposedPath::Direct);
        }

        let permission =
            permission.min(self.path_rules.calculate_expose_permission(path)?);

        if permission == PathPermission::ReadWrite {
            // Try create an empty file if one doesn't exist yet
            // Save dialogs return paths which may not exist, creating
            // an empty file allows us to grant permission to that file
            // rather than the whole parent directory
            match OpenOptions::new().write(true).create_new(true).open(path) {
                Ok(_) => {}
                Err(err) if err.kind() == ErrorKind::AlreadyExists => {}
                Err(err) => {
                    return Err(err).with_context(|| {
                        format!("creating empty file {path:?}")
                    });
                }
            }
        }

        let original_path = path;
        let path = path
            .canonicalize()
            .with_context(|| format!("canonicalizing path {path:?}"))?;

        let simplified = strip_verbatim_prefix(&path);
        let exposed = if paths_equal(
            &simplified.to_string_lossy(),
            &strip_verbatim_prefix(original_path).to_string_lossy(),
        ) {
            ExposedPath::Direct
        } else {
            ExposedPath::Proxied(simplified)
        };

        // Grants on directories are inherited by everything inside them, so
        // granting a whole drive would be extremely slow and far too broad
        ensure!(
            path.parent().is_some(),
            "refusing to expose a drive root: {path:?}"
        );

        let _expose_guard = self.expose_lock.lock();

        if is_accessible(&path, permission) {
            return Ok(exposed);
        }

        let mut grants =
            vec![Grant::from_path(&path, permission.into())?.ok_or_else(
                || anyhow!("unable to canonicalize path: {path:?}"),
            )?];
        let mut parent = path.parent();
        while let Some(parent_path) = parent {
            grants.extend(Grant::from_path(
                parent_path,
                PermissionType::Traverse,
            )?);
            parent = parent_path.parent();
        }

        let acl_ledger = AppliedAclLedger::open(&self.app_container_sid)
            .context("opening AppContainer ACL ledger")?;
        let sid = self
            .app_container_sid
            .encode_wide()
            .chain([0])
            .collect::<Vec<_>>();
        let mut app_container_sid = PSID::default();
        unsafe {
            ConvertStringSidToSidW(
                PCWSTR(sid.as_ptr()),
                &mut app_container_sid,
            )?
        };
        ensure!(
            !app_container_sid.is_invalid(),
            "ConvertStringSidToSidW returned invalid SID"
        );
        scopeguard::defer! {
            unsafe { LocalFree(Some(windows::Win32::Foundation::HLOCAL(app_container_sid.0))) };
        }

        acl_ledger
            .add_grants(
                self.acl_instance_id,
                grants,
                |grant| {
                    apply_grant(&app_container_sid, grant).with_context(|| {
                        format!(
                            "granting dynamic sandbox {:?} access to {:?}",
                            grant.permission, grant.path
                        )
                    })
                },
                |grant| revoke_grant(&app_container_sid, grant),
            )
            .context("adding dynamic AppContainer ACL grants")?;

        let mut path_set = self.path_set.lock();
        match permission {
            PathPermission::ReadOnly => path_set.insert_readable(path),
            PathPermission::ReadWrite => path_set.insert_writeable(path),
        };
        Ok(exposed)
    }

    fn is_path_readable(&self, path: &Path) -> bool {
        self.path_set.lock().can_read(path.to_path_buf())
    }

    fn is_path_writeable(&self, path: &Path) -> bool {
        self.path_set.lock().can_write(path.to_path_buf())
    }

    fn is_path_blocked(&self, path: &Path) -> bool {
        self.path_rules.is_blocked(path)
    }
}

/// Some applications, including Java, don't support the verbatim prefix that
/// canonical paths have.
pub(super) fn strip_verbatim_prefix(path: &Path) -> PathBuf {
    let Some(path_str) = path.to_str() else {
        return path.to_owned();
    };
    if let Some(rest) = path_str.strip_prefix(r"\\?\UNC\") {
        PathBuf::from(format!(r"\\{rest}"))
    } else if let Some(rest) = path_str.strip_prefix(r"\\?\") {
        PathBuf::from(rest)
    } else {
        path.to_owned()
    }
}

struct OwnedCapability {
    sid_alloc: Vec<u8>,
}

impl OwnedCapability {
    pub fn new(known: WELL_KNOWN_SID_TYPE) -> Result<Self> {
        let mut size = 0;
        let result =
            unsafe { CreateWellKnownSid(known, None, None, &mut size) };
        if result
            != Err(windows::core::Error::from_hresult(HRESULT::from_win32(
                ERROR_INSUFFICIENT_BUFFER.0,
            )))
        {
            result?;
        }
        let mut sid_alloc = vec![0; size as usize];
        unsafe {
            CreateWellKnownSid(
                known,
                None,
                Some(PSID(sid_alloc.as_mut_ptr() as *mut _)),
                &mut size,
            )?
        };
        Ok(Self { sid_alloc })
    }

    pub fn as_psid(&mut self) -> PSID {
        PSID(self.sid_alloc.as_mut_ptr() as *mut _)
    }
}

fn app_container_sid_to_string(app_container_sid: &PSID) -> Result<OsString> {
    let mut string_sid = PWSTR::default();
    unsafe { ConvertSidToStringSidW(*app_container_sid, &mut string_sid)? };
    if string_sid.is_null() {
        return Err(anyhow!("ConvertSidToStringSidW returned a null SID"));
    }
    scopeguard::defer! {
        unsafe {
            LocalFree(Some(windows::Win32::Foundation::HLOCAL(
                string_sid.0.cast(),
            )))
        };
    }

    Ok(OsString::from_wide(unsafe { string_sid.as_wide() }))
}

fn get_app_container_folder(app_container_sid: &OsStr) -> Result<PathBuf> {
    let string_sid = app_container_sid
        .encode_wide()
        .chain([0])
        .collect::<Vec<_>>();
    let folder =
        unsafe { GetAppContainerFolderPath(PCWSTR(string_sid.as_ptr()))? };
    if folder.is_null() {
        return Err(anyhow!("GetAppContainerFolderPath returned a null path"));
    }
    scopeguard::defer! {
        unsafe { CoTaskMemFree(Some(folder.0.cast())) };
    }

    Ok(PathBuf::from(OsString::from_wide(unsafe {
        folder.as_wide()
    })))
}

fn create_app_container(
    name: &OsStr,
    description: &OsStr,
) -> windows::core::Result<PSID> {
    let encoded_name = name.encode_wide().chain([0]).collect::<Vec<_>>();
    let encoded_description =
        description.encode_wide().chain([0]).collect::<Vec<_>>();

    let mut result = unsafe {
        CreateAppContainerProfile(
            PCWSTR(encoded_name.as_ptr()),
            PCWSTR(encoded_name.as_ptr()),
            PCWSTR(encoded_description.as_ptr()),
            None,
        )
    };

    if result
        == Err(windows::core::Error::from_hresult(HRESULT::from_win32(
            ERROR_ALREADY_EXISTS.0,
        )))
    {
        result = unsafe {
            DeriveAppContainerSidFromAppContainerName(PCWSTR(
                encoded_name.as_ptr(),
            ))
        };
    }

    result
}

fn try_set_network_isolation(app_container: &PSID) -> Result<()> {
    let mut current_count: u32 = 0;
    let mut current_containers: *mut SID_AND_ATTRIBUTES = std::ptr::null_mut();

    let result = unsafe {
        NetworkIsolationGetAppContainerConfig(
            &mut current_count,
            &mut current_containers,
        )
    };
    scopeguard::defer! {
        // https://learn.microsoft.com/en-us/windows/win32/api/netfw/nf-netfw-networkisolationgetappcontainerconfig
        let process_heap = unsafe { GetProcessHeap() };
        if let Ok(process_heap) = process_heap {
            for index in 0..current_count {
                let sid = unsafe { current_containers.offset(index as isize).read() }.Sid;
                let _ = unsafe { HeapFree(process_heap, HEAP_FLAGS(0), Some(sid.0)) };
            }
            if !current_containers.is_null() {
                let _ = unsafe { HeapFree(process_heap, HEAP_FLAGS(0), Some(current_containers as *mut _)) };
            }
        }
    }

    if result != 0 {
        return Err(windows::core::Error::from_hresult(HRESULT::from_win32(
            result,
        ))
        .into());
    }

    let mut new_containers = Vec::with_capacity(current_count as usize + 1);
    for index in 0..current_count {
        let sid_and_attributes =
            unsafe { current_containers.offset(index as isize).read() };
        if unsafe { EqualSid(sid_and_attributes.Sid, *app_container) }.is_ok() {
            // Already allowed
            return Ok(());
        }

        new_containers.push(sid_and_attributes);
    }
    new_containers.push(SID_AND_ATTRIBUTES {
        Sid: *app_container,
        Attributes: 0,
    });

    let result = unsafe {
        NetworkIsolationSetAppContainerConfig(new_containers.as_slice())
    };
    if result != 0 {
        return Err(windows::core::Error::from_hresult(HRESULT::from_win32(
            result,
        ))
        .into());
    }

    Ok(())
}

pub(super) fn is_access_denied(err: &anyhow::Error) -> bool {
    err.downcast_ref::<std::io::Error>().is_some_and(|err| {
        err.kind() == ErrorKind::PermissionDenied
            || err.raw_os_error().unwrap_or(0)
                == HRESULT::from_win32(ERROR_ACCESS_DENIED.0).0
    }) || err
        .downcast_ref::<windows::core::Error>()
        .is_some_and(|err| {
            err.code() == HRESULT::from_win32(ERROR_ACCESS_DENIED.0)
        })
}

fn add_writeattributes_to_winsta(app_container: &PSID) -> Result<()> {
    let winsta = unsafe {
        OpenWindowStationW(
            windows::core::w!("winsta0"),
            false,
            (READ_CONTROL | WRITE_DAC).0,
        )
    }?;
    let mut old_acl = std::ptr::null_mut();
    let err = unsafe {
        GetSecurityInfo(
            HANDLE(winsta.0),
            SE_WINDOW_OBJECT,
            DACL_SECURITY_INFORMATION,
            None,
            None,
            Some(&mut old_acl),
            None,
            None,
        )
    };
    if err != ERROR_SUCCESS {
        return Err(windows::core::Error::from_hresult(HRESULT::from_win32(
            err.0,
        ))
        .into());
    }

    let mut ea = EXPLICIT_ACCESS_W::default();
    ea.grfAccessMode = GRANT_ACCESS;
    ea.grfAccessPermissions = WINSTA_WRITEATTRIBUTES as u32; // Allows use of ClipCursor / SetCursorPos / etc.
    ea.grfInheritance = NO_INHERITANCE;
    ea.Trustee.MultipleTrusteeOperation = NO_MULTIPLE_TRUSTEE;
    ea.Trustee.pMultipleTrustee = std::ptr::null_mut();
    ea.Trustee.ptstrName = windows::core::PWSTR(app_container.0 as *mut u16);
    ea.Trustee.TrusteeForm = TRUSTEE_IS_SID;
    ea.Trustee.TrusteeType = TRUSTEE_IS_GROUP;

    let mut new_acl = std::ptr::null_mut();
    let err =
        unsafe { SetEntriesInAclW(Some(&[ea]), Some(old_acl), &mut new_acl) };
    if new_acl.is_null() {
        return Err(Error::new(ErrorKind::Other, "new acl was null").into());
    }
    scopeguard::defer! {
        unsafe { LocalFree(Some(windows::Win32::Foundation::HLOCAL(new_acl as *mut _))) };
    }
    if err != ERROR_SUCCESS {
        return Err(windows::core::Error::from_hresult(HRESULT::from_win32(
            err.0,
        ))
        .into());
    }

    match acl_eq(old_acl, new_acl) {
        Ok(true) => return Ok(()),
        Ok(false) => {}
        Err(err) => {
            tracing::error!(
                "Error comparing ACL for winsta0, updating acl anyways: {err}"
            );
        }
    }

    let err = unsafe {
        SetSecurityInfo(
            HANDLE(winsta.0),
            SE_WINDOW_OBJECT,
            DACL_SECURITY_INFORMATION,
            None,
            None,
            Some(new_acl),
            None,
        )
    };
    if err != ERROR_SUCCESS {
        return Err(windows::core::Error::from_hresult(HRESULT::from_win32(
            err.0,
        ))
        .into());
    }

    Ok(())
}

fn access_rights(permission: PermissionType) -> u32 {
    const TRAVERSE: u32 = FILE_GENERIC_READ.0 | FILE_TRAVERSE.0;
    const READ: u32 = TRAVERSE | FILE_GENERIC_EXECUTE.0;
    const WRITE: u32 = READ | FILE_GENERIC_WRITE.0 | DELETE.0;

    match permission {
        PermissionType::Traverse => TRAVERSE,
        PermissionType::Read => READ,
        PermissionType::Write => WRITE,
    }
}

/// Prevents elevation from giving the sandbox access to something that the user
/// can't normally access.
fn can_access_ourselves(path: &Path, permission: PermissionType) -> bool {
    let result = OpenOptions::new()
        .access_mode(access_rights(permission))
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS.0)
        .open(path);
    // Other errors, such as the file being in use, aren't related to permissions
    !result.is_err_and(|err| err.kind() == ErrorKind::PermissionDenied)
}

struct ExplicitAce {
    ace_type: u32,
    flags: u32,
    mask: u32,
}

/// Finds the entries for an AppContainer in an ACL, other than inherited ones.
fn find_explicit_aces(
    app_container: &PSID,
    path: &Path,
    object_type: SE_OBJECT_TYPE,
) -> Result<Vec<ExplicitAce>> {
    let encoded_path = path
        .as_os_str()
        .encode_wide()
        .chain([0])
        .collect::<Vec<_>>();
    let mut acl = std::ptr::null_mut();
    let mut descriptor = PSECURITY_DESCRIPTOR::default();
    unsafe {
        GetNamedSecurityInfoW(
            PCWSTR(encoded_path.as_ptr()),
            object_type,
            DACL_SECURITY_INFORMATION,
            None,
            None,
            Some(&raw mut acl),
            None,
            &raw mut descriptor,
        )
        .ok()?;
    }
    scopeguard::defer! {
        unsafe { LocalFree(Some(windows::Win32::Foundation::HLOCAL(descriptor.0))) };
    }
    let Some(acl_ref) = (unsafe { acl.as_ref() }) else {
        return Ok(Vec::new());
    };

    let mut aces = Vec::new();
    for index in 0..acl_ref.AceCount {
        let mut ace = std::ptr::null_mut();
        unsafe { GetAce(acl, u32::from(index), &raw mut ace)? };
        let header = unsafe { ace.cast::<ACE_HEADER>().read() };
        let ace_type = u32::from(header.AceType);
        let flags = u32::from(header.AceFlags);

        if flags & INHERITED_ACE.0 != 0 {
            continue;
        }

        let (sid, mask) = match ace_type {
            ACCESS_ALLOWED_ACE_TYPE => {
                let ace = ace.cast::<ACCESS_ALLOWED_ACE>();
                unsafe { (&raw mut (*ace).SidStart, (*ace).Mask) }
            }
            ACCESS_DENIED_ACE_TYPE => {
                let ace = ace.cast::<ACCESS_DENIED_ACE>();
                unsafe { (&raw mut (*ace).SidStart, (*ace).Mask) }
            }
            _ => continue,
        };
        if unsafe { EqualSid(PSID(sid.cast()), *app_container) }.is_ok() {
            aces.push(ExplicitAce {
                ace_type,
                flags,
                mask,
            });
        }
    }
    Ok(aces)
}

/// Checks for an entry that exactly matches what [`set_acl`] would add.
fn is_already_granted(
    app_container: &PSID,
    path: &Path,
    permission: PermissionType,
) -> Result<bool> {
    let aces = find_explicit_aces(app_container, path, SE_FILE_OBJECT)?;

    // Windows removes the inheritance flags from files
    let expected_flags =
        if permission == PermissionType::Traverse || !path.is_dir() {
            0
        } else {
            (OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE).0
        };
    let expected_mask = access_rights(permission);

    Ok(!aces.is_empty()
        && aces.iter().all(|ace| {
            ace.ace_type == ACCESS_ALLOWED_ACE_TYPE
                && ace.flags == expected_flags
                && ace.mask == expected_mask
        }))
}

fn set_acl(
    app_container: &PSID,
    path: &Path,
    perms: PermissionType,
) -> Result<()> {
    let encoded_path = path
        .as_os_str()
        .encode_wide()
        .chain([0])
        .collect::<Vec<_>>();
    let mut old_acl = std::ptr::null_mut();
    let mut old_descriptor = PSECURITY_DESCRIPTOR::default();
    let err = unsafe {
        GetNamedSecurityInfoW(
            PCWSTR(encoded_path.as_ptr()),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            None,
            None,
            Some(&mut old_acl),
            None,
            &mut old_descriptor,
        )
    };
    if err != ERROR_SUCCESS {
        return Err(windows::core::Error::from_hresult(HRESULT::from_win32(
            err.0,
        ))
        .into());
    }

    scopeguard::defer! {
        unsafe { LocalFree(Some(windows::Win32::Foundation::HLOCAL(old_descriptor.0))) };
    }

    let mut ea = EXPLICIT_ACCESS_W::default();
    ea.grfAccessMode = SET_ACCESS;
    ea.grfAccessPermissions = access_rights(perms);
    ea.grfInheritance = if matches!(perms, PermissionType::Traverse) {
        NO_INHERITANCE
    } else {
        OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE
    };
    ea.Trustee.MultipleTrusteeOperation = NO_MULTIPLE_TRUSTEE;
    ea.Trustee.pMultipleTrustee = std::ptr::null_mut();
    ea.Trustee.ptstrName = windows::core::PWSTR(app_container.0 as *mut u16);
    ea.Trustee.TrusteeForm = TRUSTEE_IS_SID;
    ea.Trustee.TrusteeType = TRUSTEE_IS_GROUP;

    let mut new_acl = std::ptr::null_mut();
    let err =
        unsafe { SetEntriesInAclW(Some(&[ea]), Some(old_acl), &mut new_acl) };
    if new_acl.is_null() {
        return Err(Error::new(ErrorKind::Other, "new acl was null").into());
    }
    scopeguard::defer! {
        unsafe { LocalFree(Some(windows::Win32::Foundation::HLOCAL(new_acl as *mut _))) };
    }
    if err != ERROR_SUCCESS {
        return Err(windows::core::Error::from_hresult(HRESULT::from_win32(
            err.0,
        ))
        .into());
    }

    if ea.grfInheritance == NO_INHERITANCE {
        // Only compare ACLs if inheritance is disabled.
        // Sometimes children don't properly inherit the ACLs,
        // so we need to be sure to apply them even if the parent is correct
        match acl_eq(old_acl, new_acl) {
            Ok(true) => return Ok(()),
            Ok(false) => {}
            Err(err) => {
                tracing::error!(
                    "Error comparing ACL for {path:?}, updating acl anyways: {err}"
                );
            }
        }
    }

    let err = if ea.grfInheritance == NO_INHERITANCE {
        // We use SetFileSecurityW (obselete) instead of SetNamedSecurityInfoW (recommended)
        // because SetFileSecurityW doesn't automatically propagate changes to children.
        // Ideally SetNamedSecurityInfoW would realize that the changes don't affect the
        // children, but it's not smart enough.
        // This is necessary because propagating to all children of C:\ (for example) is extremely slow

        let mut security_buf = vec![0; 64]; // 64 is SECURITY_DESCRIPTOR_MIN_LENGTH

        unsafe {
            InitializeSecurityDescriptor(
                PSECURITY_DESCRIPTOR(security_buf.as_mut_ptr() as *mut _),
                SECURITY_DESCRIPTOR_REVISION,
            )?;
        }

        unsafe {
            SetSecurityDescriptorDacl(
                PSECURITY_DESCRIPTOR(security_buf.as_mut_ptr() as *mut _),
                true,
                Some(new_acl),
                false,
            )?;
        }

        // SetFileSecurityW clears these flags unless they're requested. Without
        // SE_DACL_AUTO_INHERITED, anything created in the directory won't inherit
        // from its new parent after being moved. Without SE_DACL_PROTECTED, the
        // directory can inherit from its parent even if the user disabled that
        let mut old_control = 0;
        let mut revision = 0;
        unsafe {
            GetSecurityDescriptorControl(
                old_descriptor,
                &mut old_control,
                &mut revision,
            )?;
        }
        let preserved = SE_DACL_AUTO_INHERITED | SE_DACL_PROTECTED;
        let mut flags = SECURITY_DESCRIPTOR_CONTROL(old_control) & preserved;
        if flags.contains(SE_DACL_AUTO_INHERITED) {
            flags |= SE_DACL_AUTO_INHERIT_REQ;
        }
        unsafe {
            SetSecurityDescriptorControl(
                PSECURITY_DESCRIPTOR(security_buf.as_mut_ptr() as *mut _),
                preserved | SE_DACL_AUTO_INHERIT_REQ,
                flags,
            )?;
        }

        unsafe {
            let success = SetFileSecurityW(
                PCWSTR(encoded_path.as_ptr()),
                DACL_SECURITY_INFORMATION,
                PSECURITY_DESCRIPTOR(security_buf.as_mut_ptr() as *mut _),
            );
            if success.as_bool() {
                ERROR_SUCCESS
            } else {
                GetLastError()
            }
        }
    } else {
        unsafe {
            SetNamedSecurityInfoW(
                PCWSTR(encoded_path.as_ptr()),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION,
                None,
                None,
                Some(new_acl),
                None,
            )
        }
    };

    if err != ERROR_SUCCESS {
        return Err(windows::core::Error::from_hresult(HRESULT::from_win32(
            err.0,
        ))
        .into());
    }

    Ok(())
}

fn set_registry_acl(app_container: &PSID, path: &Path) -> Result<()> {
    let encoded_path = path
        .as_os_str()
        .encode_wide()
        .chain([0])
        .collect::<Vec<_>>();
    let mut old_acl = std::ptr::null_mut();
    let mut old_descriptor = PSECURITY_DESCRIPTOR::default();
    unsafe {
        GetNamedSecurityInfoW(
            PCWSTR(encoded_path.as_ptr()),
            SE_REGISTRY_KEY,
            DACL_SECURITY_INFORMATION,
            None,
            None,
            Some(&raw mut old_acl),
            None,
            &raw mut old_descriptor,
        )
        .ok()?;
    }
    scopeguard::defer! {
        unsafe { LocalFree(Some(windows::Win32::Foundation::HLOCAL(old_descriptor.0))) };
    }

    let access = EXPLICIT_ACCESS_W {
        grfAccessPermissions: KEY_READ.0,
        grfAccessMode: SET_ACCESS,
        grfInheritance: NO_INHERITANCE,
        Trustee: TRUSTEE_W {
            pMultipleTrustee: std::ptr::null_mut(),
            MultipleTrusteeOperation: NO_MULTIPLE_TRUSTEE,
            TrusteeForm: TRUSTEE_IS_SID,
            TrusteeType: TRUSTEE_IS_GROUP,
            ptstrName: PWSTR(app_container.0.cast()),
        },
    };
    let mut new_acl = std::ptr::null_mut();
    unsafe {
        SetEntriesInAclW(Some(&[access]), Some(old_acl), &raw mut new_acl)
            .ok()?;
    }
    scopeguard::defer! {
        unsafe { LocalFree(Some(windows::Win32::Foundation::HLOCAL(new_acl.cast()))) };
    }

    if acl_eq(old_acl, new_acl)? {
        return Ok(());
    }
    unsafe {
        SetNamedSecurityInfoW(
            PCWSTR(encoded_path.as_ptr()),
            SE_REGISTRY_KEY,
            DACL_SECURITY_INFORMATION,
            None,
            None,
            Some(new_acl),
            None,
        )
        .ok()?;
    }
    Ok(())
}

fn apply_grant(app_container: &PSID, grant: &Grant) -> Result<()> {
    let path = Path::new(&grant.path);
    match grant.kind {
        GrantKind::File => set_acl(app_container, path, grant.permission),
        GrantKind::RegistryKey => set_registry_acl(app_container, path),
    }
}

fn revoke_grant(app_container: &PSID, grant: &Grant) -> Result<()> {
    remove_from_acl(
        app_container,
        Path::new(&grant.path),
        object_type(grant.kind),
    )
}

/// Whether there's anything for [`revoke_grant`] to remove from the path itself.
fn is_grant_listed(app_container: &PSID, grant: &Grant) -> Result<bool> {
    let aces = find_explicit_aces(
        app_container,
        Path::new(&grant.path),
        object_type(grant.kind),
    )?;
    Ok(!aces.is_empty())
}

fn object_type(kind: GrantKind) -> SE_OBJECT_TYPE {
    match kind {
        GrantKind::File => SE_FILE_OBJECT,
        GrantKind::RegistryKey => SE_REGISTRY_KEY,
    }
}

fn remove_from_acl(
    app_container: &PSID,
    path: &Path,
    object_type: SE_OBJECT_TYPE,
) -> Result<()> {
    let encoded_path = path
        .as_os_str()
        .encode_wide()
        .chain([0])
        .collect::<Vec<_>>();
    let mut old_acl = std::ptr::null_mut();
    let mut old_descriptor = PSECURITY_DESCRIPTOR::default();
    let err = unsafe {
        GetNamedSecurityInfoW(
            PCWSTR(encoded_path.as_ptr()),
            object_type,
            DACL_SECURITY_INFORMATION,
            None,
            None,
            Some(&mut old_acl),
            None,
            &mut old_descriptor,
        )
    };
    if err == ERROR_FILE_NOT_FOUND || err == ERROR_PATH_NOT_FOUND {
        return Ok(());
    }
    if err != ERROR_SUCCESS {
        return Err(windows::core::Error::from_hresult(HRESULT::from_win32(
            err.0,
        ))
        .into());
    }
    scopeguard::defer! {
        unsafe { LocalFree(Some(windows::Win32::Foundation::HLOCAL(old_descriptor.0))) };
    }

    let mut access = EXPLICIT_ACCESS_W::default();
    access.grfAccessMode = REVOKE_ACCESS;
    access.Trustee.MultipleTrusteeOperation = NO_MULTIPLE_TRUSTEE;
    access.Trustee.pMultipleTrustee = std::ptr::null_mut();
    access.Trustee.ptstrName = PWSTR(app_container.0 as *mut u16);
    access.Trustee.TrusteeForm = TRUSTEE_IS_SID;
    access.Trustee.TrusteeType = TRUSTEE_IS_GROUP;

    let mut new_acl = std::ptr::null_mut();
    let err = unsafe {
        SetEntriesInAclW(Some(&[access]), Some(old_acl), &mut new_acl)
    };
    if new_acl.is_null() {
        return Err(Error::new(ErrorKind::Other, "new acl was null").into());
    }
    scopeguard::defer! {
        unsafe { LocalFree(Some(windows::Win32::Foundation::HLOCAL(new_acl.cast()))) };
    }
    if err != ERROR_SUCCESS {
        return Err(windows::core::Error::from_hresult(HRESULT::from_win32(
            err.0,
        ))
        .into());
    }

    let err = unsafe {
        SetNamedSecurityInfoW(
            PCWSTR(encoded_path.as_ptr()),
            object_type,
            DACL_SECURITY_INFORMATION,
            None,
            None,
            Some(new_acl),
            None,
        )
    };
    if err == ERROR_FILE_NOT_FOUND || err == ERROR_PATH_NOT_FOUND {
        return Ok(());
    }
    if err != ERROR_SUCCESS {
        return Err(windows::core::Error::from_hresult(HRESULT::from_win32(
            err.0,
        ))
        .into());
    }
    Ok(())
}

fn acl_eq(first: *const ACL, second: *const ACL) -> Result<bool> {
    if first.is_null() && second.is_null() {
        return Ok(true);
    }
    let first = unsafe { first.as_ref() };
    let Some(first) = first else {
        return Ok(false);
    };
    let second = unsafe { second.as_ref() };
    let Some(second) = second else {
        return Ok(false);
    };
    if first.AceCount != second.AceCount {
        return Ok(false);
    }

    for i in 0..first.AceCount {
        let mut first_ace_ptr = std::ptr::null_mut();
        let mut second_ace_ptr = std::ptr::null_mut();
        unsafe { GetAce(first, i as u32, &mut first_ace_ptr)? };
        unsafe { GetAce(second, i as u32, &mut second_ace_ptr)? };

        if first_ace_ptr.is_null() && second_ace_ptr.is_null() {
            continue;
        }
        let first_ace_header =
            unsafe { first_ace_ptr.cast::<ACE_HEADER>().as_ref() };
        let Some(first_ace_header) = first_ace_header else {
            return Ok(false);
        };
        let second_ace_header =
            unsafe { second_ace_ptr.cast::<ACE_HEADER>().as_ref() };
        let Some(second_ace_header) = second_ace_header else {
            return Ok(false);
        };

        if first_ace_header != second_ace_header {
            return Ok(false);
        }

        let first_data = unsafe {
            std::slice::from_raw_parts(
                first_ace_ptr.cast::<u8>(),
                first_ace_header.AceSize as usize,
            )
        };
        let second_data = unsafe {
            std::slice::from_raw_parts(
                second_ace_ptr.cast::<u8>(),
                second_ace_header.AceSize as usize,
            )
        };

        if first_data != second_data {
            return Ok(false);
        }
    }

    Ok(true)
}

pub fn run_helper(
    args: impl IntoIterator<Item = OsString>,
) -> anyhow::Result<bool> {
    let mut args = args.into_iter();
    let (
        Some(app_container_name),
        Some(need_set_network_isolation_elevated),
        Some(traverse_paths),
        Some(read_paths),
        Some(write_paths),
        Some(remove_paths),
        None,
    ) = (
        args.next(),
        args.next(),
        args.next(),
        args.next(),
        args.next(),
        args.next(),
        args.next(),
    )
    else {
        bail!("invalid arguments");
    };

    let encoded_name = app_container_name
        .as_os_str()
        .encode_wide()
        .chain([0])
        .collect::<Vec<_>>();

    let psid = unsafe {
        DeriveAppContainerSidFromAppContainerName(PCWSTR(
            encoded_name.as_ptr(),
        ))?
    };
    scopeguard::defer! {
        unsafe { FreeSid(psid) };
    }
    ensure!(
        !psid.is_invalid(),
        "DeriveAppContainerSidFromAppContainerName returned invalid sid"
    );

    let need_set_network_isolation_elevated =
        need_set_network_isolation_elevated == OsStr::new("true");
    if need_set_network_isolation_elevated {
        try_set_network_isolation(&psid)?;
    }

    for path in std::env::split_paths(remove_paths.as_os_str()) {
        if !path.as_os_str().is_empty() {
            if let Err(err) = remove_from_acl(&psid, &path, SE_FILE_OBJECT) {
                tracing::warn!(
                    "unable to revoke sandbox access from {path:?}: {err:#}"
                );
            }
        }
    }
    apply_acl_paths(
        &psid,
        std::env::split_paths(traverse_paths.as_os_str()),
        PermissionType::Traverse,
    )?;
    apply_acl_paths(
        &psid,
        std::env::split_paths(read_paths.as_os_str()),
        PermissionType::Read,
    )?;
    apply_acl_paths(
        &psid,
        std::env::split_paths(write_paths.as_os_str()),
        PermissionType::Write,
    )?;

    Ok(true)
}

fn apply_acl_paths(
    app_container: &PSID,
    paths: impl IntoIterator<Item = std::path::PathBuf>,
    permission: PermissionType,
) -> Result<()> {
    for path in paths {
        if path.as_os_str().is_empty() {
            continue;
        }
        set_acl(app_container, &path, permission)?;
    }
    Ok(())
}
