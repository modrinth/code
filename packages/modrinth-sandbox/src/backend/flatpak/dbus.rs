#![allow(dead_code, reason = "kept for completeness' sake")]

use std::collections::BTreeMap;

use zbus::zvariant::{Fd, OwnedFd, SerializeDict, Signature, Type};

pub use imp::{SpawnExitedStream, SpawnStartedStream};

bitflags::bitflags! {
    /// Controls how the Flatpak portal creates and manages a process.
    ///
    /// These flags are passed directly to the `flags` argument of the
    /// `org.freedesktop.portal.Flatpak.Spawn` method. Unknown flag bits cause
    /// the portal call to fail.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SpawnFlags: u32 {
        /// Clears the inherited environment before applying the variables in
        /// the `envs` argument.
        const CLEAR_ENV = 1 << 0;
        /// Runs the latest installed version of the calling Flatpak app instead
        /// of the version currently running.
        const LATEST_VERSION = 1 << 1;
        /// Creates a new, more restrictive subsandbox, equivalent to the
        /// sandboxing performed by `flatpak run --sandbox`.
        const SANDBOX = 1 << 2;
        /// Creates the process without network access by giving it a separate
        /// network namespace.
        ///
        /// This also separates its loopback interface from the caller, so it
        /// must not be used when the child needs loopback IPC with the caller.
        const NO_NETWORK = 1 << 3;
        /// Terminates the spawned sandbox when the calling D-Bus connection
        /// disappears from the session bus.
        const WATCH_BUS = 1 << 4;
        /// Makes the sandbox's process IDs visible in the caller's PID namespace.
        ///
        /// This requires [`Supports::EXPOSE_PIDS`] and is only supported when
        /// Flatpak uses user namespaces rather than a setuid sandbox helper.
        const EXPOSE_PIDS = 1 << 5;
        /// Requests a `SpawnStarted` signal after the process inside the new
        /// sandbox has started.
        ///
        /// This requires Flatpak portal interface version 4 or newer.
        const NOTIFY_START = 1 << 6;
        /// Shares process IDs in both directions between the caller and the new
        /// sandbox.
        ///
        /// This is broader than [`SpawnFlags::EXPOSE_PIDS`], requires PID
        /// exposure support, and requires Flatpak portal interface version 5 or
        /// newer.
        const SHARE_PIDS = 1 << 7;
        /// Mounts an empty directory at `/app` in the new sandbox.
        ///
        /// The caller's app files remain available below `/run/parent/app`.
        /// This flag is mutually exclusive with the `app-fd` spawn option and
        /// requires Flatpak portal interface version 6 or newer.
        const EMPTY_APP = 1 << 8;
    }

    /// Features supported by the running Flatpak portal implementation.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Supports: u32 {
        /// The portal supports [`SpawnFlags::EXPOSE_PIDS`]. On interface version
        /// 5 or newer, this also indicates support for [`SpawnFlags::SHARE_PIDS`].
        const EXPOSE_PIDS = 1 << 0;
    }

    /// Capabilities selectively granted to a process created as a subsandbox.
    ///
    /// These flags are encoded in [`SpawnOptions::sandbox_flags`]. They only
    /// apply when [`SpawnFlags::SANDBOX`] is set and cannot grant access that the
    /// calling Flatpak does not already have.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
    pub struct SandboxFlags: u32 {
        /// Shares the caller's X11 and Wayland display access with the sandbox.
        const SHARE_DISPLAY = 1 << 0;
        /// Shares the caller's PulseAudio sound access with the sandbox.
        const SHARE_SOUND = 1 << 1;
        /// Shares the caller's GPU access with the sandbox.
        const SHARE_GPU = 1 << 2;
        /// Grants access to a filtered session D-Bus connection.
        const SESSION_BUS = 1 << 3;
        /// Grants access to the accessibility D-Bus connection.
        const ACCESSIBILITY_BUS = 1 << 4;
        /// Grants access to input devices.
        const INPUT_DEVICES = 1 << 5;
        /// Grants access to USB devices.
        const USB_DEVICES = 1 << 6;
        /// Grants access to hardware virtualization through KVM.
        const KVM = 1 << 7;
        /// Grants access to shared memory provided by the caller's sandbox.
        const SHARED_MEMORY = 1 << 8;
        /// Grants access to all devices available to the caller.
        ///
        /// This is substantially broader than the individual device flags and
        /// should only be used when narrowly scoped grants are insufficient.
        const ALL_DEVICES = 1 << 9;
    }
}

