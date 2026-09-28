use std::{
    ffi::{CString, OsString},
    os::unix::ffi::OsStringExt,
    path::{Path, PathBuf},
};

use async_trait::async_trait;
use derive_more::Debug;
use eyre::{Context, OptionExt, Result};

use crate::{
    SandboxCommand, SandboxExitStatus,
    backend::{
        Backend, SandboxChild, SandboxChildOp, SandboxEnv,
        unix::{self, UnixChild},
    },
    util::{RawStringVec, resolve_path},
};

#[derive(Debug)]
pub struct Macos;

#[async_trait]
impl Backend for Macos {
    async fn init() -> Result<Box<dyn SandboxEnv>> {
        let dev_null =
            super::unix::open_dev_null().wrap_err("opening /dev/null")?;
        Ok(Box::new(MacosEnv { dev_null }))
    }
}

#[derive(Debug, Clone)]
#[debug("MacosEnv")]
pub struct MacosEnv {
    dev_null: libc::c_int,
}

#[async_trait]
impl SandboxEnv for MacosEnv {
    async fn spawn(
        &self,
        command: SandboxCommand,
    ) -> Result<crate::SandboxChild> {
        let env = self.clone();
        let child = tokio::task::spawn_blocking(move || spawn(&env, command))
            .await
            .wrap_err("spawn task dropped")??;
        Ok(child)
    }
}

fn spawn(
    env: &MacosEnv,
    mut command: SandboxCommand,
) -> Result<crate::SandboxChild> {
    let temp_root = confstr(libc::_CS_DARWIN_USER_TEMP_DIR)
        .map(OsString::from_vec)
        .map(PathBuf::from)
        .ok_or_eyre("macOS user temporary directory is unavailable")?
        .canonicalize()
        .wrap_err("resolving macOS user temporary directory")?;
    let temp_dir = tempfile::Builder::new()
        .prefix("modrinth-sandbox-")
        .tempdir_in(&temp_root)
        .wrap_err("creating sandbox temporary directory")?;
    let temp_path = temp_dir.path().to_path_buf();

    command
        .extra_environment
        .insert("TMPDIR".into(), temp_path.clone().into());
    if command.is_jvm {
        let mut java_temp_argument = OsString::from("-Djava.io.tmpdir=");
        java_temp_argument.push(&temp_path);
        command.args.insert(0, java_temp_argument.into());
    }

    let environment = command.take_environment();
    let mut sandbox_profile = OsString::from(BASE_PROFILE);
    allow_read_write(&mut sandbox_profile, &temp_path);

    let resolved_executable = resolve_path(&command.executable)?;
    allow_read(&mut sandbox_profile, &resolved_executable);

    if command.network {
        sandbox_profile.push(NETWORK);
    }

    if command.is_jvm {
        sandbox_profile.push("(allow file-write* file-read* (prefix \"");
        sandbox_profile.push(temp_root.join("hsperfdata_"));
        sandbox_profile.push("\"))\n");

        let cache_root = confstr(libc::_CS_DARWIN_USER_CACHE_DIR)
            .map(OsString::from_vec)
            .map(PathBuf::from)
            .ok_or_eyre("macOS user cache directory is unavailable")?
            .canonicalize()
            .wrap_err("resolving macOS user cache directory")?;
        sandbox_profile.push("(allow file-write* file-read* file-map-executable process-exec (subpath \"");
        sandbox_profile.push(cache_root.join("net.java.openjdk.java"));
        sandbox_profile.push("\"))\n");
    }

    for path in command.read_only_paths {
        allow_read(&mut sandbox_profile, &path);
    }
    for path in command.read_write_paths {
        allow_read_write(&mut sandbox_profile, &path);
    }

    sandbox_profile.push(PROTECT);

    let mut sandbox_params = RawStringVec::with_capacity(1);
    let home =
        std::env::var_os("HOME").ok_or_eyre("`HOME` not set in environment")?;
    sandbox_params.push_os("HOME".into())?;
    sandbox_params.push_os(home)?;

    let (pipes, child) = unix::spawn(
        resolved_executable.into(),
        command.args,
        environment,
        command.stdin,
        command.stdout,
        command.stderr,
        command.working_directory,
        Vec::new(),
        env.dev_null,
        CString::new(sandbox_profile.into_encoded_bytes())
            .wrap_err("converting sandbox profile to C-string")?,
        sandbox_params,
    )
    .wrap_err("spawning child")?;

    Ok(crate::SandboxChild {
        stdin: pipes.stdin,
        stdout: pipes.stdout,
        stderr: pipes.stderr,
        imp: SandboxChild::Macos(MacosChild {
            child,
            _temp_dir: temp_dir,
        }),
    })
}

