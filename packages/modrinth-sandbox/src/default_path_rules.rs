use std::path::{Path, PathBuf};

use crate::{
    PathRule,
    path_access::{normalize_path, resolve_existing},
};

/// Credentials of other programs, as well as files that could be used to run code
/// outside of the sandbox. Users can't override these, so this only contains specific
/// paths that the game has no reason to access.
const HARD_BLOCKED_IN_HOME: &[&str] = &[
    ".aws",
    ".azure",
    ".bash_history",
    ".bash_login",
    ".bash_logout",
    ".bash_profile",
    ".bashrc",
    ".cargo/config.toml", // Can contain registry tokens, and commands that Cargo runs
    ".cargo/credentials.toml",
    ".config/BraveSoftware",
    ".config/chromium",
    ".config/discord",
    ".config/discordcanary",
    ".config/discorddevelopment",
    ".config/discordptb",
    ".config/gcloud", // Google Cloud credentials
    ".config/gh",     // GitHub CLI tokens
    ".config/google-chrome",
    ".config/google-chrome-beta",
    ".config/google-chrome-unstable",
    ".config/microsoft-edge",
    ".config/mozilla", // Firefox profiles
    ".config/opera",
    ".config/vivaldi",
    ".config/yandex-browser",
    ".docker",                   // Registry credentials
    ".git-credentials",          // Passwords saved by Git's credential store
    ".gitconfig", // Can contain tokens, and commands that Git runs
    ".gnupg",     // GPG private keys
    ".gradle/gradle.properties", // Can contain repository passwords and signing keys
    ".gradle/init.d",            // Scripts which run for every Gradle build
    ".kube",                     // Kubernetes credentials
    ".local/share/PrismLauncher/accounts.json",
    ".local/share/TelegramDesktop",
    ".lunarclient/settings/game/accounts.json",
    ".m2/settings.xml", // Maven repository passwords
    ".mozilla",         // Firefox profiles
    ".netrc",           // Passwords used by curl, Git, FTP clients, etc.
    ".npmrc",           // npm registry tokens
    ".password-store",  // Used by the pass password manager
    ".profile",
    ".pypirc", // PyPI tokens
    ".ssh",
    ".thunderbird",
    ".var/app/com.discordapp.Discord",
    ".var/app/com.google.Chrome",
    ".var/app/org.mozilla.firefox",
    ".var/app/org.prismlauncher.PrismLauncher/data/PrismLauncher/accounts.json",
    ".zprofile",
    ".zsh_history",
    ".zshenv",
    ".zshrc",
    "_netrc", // Windows equivalent of .netrc
    "snap/discord",
    "snap/firefox",
];

#[cfg(windows)]
const HARD_BLOCKED_IN_ROAMING_APP_DATA: &[&str] = &[
    "discord",
    "discordcanary",
    "discordptb",
    "discorddevelopment",
    "Discord Canary",
    "Discord PTB",
    "Discord Development",
    "Microsoft/Credentials",
    "Microsoft/Crypto",  // Private keys
    "Microsoft/Protect", // Keys which encrypt the user's other credentials
    "Microsoft/Vault",
    "Microsoft/Windows/Start Menu", // Programs which run when logging in
    "Mozilla",                      // Firefox profiles
    "Opera Software",
    "PrismLauncher/accounts.json",
    "Telegram Desktop",
    "Thunderbird",
];

#[cfg(windows)]
const HARD_BLOCKED_IN_LOCAL_APP_DATA: &[&str] = &[
    "BraveSoftware",
    "Chromium",
    "Google/Chrome",
    "Google/Chrome Beta",
    "Google/Chrome Dev",
    "Google/Chrome SxS", // Chrome Canary
    "Microsoft/Credentials",
    "Microsoft/Edge",
    "Microsoft/Edge Beta",
    "Microsoft/Edge Dev",
    "Microsoft/Edge SxS", // Edge Canary
    "Microsoft/Vault",
    "Vivaldi",
    "Yandex/YandexBrowser",
];

