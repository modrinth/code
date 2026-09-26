use std::{cell::OnceCell, collections::HashSet, ffi::{OsStr, OsString}, io::{Error, ErrorKind}, os::windows::{ffi::{OsStrExt, OsStringExt}, io::OwnedHandle}, path::Path, sync::OnceLock};

use windows::{Win32::{Foundation::{ERROR_ACCESS_DENIED, ERROR_ALREADY_EXISTS, ERROR_INSUFFICIENT_BUFFER, ERROR_SUCCESS, GetLastError, HANDLE, LocalFree}, Security::{ACE_HEADER, ACL, Authorization::{ConvertSidToStringSidW, ConvertStringSidToSidW, EXPLICIT_ACCESS_W, GRANT_ACCESS, GetNamedSecurityInfoW, GetSecurityInfo, NO_MULTIPLE_TRUSTEE, SE_FILE_OBJECT, SE_WINDOW_OBJECT, SetEntriesInAclW, SetNamedSecurityInfoW, SetSecurityInfo, TRUSTEE_IS_GROUP, TRUSTEE_IS_SID}, CONTAINER_INHERIT_ACE, CreateWellKnownSid, DACL_SECURITY_INFORMATION, DeriveCapabilitySidsFromName, FreeSid, GetAce, InitializeSecurityDescriptor, Isolation::{CreateAppContainerProfile, DeleteAppContainerProfile, DeriveAppContainerSidFromAppContainerName}, NO_INHERITANCE, OBJECT_INHERIT_ACE, PSECURITY_DESCRIPTOR, PSID, SECURITY_CAPABILITIES, SID_AND_ATTRIBUTES, SetFileSecurityW, SetSecurityDescriptorDacl, WELL_KNOWN_SID_TYPE, WinCapabilityInternetClientServerSid, WinCapabilityInternetClientSid, WinCapabilityPrivateNetworkClientServerSid}, Storage::FileSystem::{FILE_ALL_ACCESS, FILE_GENERIC_EXECUTE, FILE_GENERIC_READ, FILE_TRAVERSE, READ_CONTROL, WRITE_DAC}, System::{StationsAndDesktops::OpenWindowStationW, SystemServices::{SE_GROUP_ENABLED, SECURITY_DESCRIPTOR_REVISION}, Threading::{DeleteProcThreadAttributeList, InitializeProcThreadAttributeList, LPPROC_THREAD_ATTRIBUTE_LIST, PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES, UpdateProcThreadAttribute}}, UI::WindowsAndMessaging::WINSTA_WRITEATTRIBUTES}, core::{HRESULT, PCWSTR, PWSTR}};

use async_trait::async_trait;
use derive_more::Debug;
use eyre::{Context, OptionExt, Result, eyre};

use crate::{
    SandboxArg, SandboxCommand, SandboxExitStatus, backend::{Backend, SandboxChild, SandboxChildOp, SandboxEnv},
};

#[derive(Debug)]
pub struct AppContainer;

#[async_trait]
impl Backend for AppContainer {
    async fn init() -> Result<Box<dyn SandboxEnv>> {
        Ok(Box::new(AppContainerEnv {
            job_handle: super::create_job_object().wrap_err("error creating job object")?,
            null_device: super::open_null_device().wrap_err("error opening null device")?,
        }))
    }
}

#[derive(Clone, Debug)]
#[debug("AppContainerEnv")]
pub struct AppContainerEnv {
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
        let this = self.clone();
        send_spawn(this, command).await?
    }
}

struct SpawnInfo {
    env: AppContainerEnv,
    command: SandboxCommand,
    sender: tokio::sync::oneshot::Sender<eyre::Result<crate::SandboxChild>>
}