#[derive(Debug)]
pub struct MacosChild {
    child: UnixChild,
    _temp_dir: tempfile::TempDir,
}

#[async_trait]
impl SandboxChildOp for MacosChild {
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

fn confstr(name: libc::c_int) -> Option<Vec<u8>> {
    let size = unsafe { libc::confstr(name, std::ptr::null_mut(), 0) };
    if size <= 0 {
        return None;
    }

    let mut buf = vec![0_u8; size];
    let new_size =
        unsafe { libc::confstr(name, buf.as_mut_ptr().cast(), size) };
    assert_eq!(size, new_size);
    buf.truncate(size - 1);
    Some(buf)
}

fn allow_read(profile: &mut OsString, path: &Path) {
    let Ok(path) = path.canonicalize() else {
        return;
    };
    if path.is_dir() {
        profile.push("(allow file-read* file-map-executable process-exec file-issue-extension (subpath \"");
    } else {
        profile.push("(allow file-read* file-map-executable process-exec file-issue-extension (literal \"");
    }
    profile.push(path);
    profile.push("\"))\n");
}

fn allow_read_write(profile: &mut OsString, path: &Path) {
    let Ok(path) = path.canonicalize() else {
        return;
    };
    if path.is_dir() {
        profile.push("(allow file-write* file-link file-read* file-map-executable process-exec file-issue-extension (subpath \"");
    } else {
        profile.push("(allow file-write* file-read* file-map-executable process-exec file-issue-extension (literal \"");
    }
    profile.push(path);
    profile.push("\"))\n");
}

// View debug logs with `log stream --style compact --predicate 'eventMessage CONTAINS "Sandbox: "`

static BASE_PROFILE: &'static str = r#"
(version 1)
(deny default)
(import "system.sb")

; Debugging
; (debug deny)

; Defines
(define (home path)
  (string-append (param "HOME") path))

; Basic rules
(deny nvram*)
(deny process-info*)
(deny file-link)
(allow hid-control process-fork lsopen)
(allow signal (target same-sandbox))
(allow process-info-pidinfo)
(allow
  process-info-pidfdinfo
  process-info-pidfileportinfo
  process-info-setcontrol
  process-info-dirtycontrol
  process-info-rusage
  process-info-ledger
  (target self))
(allow sysctl-write (sysctl-name "kern.tcsm_enable"))

(system-graphics)

; Allow any symlink to be resolved
(allow file-read-metadata)

; Allow reading some system files
(allow file-read*
  (literal "/private/etc/hosts")
  (literal "/private/etc/passwd")
  (literal "/private/etc/resolv.conf")
  (literal "/private/etc/ssl/cert.pem")
  (literal "/private/etc/ssl/openssl.cnf")
  (literal "/private/var/run/resolv.conf")
  (subpath (home "/Library/Audio"))
  (subpath "/Library/Audio/Plug-Ins")
  (literal (home "/.CFUserTextEncoding")))
(allow file-read* process-exec
  (subpath "/bin")
  (subpath "/sbin")
  (subpath "/usr/bin")
  (subpath "/usr/sbin"))
(allow file-read* file-write*
  (subpath "/dev/fd"))

(allow user-preference-read
  (preference-domain
    "com.apple.MobileAsset"
    "com.apple.HIToolbox"
    "kcfpreferencesanyapplication"
    "net.java.openjdk.java"
    ; Text-to-speech
    "com.apple.accessibility"
    "com.apple.universalaccess"
    "com.apple.speech.recognition.applespeechrecognition.prefs"
    "com.apple.speakselection"
    "com.apple.voiceservices"
    "com.apple.assistant.backedup"
    "com.apple.speech.voice.prefs"))

(allow mach-lookup
  ; Core
  (global-name "com.apple.CoreServices.coreservicesd")
  (global-name "com.apple.coreservices.appleevents")
  (global-name "com.apple.coreservices.launchservicesd")
  (global-name "com.apple.coreservices.quarantine-resolver")
  (global-name "com.apple.DiskArbitration.diskarbitrationd")
  (global-name "com.apple.pasteboard.1")
  (global-name "com.apple.pbs.fetch_services")
  (global-name "com.apple.spindump")
  (global-name "com.apple.mobileassetd")
  (global-name "com.apple.mobileassetd.v2")
  (global-name "com.apple.distributed_notifications@1v3")
  (global-name "com.apple.distributed_notifications@Uv3")
  (global-name "com.apple.SystemConfiguration.configd")
  (global-name "com.apple.uiintelligencesupport.agent")
  (global-name "com.apple.pluginkit.pkd")
  ; TCCD (Popup for microphone, webcam, etc.)
  (global-name "com.apple.tccd")
  (global-name "com.apple.tccd.system")
  ; Input
  (global-name "com.apple.tsm.uiserver")
  (global-name "com.apple.inputanalyticsd")
  (global-name "com.apple.inputmethodkit.launchagent")
  (global-name "com.apple.inputmethodkit.launcher")
  (global-name "com.apple.inputmethodkit.getxpcendpoint")
  (global-name "com.apple.iohideventsystem")
  (global-name "com.apple.touchbarserver.mig")
  ; Windowing
  (global-name "com.apple.windowmanager.server")
  (global-name "com.apple.windowserver.active")
  (global-name "com.apple.window_proxies")
  (global-name "com.apple.dock.server")
  (global-name "com.apple.dock.fullscreen")
  ; Fonts
  (global-name "com.apple.fonts")
  (global-name "com.apple.FontObjectsServer")
  ; Audio
  (global-name "com.apple.audio.AudioComponentPrefs")
  (global-name "com.apple.audio.AudioComponentRegistrar")
  (global-name "com.apple.audio.AudioSession")
  (global-name "com.apple.audio.audiohald")
  (global-name "com.apple.audio.coreaudiod")
  (global-name "com.apple.audioanalyticsd"))

; Audio Input/Output
(allow device-microphone)
(allow iokit-open-user-client
  (require-all
    (iokit-connection "AppleHDAEngineInput")
      (iokit-user-client-class
        "IOAudioControlUserClient"
        "IOAudioEngineUserClient")))
(allow mach-lookup
  (xpc-service-name "com.apple.audio.AudioConverterService")
  (global-name "com.apple.cmio.registerassistantservice.system-extensions")
  (global-name "com.apple.relatived.public")
  (global-name "com.apple.relatived.status")
  (global-name "com.apple.relatived.tempest"))
"#;

static PROTECT: &'static str = r#"
(deny network-outbound (literal "/private/var/run/cupsd"))
(deny network-outbound (remote ip "localhost:631"))
(deny file-write-xattr (xattr "com.apple.quarantine") (with no-log))
(deny file-read-xattr file-write-xattr (xattr-prefix "com.apple.security.private."))
"#;

static NETWORK: &'static str = r#"
(system-network)
(allow network-outbound (literal "/private/var/run/mDNSResponder"))
(allow network-outbound (remote ip))
(allow network-inbound (local ip))
(allow mach-lookup
       (global-name
         "com.apple.NetworkDiagnostic.agent"
         "com.apple.WebKit.PluginAgent"
         "com.apple.airportd"
         "com.apple.cfnetwork.AuthBrokerAgent"
         "com.apple.cfnetwork.cfnetworkagent"
         "com.apple.corewlan-xpc"
         "com.apple.nesessionmanager.content-filter"
         "com.apple.networkserviceproxy.fetch-token"
         "com.apple.nsurlsessiond"))
"#;

// static ALLOWED_ENV_VARS: Lazy<FxHashSet<&'static OsStr>> = Lazy::new(|| {
//     [
//         "TMPDIR", "PATH", "HOME", "LANG", "LC_ALL", "TERM", "USER", "USERNAME",
//     ]
//     .iter()
//     .map(OsStr::new)
//     .collect()
// });

// pub fn should_pass_env_var(var: &OsStr) -> bool {
//     return ALLOWED_ENV_VARS.contains(var);
// }
