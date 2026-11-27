use std::{
    collections::BTreeMap,
    os::fd::{AsRawFd, OwnedFd},
    path::{Path, PathBuf},
};

use anyhow::{Result, Context, anyhow};

use crate::{SandboxStdio, backend::unix, util::SandboxArg};

#[derive(Debug)]
pub struct DbusProxy {
    socket_path: PathBuf,
    _keep_alive_read_fd: OwnedFd,
}

impl DbusProxy {
    pub fn start(
        program: PathBuf,
        mut arguments: Vec<SandboxArg>,
        socket_path: PathBuf,
        working_directory: PathBuf,
        mut pass_fds: Vec<OwnedFd>,
        dev_null: libc::c_int,
    ) -> Result<Self> {
        const DBUS_ADDRESS_ENV: &str = "DBUS_SESSION_BUS_ADDRESS";

        let (keep_alive_read_fd, keep_alive_write_fd) = super::open_pipe()?;

        let session_bus_address = std::env::var_os(DBUS_ADDRESS_ENV)
            .ok_or_else(|| anyhow!("reading `{DBUS_ADDRESS_ENV}`"))?;

        arguments.push(session_bus_address.into());
        arguments.push(socket_path.clone().into());
        arguments
            .push(format!("--fd={}", keep_alive_write_fd.as_raw_fd()).into());
        arguments.push("--filter".into());
        arguments.extend(dbus_proxy_rules().into_iter().map(SandboxArg::from));

        pass_fds.push(keep_alive_write_fd);

        let mut environment = BTreeMap::default();
        for (arg, value) in std::env::vars_os() {
            environment.insert(arg.into(), value.into());
        }

        unix::spawn(
            program.into(),
            arguments,
            environment,
            SandboxStdio::Null,
            SandboxStdio::Null,
            SandboxStdio::Null,
            Some(working_directory),
            pass_fds,
            dev_null,
            true,
            false,
        )
        .context("spawning dbus proxy child")?;

        // Wait for proxy to start by reading from fd
        let mut buf = 0 as libc::c_char;
        unsafe {
            let start = std::time::Instant::now();
            if libc::read(
                keep_alive_read_fd.as_raw_fd(),
                &raw mut buf as *mut libc::c_char as *mut _,
                1,
            ) != 1
            {
                return Err(anyhow!("syncing with xdg-dbus-proxy"));
            }
            tracing::info!(
                "xdg-dbus-proxy took {:.1?} to start",
                start.elapsed()
            );
        }

        Ok(Self {
            socket_path,
            _keep_alive_read_fd: keep_alive_read_fd,
        })
    }

    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }
}

impl Drop for DbusProxy {
    fn drop(&mut self) {
        _ = std::fs::remove_file(&self.socket_path);
    }
}

/// Portals that can be used by the sandboxed process. Others aren't allowed because they
/// could be used to escape the sandbox or bypass its restrictions:
///
/// - Open files or run programs outside of the sandbox: Background, DynamicLauncher,
///   Email, OpenURI, Print, Wallpaper
/// - Access files without being checked by [`PathRules`](crate::path_access::PathRules): FileChooser, Trash
/// - Control or record the desktop: Clipboard, InputCapture, RemoteDesktop, ScreenCast,
///   Screenshot
/// - Access devices or credentials: Usb, experimental.Credential
const ALLOWED_DESKTOP_PORTALS: &[&str] = &[
    "Account",
    "Camera",
    "GameMode",
    "GlobalShortcuts",
    "Inhibit",
    "Location",
    "MemoryMonitor",
    "NetworkMonitor",
    "Notification",
    "PowerProfileMonitor",
    "ProxyResolver",
    "Realtime",
    "Secret",
    "Settings",
];