// probably not needed, just added to be as close to pandora as possible
fn send_spawn(env: AppContainerEnv, command: SandboxCommand) -> tokio::sync::oneshot::Receiver<eyre::Result<crate::SandboxChild>> {
    let (sender, receiver) = tokio::sync::oneshot::channel();

    static SPAWNING_CHANNEL: OnceLock<std::sync::mpsc::Sender<SpawnInfo>> = OnceLock::new();
    let channel = SPAWNING_CHANNEL.get_or_init(|| {
        let (send, recv) = std::sync::mpsc::channel::<SpawnInfo>();

        std::thread::Builder::new()
            .name("Modrinth Command Spawner".to_string())
            .stack_size(128 * 1024)
            .spawn(|| {
                // Initialize COM on this thread. In my testing this wasn't needed, but it shouldn't hurt
                #[cfg(windows)]
                unsafe {
                    _ = windows::Win32::System::Com::CoInitializeEx(
                        None,
                        windows::Win32::System::Com::COINIT_APARTMENTTHREADED |  windows::Win32::System::Com::COINIT_DISABLE_OLE1DDE,
                    );
                }

                for info in recv {
                    _ = info.sender.send(spawn(&info.env, info.command));
                }
            })
            .unwrap();

        send
    });
    channel.send(SpawnInfo { env, command, sender }).unwrap();

    receiver
}