#[cfg(target_os = "macos")]
const HARD_BLOCKED_IN_LIBRARY: &[&str] = &[
    "Application Support/BraveSoftware",
    "Application Support/Chromium",
    "Application Support/Firefox",
    "Application Support/Google/Chrome",
    "Application Support/Google/Chrome Beta",
    "Application Support/Google/Chrome Canary",
    "Application Support/Google/Chrome Dev",
    "Application Support/Microsoft Edge",
    "Application Support/Microsoft Edge Beta",
    "Application Support/Microsoft Edge Canary",
    "Application Support/Microsoft Edge Dev",
    "Application Support/PrismLauncher/accounts.json",
    "Application Support/Telegram Desktop",
    "Application Support/Vivaldi",
    "Application Support/Yandex/YandexBrowser",
    "Application Support/com.operasoftware.Opera",
    "Application Support/com.operasoftware.OperaGX",
    "Application Support/discord",
    "Application Support/discordcanary",
    "Application Support/discorddevelopment",
    "Application Support/discordptb",
    "Cookies", // Safari cookies
    "Keychains",
    "Safari",
    "Thunderbird",
];

const HARD_BLOCKED_IN_VANILLA: &[&str] = &[
    "launcher_accounts.json",
    "launcher_accounts_microsoft_store.json",
    "launcher_msa_credentials.bin",
    "launcher_msa_credentials_microsoft_store.bin",
];

const UNBLOCKED_IN_VANILLA: &[&str] = &[
    "resourcepacks",
    "saves",
    "schematics",
    "screenshots",
    "shaderpacks",
];

/// The game directory of the vanilla launcher.
fn vanilla_dir() -> Option<PathBuf> {
    if cfg!(windows) {
        let app_data = std::env::var_os("APPDATA")?;
        return Some(Path::new(&app_data).join(".minecraft"));
    }
    let home = std::env::home_dir()?;
    Some(if cfg!(target_os = "macos") {
        home.join("Library/Application Support/minecraft")
    } else {
        home.join(".minecraft")
    })
}

/// Folders which can't be exposed unless the launcher or user explicitly gives
/// access to something inside of them.
pub fn soft_path_rules() -> Vec<PathRule> {
    let mut rules = Vec::new();

    #[cfg(windows)]
    {
        use windows::Win32::UI::Shell::{
            FOLDERID_LocalAppData, FOLDERID_LocalAppDataLow, FOLDERID_Profile,
            FOLDERID_ProgramData, FOLDERID_ProgramFiles,
            FOLDERID_ProgramFilesX86, FOLDERID_RoamingAppData,
            FOLDERID_Windows,
        };

        rules.extend(
            [
                FOLDERID_LocalAppData,
                FOLDERID_LocalAppDataLow,
                FOLDERID_RoamingAppData,
            ]
            .iter()
            .filter_map(get_known_folder)
            .map(PathRule::SoftBlock),
        );
        if let Some(profile) = get_known_folder(&FOLDERID_Profile) {
            rules.push(PathRule::SoftBlock(profile.join("AppData")));
        }
        if let Some(local_app_data) = get_known_folder(&FOLDERID_LocalAppData) {
            // Programs extract files to this folder when dragging out of an archive
            rules.push(PathRule::Unblock(local_app_data.join("Temp")));
        }
        rules.extend(
            [
                FOLDERID_ProgramData,
                FOLDERID_ProgramFiles,
                FOLDERID_ProgramFilesX86,
                FOLDERID_Windows,
            ]
            .iter()
            .filter_map(get_known_folder)
            .map(PathRule::SoftBlockWrite),
        );
    }

    if let Some(home) = std::env::home_dir() {
        // Data of many programs on Linux
        rules.push(PathRule::SoftBlock(home.join(".config")));
    }
    if let Some(vanilla) = vanilla_dir() {
        rules.push(PathRule::SoftBlock(vanilla.clone()));
        rules.extend(
            UNBLOCKED_IN_VANILLA
                .iter()
                .map(|name| PathRule::Unblock(vanilla.join(name))),
        );
    }
    rules.extend(path_variable_rules());
    rules
}