impl Type for SandboxFlags {
    const SIGNATURE: &'static Signature = <u32 as Type>::SIGNATURE;
}

/// Optional settings for [`FlatpakPortalProxy::spawn`].
///
/// This serializes as the extensible D-Bus dictionary `a{sv}` expected by the
/// Flatpak portal. Fields set to [`None`] are omitted from the dictionary.
/// Unknown dictionary keys are ignored by the portal, unlike unknown bits in
/// [`SpawnFlags`], which cause the spawn request to fail.
#[derive(Debug, Default, SerializeDict, Type)]
#[zvariant(signature = "dict")]
pub struct SpawnOptions {
    /// Capabilities granted to the new subsandbox.
    ///
    /// This corresponds to the `sandbox-flags` option and only applies when
    /// [`SpawnFlags::SANDBOX`] is set.
    #[zvariant(rename = "sandbox-flags")]
    pub sandbox_flags: Option<SandboxFlags>,
    /// Read-write entries from the caller's Flatpak sandbox directory to expose.
    ///
    /// Each entry must be a single filename inside
    /// `~/.var/app/<APP_ID>/sandbox`. Absolute paths and subdirectories are not
    /// accepted. Use [`SpawnOptions::sandbox_expose_fd`] for arbitrary paths.
    #[zvariant(rename = "sandbox-expose")]
    pub sandbox_expose: Option<Vec<String>>,
    /// Read-only entries from the caller's Flatpak sandbox directory to expose.
    ///
    /// Each entry must be a single filename inside
    /// `~/.var/app/<APP_ID>/sandbox`. Absolute paths and subdirectories are not
    /// accepted. Use [`SpawnOptions::sandbox_expose_fd_ro`] for arbitrary paths.
    #[zvariant(rename = "sandbox-expose-ro")]
    pub sandbox_expose_ro: Option<Vec<String>>,
    /// Paths exposed read-write, identified by owned file descriptors.
    ///
    /// Each descriptor must have been opened with `O_PATH | O_NOFOLLOW` and
    /// must not refer to a symbolic link. A path is writable only when the
    /// caller already has write access to it. This option requires Flatpak
    /// portal interface version 3 or newer.
    #[zvariant(rename = "sandbox-expose-fd")]
    pub sandbox_expose_fd: Option<Vec<OwnedFd>>,
    /// Paths exposed read-only, identified by owned file descriptors.
    ///
    /// Each descriptor must have been opened with `O_PATH | O_NOFOLLOW` and
    /// must not refer to a symbolic link. This option requires Flatpak portal
    /// interface version 3 or newer.
    #[zvariant(rename = "sandbox-expose-fd-ro")]
    pub sandbox_expose_fd_ro: Option<Vec<OwnedFd>>,
    /// Environment variables removed from the spawned process.
    ///
    /// This corresponds to the `unset-env` option and requires Flatpak portal
    /// interface version 5 or newer.
    #[zvariant(rename = "unset-env")]
    pub unset_env: Option<Vec<String>>,
    /// Directory to mount as `/usr` in the new sandbox.
    ///
    /// The descriptor must have been opened with `O_PATH | O_NOFOLLOW` and
    /// must not refer to a symbolic link. The caller's original runtime remains
    /// available under `/run/parent/usr`. This option requires Flatpak portal
    /// interface version 6 or newer.
    #[zvariant(rename = "usr-fd")]
    pub usr_fd: Option<OwnedFd>,
    /// Directory to mount as `/app` in the new sandbox.
    ///
    /// The descriptor must have been opened with `O_PATH | O_NOFOLLOW` and
    /// must not refer to a symbolic link. The caller's original app remains
    /// available under `/run/parent/app`. This option is mutually exclusive
    /// with [`SpawnFlags::EMPTY_APP`] and requires Flatpak portal interface
    /// version 6 or newer.
    #[zvariant(rename = "app-fd")]
    pub app_fd: Option<OwnedFd>,
    /// D-Bus names the sandbox may own on the accessibility bus.
    ///
    /// Each name must use the caller's application ID as its prefix. This only
    /// applies when [`SandboxFlags::ACCESSIBILITY_BUS`] is set and requires
    /// Flatpak portal interface version 7 or newer.
    #[zvariant(rename = "sandbox-a11y-own-names")]
    pub sandbox_a11y_own_names: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct FlatpakPortalProxy<'p> {
    inner: imp::FlatpakPortalProxy<'p>,
}