fn spawn(env: &AppContainerEnv, mut command: SandboxCommand) -> Result<crate::SandboxChild> {
    let environment = command.take_environment();

    if !environment.contains_key(&"APPDATA".into()) {
        return Err(eyre!("Missing APPDATA environment variable"));
    }
    if !environment.contains_key(&"LOCALAPPDATA".into()) {
        return Err(eyre!("Missing LOCALAPPDATA environment variable"));
    }

    let app_container_sid = create_app_container(command.app_container_name.as_os_str(), command.app_container_description.as_os_str())?;
    scopeguard::defer! {
        unsafe { FreeSid(app_container_sid) };
    }

    let mut lpsize = 0;
    let result = unsafe { InitializeProcThreadAttributeList(None, 1, None, &mut lpsize) };

    if result != Err(windows::core::Error::from_hresult(HRESULT::from_win32(ERROR_INSUFFICIENT_BUFFER.0))) {
        result?;
    }

    let mut proc_thread_attribute_list_alloc = vec![0; lpsize];
    let lpproc_thread_attribute_list = LPPROC_THREAD_ATTRIBUTE_LIST(proc_thread_attribute_list_alloc.as_mut_ptr() as *mut _);

    unsafe { InitializeProcThreadAttributeList(Some(lpproc_thread_attribute_list), 1, None, &mut lpsize)? };
    scopeguard::defer! {
        unsafe { DeleteProcThreadAttributeList(lpproc_thread_attribute_list) };
    }

    let mut owned_capabilities: Vec<OwnedCapability> = Vec::new();

    if command.network {
        owned_capabilities.push(OwnedCapability::new(WinCapabilityInternetClientSid)?);
        owned_capabilities.push(OwnedCapability::new(WinCapabilityInternetClientServerSid)?);
        owned_capabilities.push(OwnedCapability::new(WinCapabilityPrivateNetworkClientServerSid)?);
    }

    let mut capabilities = owned_capabilities.iter_mut().map(|cap| {
        SID_AND_ATTRIBUTES {
            Sid: cap.as_psid(),
            Attributes: SE_GROUP_ENABLED as u32,
        }
    }).collect::<Vec<_>>();

    // Ideally we could just add the appcontainer's sid to winsta0 directly,
    // but for some reason this triggers a bug in chrome.
    // See https://github.com/Moulberry/PandoraLauncher/issues/415

    let mut group_sids = std::ptr::null_mut();
    let mut group_sid_count = 0;
    let mut capability_sids = std::ptr::null_mut();
    let mut capability_sid_count = 0;
    unsafe {
        DeriveCapabilitySidsFromName(windows::core::w!("modrinth.grantWinstaWriteAttributesCapability_ikU1gY09aHdb3PsX"),
            &mut group_sids, &mut group_sid_count, &mut capability_sids, &mut capability_sid_count)?
    };

    if capability_sid_count == 0 {
        tracing::error!("DeriveCapabilitySidsFromName returned 0 for capabilitysidcount");
    } else if capability_sids.is_null() {
        tracing::error!("DeriveCapabilitySidsFromName returned null for capability_sids");
    } else {
        let derived_sid = unsafe { capability_sids.read() };
        if let Err(err) = add_writeattributes_to_winsta(&derived_sid) {
            tracing::error!("Unable to set WINSTA_WRITEATTRIBUTES: {err}");
        } else {
            capabilities.push(SID_AND_ATTRIBUTES { Sid: derived_sid, Attributes: SE_GROUP_ENABLED as u32 });
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

    unsafe {
        UpdateProcThreadAttribute(
            lpproc_thread_attribute_list,
            0,
            PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES as usize,
            Some(&security_capabilities as *const SECURITY_CAPABILITIES as *const _),
            size_of::<SECURITY_CAPABILITIES>(),
            None,
            None
        )?
    };

    let mut readable: HashSet<std::path::PathBuf> = HashSet::default();
    let mut parents_to_add: HashSet<std::path::PathBuf> = HashSet::default();
    for allow_read in &command.read_only_paths {
        readable.insert(allow_read.clone());
        if let Some(parent) = allow_read.parent() {
            parents_to_add.insert(parent.to_path_buf());
        }
        if let Err(err) = add_to_acl(&app_container_sid, &allow_read, PermissionType::Read) {
            tracing::error!("Unable to allow reading from path {allow_read:?} due to error: {err}");
        }
    }
    for allow_write in &command.read_write_paths {
        readable.insert(allow_write.clone());
        if let Some(parent) = allow_write.parent() {
            parents_to_add.insert(parent.to_path_buf());
        }
        if let Err(err) = add_to_acl(&app_container_sid, &allow_write, PermissionType::Write) {
            tracing::error!("Unable to allow writing to path {allow_write:?} due to error: {err}");
        }
    }

    let mut parents: HashSet<std::path::PathBuf> = HashSet::default();
    loop {
        let taken = std::mem::take(&mut parents_to_add);
        if taken.is_empty() {
            break;
        }
        for to_add in taken {
            if readable.contains(to_add.as_path()) || parents.contains(&to_add) {
                continue;
            }
            if let Some(parent) = to_add.parent() {
                parents_to_add.insert(parent.to_path_buf());
            }
            parents.insert(to_add);
        }
    }
    let mut parents = parents.into_iter().collect::<Vec<_>>();
    parents.sort();

    parents.retain(|path| {
        if let Err(err) = add_to_acl(&app_container_sid, path, PermissionType::TraverseNoInherit) {
            let raw_access_denied = err.raw_os_error().unwrap_or(0) == HRESULT::from_win32(ERROR_ACCESS_DENIED.0).0;
            if err.kind() == ErrorKind::PermissionDenied || raw_access_denied {
                tracing::warn!("Lacking permission to allow traversal of {path:?}... will need to elevate");
                return true;
            }
            tracing::error!("Unable to allow traversal of path {path:?} due to error: {err}");
        }
        false
    });


    if !parents.is_empty() {
        // let Some(self_elevate_for_acl_arg) = sandbox.self_elevate_for_acl_arg else {
        //     return Err(eyre!("unable to do elevated acl modification because self_elevate_for_acl_arg wasn't set"));
        // };

        // let mut stringsid = PWSTR::default();
        // unsafe { ConvertSidToStringSidW(app_container_sid, &mut stringsid)? };
        // assert!(!stringsid.is_null());
        // let app_container_osstring = OsString::from_wide(unsafe { stringsid.as_wide() });

        // let mut command = PandoraCommand::new(std::env::current_exe()?);
        // command.arg(self_elevate_for_acl_arg);
        // command.arg(app_container_osstring);

        // for path in parents {
        //     command.arg(path);
        // }

        // tracing::info!("Spawning elevated self to modify acl");
        // let elevated = crate::windows::runas::spawn(command, context)?;
        // let elevated_status = elevated.process.wait()?;
        // tracing::info!("Done spawning elevated self to modify acl: {elevated_status}");
    }

    _ = add_to_acl(&app_container_sid, &command.executable, PermissionType::Read);

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
        Some(lpproc_thread_attribute_list),
    )
    .wrap_err("spawning child")?;

    Ok(crate::SandboxChild {
        stdin: pipes.stdin,
        stdout: pipes.stdout,
        stderr: pipes.stderr,
        imp: SandboxChild::AppContainer(child),
    })
}

struct OwnedCapability {
    sid_alloc: Vec<u8>,
}

impl OwnedCapability {
    pub fn new(known: WELL_KNOWN_SID_TYPE) -> std::io::Result<Self> {
        let mut size = 0;
        let result = unsafe { CreateWellKnownSid(known, None, None, &mut size) };
        if result != Err(windows::core::Error::from_hresult(HRESULT::from_win32(ERROR_INSUFFICIENT_BUFFER.0))) {
            result?;
        }
        let mut sid_alloc = vec![0; size as usize];
        unsafe { CreateWellKnownSid(known, None, Some(PSID(sid_alloc.as_mut_ptr() as *mut _)), &mut size)? };
        Ok(Self {
            sid_alloc
        })
    }

    pub fn as_psid(&mut self) -> PSID {
        PSID(self.sid_alloc.as_mut_ptr() as *mut _)
    }
}

fn create_app_container(name: &OsStr, description: &OsStr) -> windows::core::Result<PSID> {
    let encoded_name = name.encode_wide()
        .chain([0])
        .collect::<Vec<_>>();
    let encoded_description = description.encode_wide()
        .chain([0])
        .collect::<Vec<_>>();

    let mut result = unsafe {
        CreateAppContainerProfile(
            PCWSTR(encoded_name.as_ptr()),
            PCWSTR(encoded_name.as_ptr()),
            PCWSTR(encoded_description.as_ptr()),
            None,
        )
    };

    if result == Err(windows::core::Error::from_hresult(HRESULT::from_win32(ERROR_ALREADY_EXISTS.0))) {
        result = unsafe { DeriveAppContainerSidFromAppContainerName(PCWSTR(encoded_name.as_ptr())) };
    }

    result
}

enum PermissionType {
    TraverseNoInherit,
    Write,
    Read,
}

fn add_writeattributes_to_winsta(app_container: &PSID) -> std::io::Result<()> {
    let winsta = unsafe { OpenWindowStationW(windows::core::w!("winsta0"), false, (READ_CONTROL | WRITE_DAC).0) }?;
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
        return Err(windows::core::Error::from_hresult(HRESULT::from_win32(err.0)).into());
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
    let err = unsafe {
        SetEntriesInAclW(Some(&[ea]), Some(old_acl), &mut new_acl)
    };
    if new_acl.is_null() {
        return Err(Error::new(ErrorKind::Other, "new acl was null"));
    }
    scopeguard::defer! {
        unsafe { LocalFree(Some(windows::Win32::Foundation::HLOCAL(new_acl as *mut _))) };
    }
    if err != ERROR_SUCCESS {
        return Err(windows::core::Error::from_hresult(HRESULT::from_win32(err.0)).into());
    }

    match acl_eq(old_acl, new_acl) {
        Ok(true) => {
            return Ok(())
        },
        Ok(false) => {},
        Err(err) => {
            tracing::error!("Error comparing ACL for winsta0, updating acl anyways: {err}");
        },
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
        return Err(windows::core::Error::from_hresult(HRESULT::from_win32(err.0)).into());
    }

    Ok(())
}

fn add_to_acl(app_container: &PSID, path: &Path, perms: PermissionType) -> std::io::Result<()> {
    let encoded_path = path.as_os_str().encode_wide()
        .chain([0])
        .collect::<Vec<_>>();
    let mut old_acl = std::ptr::null_mut();
    let err = unsafe {
        GetNamedSecurityInfoW(
            PCWSTR(encoded_path.as_ptr()),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            None,
            None,
            Some(&mut old_acl),
            None,
            std::ptr::null_mut()
        )
    };
    if err != ERROR_SUCCESS {
        return Err(windows::core::Error::from_hresult(HRESULT::from_win32(err.0)).into());
    }
    let mut ea = EXPLICIT_ACCESS_W::default();
    ea.grfAccessMode = GRANT_ACCESS;
    ea.grfAccessPermissions = match perms {
        PermissionType::TraverseNoInherit =>  (FILE_GENERIC_READ | FILE_TRAVERSE).0,
        PermissionType::Write => (FILE_ALL_ACCESS).0,
        PermissionType::Read => (FILE_GENERIC_READ | FILE_TRAVERSE | FILE_GENERIC_EXECUTE).0,
    };
    ea.grfInheritance = if matches!(perms, PermissionType::TraverseNoInherit) {
        NO_INHERITANCE//CONTAINER_INHERIT_ACE
    } else {
        OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE
    };
    ea.Trustee.MultipleTrusteeOperation = NO_MULTIPLE_TRUSTEE;
    ea.Trustee.pMultipleTrustee = std::ptr::null_mut();
    ea.Trustee.ptstrName = windows::core::PWSTR(app_container.0 as *mut u16);
    ea.Trustee.TrusteeForm = TRUSTEE_IS_SID;
    ea.Trustee.TrusteeType = TRUSTEE_IS_GROUP;

    let mut new_acl = std::ptr::null_mut();
    let err = unsafe {
        SetEntriesInAclW(Some(&[ea]), Some(old_acl), &mut new_acl)
    };
    if new_acl.is_null() {
        return Err(Error::new(ErrorKind::Other, "new acl was null"));
    }
    scopeguard::defer! {
        unsafe { LocalFree(Some(windows::Win32::Foundation::HLOCAL(new_acl as *mut _))) };
    }
    if err != ERROR_SUCCESS {
        return Err(windows::core::Error::from_hresult(HRESULT::from_win32(err.0)).into());
    }

    if ea.grfInheritance == NO_INHERITANCE {
        // Only compare ACLs if inheritance is disabled.
        // Sometimes children don't properly inherit the ACLs,
        // so we need to be sure to apply them even if the parent is correct
        match acl_eq(old_acl, new_acl) {
            Ok(true) => {
                return Ok(())
            },
            Ok(false) => {},
            Err(err) => {
                tracing::error!("Error comparing ACL for {path:?}, updating acl anyways: {err}");
            },
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
                false
            )?;
        }

        unsafe {
            let success = SetFileSecurityW(
                PCWSTR(encoded_path.as_ptr()),
                DACL_SECURITY_INFORMATION,
                PSECURITY_DESCRIPTOR(security_buf.as_mut_ptr() as *mut _)
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
        return Err(windows::core::Error::from_hresult(HRESULT::from_win32(err.0)).into());
    }

    Ok(())
}

fn acl_eq(first: *const ACL, second: *const ACL) -> std::io::Result<bool> {
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
        let first_ace_header = unsafe { first_ace_ptr.cast::<ACE_HEADER>().as_ref() };
        let Some(first_ace_header) = first_ace_header else {
            return Ok(false);
        };
        let second_ace_header = unsafe { second_ace_ptr.cast::<ACE_HEADER>().as_ref() };
        let Some(second_ace_header) = second_ace_header else {
            return Ok(false);
        };

        if first_ace_header != second_ace_header {
            return Ok(false);
        }

        let first_data = unsafe { std::slice::from_raw_parts(first_ace_ptr.cast::<u8>(), first_ace_header.AceSize as usize) };
        let second_data = unsafe { std::slice::from_raw_parts(second_ace_ptr.cast::<u8>(), second_ace_header.AceSize as usize) };

        if first_data != second_data {
            return Ok(false);
        }
    }

    Ok(true)
}

pub fn set_traverse_acls(args: Vec<OsString>) -> std::io::Result<()> {
    if args.is_empty() {
        return Err(Error::new(ErrorKind::InvalidInput, "missing sid"));
    }

    let stringsid = args[0].as_os_str().encode_wide()
        .chain([0])
        .collect::<Vec<_>>();

    let mut psid = PSID::default();
    unsafe { ConvertStringSidToSidW(PCWSTR(stringsid.as_ptr()), &mut psid)? };
    if psid.is_invalid() {
        return Err(Error::new(ErrorKind::Other, "ConvertStringSidToSidW returned invalid sid"));
    }

    let mut first_error = None;
    for arg in &args[1..] {
        let path = Path::new(arg);
        if let Err(err) = add_to_acl(&psid, &path, PermissionType::TraverseNoInherit) {
            if first_error.is_none() {
                first_error = Some(err);
            }
        }
    }

    if let Some(err) = first_error {
        return Err(err);
    }

    Ok(())
}