/// Known sensitive paths, which the launcher and user can't give access to.
pub fn hard_path_rules() -> Vec<PathRule> {
    let mut rules = Vec::new();

    if let Some(home) = std::env::home_dir() {
        rules.extend(
            HARD_BLOCKED_IN_HOME
                .iter()
                .map(|name| PathRule::HardBlock(home.join(name))),
        );
    }
    if let Some(vanilla) = vanilla_dir() {
        rules.extend(
            HARD_BLOCKED_IN_VANILLA
                .iter()
                .map(|name| PathRule::HardBlock(vanilla.join(name))),
        );
    }

    #[cfg(windows)]
    {
        use windows::Win32::UI::Shell::{
            FOLDERID_Documents, FOLDERID_LocalAppData, FOLDERID_RoamingAppData,
        };

        for (folder, names) in [
            (FOLDERID_RoamingAppData, HARD_BLOCKED_IN_ROAMING_APP_DATA),
            (FOLDERID_LocalAppData, HARD_BLOCKED_IN_LOCAL_APP_DATA),
        ] {
            if let Some(folder) = get_known_folder(&folder) {
                rules.extend(
                    names
                        .iter()
                        .map(|name| PathRule::HardBlock(folder.join(name))),
                );
            }
        }
        if let Some(documents) = get_known_folder(&FOLDERID_Documents) {
            // Contains scripts which run when PowerShell starts
            for name in ["PowerShell", "WindowsPowerShell"] {
                rules.push(PathRule::HardBlockWrite(documents.join(name)));
            }
        }
    }

    #[cfg(target_os = "macos")]
    if let Some(home) = std::env::home_dir() {
        let library = home.join("Library");
        rules.extend(
            HARD_BLOCKED_IN_LIBRARY
                .iter()
                .map(|name| PathRule::HardBlock(library.join(name))),
        );
        // Contains programs which run when logging in
        rules.push(PathRule::HardBlockWrite(library.join("LaunchAgents")));
    }

    #[cfg(target_os = "linux")]
    {
        // Contains the memory and environment of other processes
        rules.push(PathRule::HardBlock("/proc".into()));

        if let Some(directories) = directories::BaseDirs::new() {
            // Default XDG directories may be moved, so also block the default ones
            let home = directories.home_dir();
            let default_data_dir = home.join(".local/share");
            for data_dir in [default_data_dir.as_path(), directories.data_dir()]
            {
                // Passwords stored by GNOME and KDE
                for name in ["keyrings", "kwalletd"] {
                    rules.push(PathRule::HardBlock(data_dir.join(name)));
                }
                // Contain commands which run when opening an application, when a D-Bus
                // service is used, or when logging in
                for name in ["applications", "dbus-1", "systemd"] {
                    rules.push(PathRule::HardBlockWrite(data_dir.join(name)));
                }
            }
            let default_config_dir = home.join(".config");
            for config_dir in
                [default_config_dir.as_path(), directories.config_dir()]
            {
                // Contain commands which run, or environment variables that are set,
                // when logging in
                for name in ["autostart", "environment.d", "systemd"] {
                    rules.push(PathRule::HardBlockWrite(config_dir.join(name)));
                }
            }
            // Contains programs which are usually on the PATH
            rules.push(PathRule::HardBlockWrite(home.join(".local/bin")));
        } else {
            tracing::error!("unable to determine base directories");
        }
    }

    rules
}

/// Prevents programs in the folders of the `PATH` environment variable from being
/// added or replaced.
fn path_variable_rules() -> Vec<PathRule> {
    let Some(variable) = std::env::var_os("PATH") else {
        return Vec::new();
    };
    let home = std::env::home_dir()
        .and_then(|home| resolve_existing(&home))
        .map(normalize_path);
    std::env::split_paths(&variable)
        .filter(|path| path.is_absolute())
        .filter(|path| {
            // Ignore folders that are so broad that almost nothing could be written to
            resolve_existing(path)
                .map(normalize_path)
                .is_some_and(|path| {
                    path.parent().is_some()
                        && !home
                            .as_ref()
                            .is_some_and(|home| home.starts_with(&path))
                })
        })
        .map(PathRule::SoftBlockWrite)
        .collect()
}

#[cfg(windows)]
fn get_known_folder(id: &windows::core::GUID) -> Option<PathBuf> {
    use std::{ffi::OsString, os::windows::ffi::OsStringExt};
    use windows::Win32::{
        System::Com::CoTaskMemFree,
        UI::Shell::{KF_FLAG_DONT_VERIFY, SHGetKnownFolderPath},
    };

    match unsafe { SHGetKnownFolderPath(id, KF_FLAG_DONT_VERIFY, None) } {
        Ok(path) => {
            let folder = OsString::from_wide(unsafe { path.as_wide() });
            unsafe { CoTaskMemFree(Some(path.0.cast())) };
            Some(folder.into())
        }
        Err(err) => {
            tracing::error!("unable to get known folder {id:?}: {err}");
            None
        }
    }
}