impl<'p> FlatpakPortalProxy<'p> {
    pub async fn new(conn: &zbus::Connection) -> zbus::Result<Self> {
        imp::FlatpakPortalProxy::new(conn)
            .await
            .map(|inner| Self { inner })
    }

    pub async fn version(&self) -> zbus::Result<u32> {
        self.inner.version().await
    }

    pub async fn supports(&self) -> zbus::Result<Supports> {
        self.inner.supports().await.map(Supports::from_bits_retain)
    }

    pub async fn spawn(
        &self,
        cwd_path: &[u8],
        argv: &[Vec<u8>],
        fds: &BTreeMap<u32, Fd<'_>>,
        envs: &BTreeMap<String, String>,
        flags: SpawnFlags,
        options: &SpawnOptions,
    ) -> zbus::Result<u32> {
        self.inner
            .spawn(cwd_path, argv, fds, envs, flags.bits(), options)
            .await
    }

    pub async fn spawn_signal(
        &self,
        pid: u32,
        signal: u32,
        to_process_group: bool,
    ) -> zbus::Result<()> {
        self.inner.spawn_signal(pid, signal, to_process_group).await
    }

    pub async fn receive_spawn_started(
        &self,
    ) -> zbus::Result<SpawnStartedStream> {
        self.inner.receive_spawn_started().await
    }

    pub async fn receive_spawn_exited(
        &self,
    ) -> zbus::Result<SpawnExitedStream> {
        self.inner.receive_spawn_exited().await
    }
}

mod imp {
    use std::collections::BTreeMap;

    use zbus::zvariant::Fd;

    use super::SpawnOptions;

    #[zbus::proxy(
        interface = "org.freedesktop.portal.Flatpak",
        default_service = "org.freedesktop.portal.Flatpak",
        default_path = "/org/freedesktop/portal/Flatpak",
        gen_blocking = false
    )]
    pub trait FlatpakPortal {
        #[zbus(property, name = "version")]
        fn version(&self) -> zbus::Result<u32>;

        #[zbus(property, name = "supports")]
        fn supports(&self) -> zbus::Result<u32>;

        fn spawn(
            &self,
            cwd_path: &[u8],
            argv: &[Vec<u8>],
            fds: &BTreeMap<u32, Fd<'_>>,
            envs: &BTreeMap<String, String>,
            flags: u32,
            options: &SpawnOptions,
        ) -> zbus::Result<u32>;

        fn spawn_signal(
            &self,
            pid: u32,
            signal: u32,
            to_process_group: bool,
        ) -> zbus::Result<()>;

        #[zbus(signal)]
        fn spawn_started(&self, pid: u32, relpid: u32) -> zbus::Result<()>;

        #[zbus(signal)]
        fn spawn_exited(&self, pid: u32, exit_status: u32) -> zbus::Result<()>;
    }
}
