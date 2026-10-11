use serde::{Deserialize, Serialize};
use std::env;
use std::sync::OnceLock;

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DevAppConfig {
    pub credential_namespace: String,
    pub auth_mode: String,
    pub branch: String,
    pub title: String,
    pub data_dir: String,
    pub backend_url: Option<String>,
    pub username: Option<String>,
}

static CONFIG: OnceLock<Option<DevAppConfig>> = OnceLock::new();

pub fn prepare() {
    #[cfg(debug_assertions)]
    let config = parse_config();
    #[cfg(not(debug_assertions))]
    let config = None;
    let backend_url = config
        .as_ref()
        .and_then(|config| config.backend_url.clone())
        .or_else(|| env::var("ACCOUNT_API_URL").ok());
    if let Some(backend_url) = backend_url {
        theseus::self_hosted::accounts::configure(
            &backend_url,
            env::var("ACCOUNT_WEB_URL").ok().as_deref(),
        )
        .expect("valid account service configuration");
    }
    let _ = CONFIG.set(config);
}

pub fn config() -> Option<&'static DevAppConfig> {
    CONFIG.get().and_then(Option::as_ref)
}

#[cfg(debug_assertions)]
fn parse_config() -> Option<DevAppConfig> {
    let mut args = env::args();
    while let Some(arg) = args.next() {
        if arg != "--amberite-dev-config" {
            continue;
        }
        let raw = args
            .next()
            .unwrap_or_else(|| panic!("--amberite-dev-config requires JSON"));
        let config: DevAppConfig =
            serde_json::from_str(&raw).unwrap_or_else(|error| {
                panic!("invalid Amberite dev config: {error}")
            });
        assert!(
            !config.branch.trim().is_empty(),
            "dev branch cannot be empty"
        );
        assert!(
            !config.credential_namespace.trim().is_empty(),
            "dev credential namespace cannot be empty"
        );
        assert!(
            !config.data_dir.trim().is_empty(),
            "dev data directory cannot be empty"
        );
        unsafe {
            env::set_var("THESEUS_CONFIG_DIR", &config.data_dir);
            env::set_var(
                "WEBVIEW2_USER_DATA_FOLDER",
                format!("{}/webview2", config.data_dir),
            );
        }
        return Some(config);
    }
    None
}