/// Creates arguments for xdg-dbus-proxy which allow access to specific services.
fn dbus_proxy_rules() -> Vec<String> {
    /// Allows calling specific methods, receiving specific signals, and reading the
    /// properties of an object.
    fn allow(
        rules: &mut Vec<String>,
        name: &str,
        path: &str,
        methods: &[&str],
        signals: &[&str],
    ) {
        // --talk must not be used, since it allows calling any method of any object
        // owned by the same process, even if there are --call rules

        // Properties.Set isn't included since it could be used to change the state of
        // something outside of the sandbox
        const PROPERTY_METHODS: &[&str] = &[
            "org.freedesktop.DBus.Properties.Get",
            "org.freedesktop.DBus.Properties.GetAll",
        ];
        const PROPERTY_SIGNALS: &[&str] =
            &["org.freedesktop.DBus.Properties.PropertiesChanged"];

        for method in methods.iter().chain(PROPERTY_METHODS) {
            rules.push(format!("--call={name}={method}@{path}"));
        }
        for signal in signals.iter().chain(PROPERTY_SIGNALS) {
            rules.push(format!("--broadcast={name}={signal}@{path}"));
        }
    }

    let mut rules = Vec::new();

    allow(
        &mut rules,
        "com.feralinteractive.GameMode",
        "/com/feralinteractive/GameMode",
        &["com.feralinteractive.GameMode.*"],
        &["com.feralinteractive.GameMode.*"],
    );
    allow(
        &mut rules,
        "org.kde.StatusNotifierWatcher",
        "/StatusNotifierWatcher",
        &["org.kde.StatusNotifierWatcher.*"],
        &["org.kde.StatusNotifierWatcher.*"],
    );
    allow(
        &mut rules,
        "org.freedesktop.Notifications",
        "/org/freedesktop/Notifications",
        &[
            "org.freedesktop.Notifications.CloseNotification",
            "org.freedesktop.Notifications.GetCapabilities",
            "org.freedesktop.Notifications.GetServerInformation",
            "org.freedesktop.Notifications.Notify",
        ],
        &["org.freedesktop.Notifications.*"],
    );
    // Notably, this doesn't include OpenUri
    allow(
        &mut rules,
        "org.mpris.MediaPlayer2.*",
        "/org/mpris/MediaPlayer2",
        &[
            "org.mpris.MediaPlayer2.Player.Next",
            "org.mpris.MediaPlayer2.Player.Pause",
            "org.mpris.MediaPlayer2.Player.Play",
            "org.mpris.MediaPlayer2.Player.PlayPause",
            "org.mpris.MediaPlayer2.Player.Previous",
            "org.mpris.MediaPlayer2.Player.Seek",
            "org.mpris.MediaPlayer2.Player.SetPosition",
            "org.mpris.MediaPlayer2.Player.Stop",
        ],
        &["org.mpris.MediaPlayer2.Player.Seeked"],
    );

    let desktop_portals = ALLOWED_DESKTOP_PORTALS
        .iter()
        .map(|portal| format!("org.freedesktop.portal.{portal}.*"))
        .collect::<Vec<_>>();
    let desktop_portals = desktop_portals
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    allow(
        &mut rules,
        "org.freedesktop.portal.Desktop",
        "/org/freedesktop/portal/desktop",
        &desktop_portals,
        &desktop_portals,
    );
    allow(
        &mut rules,
        "org.freedesktop.portal.Desktop",
        "/org/freedesktop/portal/desktop/request/*",
        &["org.freedesktop.portal.Request.*"],
        &["org.freedesktop.portal.Request.*"],
    );
    allow(
        &mut rules,
        "org.freedesktop.portal.Desktop",
        "/org/freedesktop/portal/desktop/session/*",
        &["org.freedesktop.portal.Session.*"],
        &["org.freedesktop.portal.Session.*"],
    );

    // Allows access to files which are dropped on the game
    allow(
        &mut rules,
        "org.freedesktop.portal.Documents",
        "/org/freedesktop/portal/documents",
        &[
            "org.freedesktop.portal.Documents.*",
            "org.freedesktop.portal.FileTransfer.*",
        ],
        &["org.freedesktop.portal.FileTransfer.*"],
    );

    // Input methods
    for path in ["/inputmethod", "/org/freedesktop/portal/inputmethod"] {
        allow(
            &mut rules,
            "org.freedesktop.portal.Fcitx",
            path,
            &["org.fcitx.Fcitx.InputMethod1.*"],
            &["org.fcitx.Fcitx.InputMethod1.*"],
        );
    }
    allow(
        &mut rules,
        "org.freedesktop.portal.Fcitx",
        "/org/freedesktop/portal/inputcontext/*",
        &["org.fcitx.Fcitx.InputContext1.*"],
        &["org.fcitx.Fcitx.InputContext1.*"],
    );
    allow(
        &mut rules,
        "org.freedesktop.portal.IBus",
        "/org/freedesktop/IBus/*",
        &[
            // Used to configure the input context
            "org.freedesktop.DBus.Properties.Set",
            "org.freedesktop.IBus.InputContext.*",
            "org.freedesktop.IBus.Portal.*",
            "org.freedesktop.IBus.Service.*",
        ],
        &["org.freedesktop.IBus.InputContext.*"],
    );

    rules
}
