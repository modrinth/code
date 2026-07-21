use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

pub const REQUEST_CONTEXT_HEADER: &str = "Modrinth-App-Request-Context";

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum OperationCause {
    #[serde(rename = "app/startup")]
    AppStartup,
    #[serde(rename = "navigation/home")]
    NavigationHome,
    #[serde(rename = "navigation/library")]
    NavigationLibrary,
    #[serde(rename = "navigation/browse")]
    NavigationBrowse,
    #[serde(rename = "navigation/project")]
    NavigationProject,
    #[serde(rename = "navigation/instance/overview")]
    NavigationInstanceOverview,
    #[serde(rename = "navigation/instance/content")]
    NavigationInstanceContent,
    #[serde(rename = "navigation/instance/logs")]
    NavigationInstanceLogs,
    #[serde(rename = "navigation/servers")]
    NavigationServers,
    #[serde(rename = "navigation/server/manage")]
    NavigationServerManage,
    #[serde(rename = "navigation/server/content")]
    NavigationServerContent,
    #[serde(rename = "instance/refresh/user")]
    InstanceRefreshUser,
    #[serde(rename = "instance/refresh/filesystem_watch")]
    InstanceRefreshFilesystemWatch,
    #[serde(rename = "instance/update/all")]
    InstanceUpdateAll,
    #[serde(rename = "instance/update/single")]
    InstanceUpdateSingle,
    #[serde(rename = "instance/install")]
    InstanceInstall,
    #[serde(rename = "cache/revalidate")]
    CacheRevalidate,
    #[serde(rename = "auth/session_refresh")]
    AuthSessionRefresh,
    #[serde(rename = "background/friends")]
    BackgroundFriends,
    #[serde(rename = "minecraft/launch")]
    MinecraftLaunch,
    #[serde(rename = "app/update_check")]
    AppUpdateCheck,
    #[serde(rename = "unattributed")]
    Unattributed,
}

impl OperationCause {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AppStartup => "app/startup",
            Self::NavigationHome => "navigation/home",
            Self::NavigationLibrary => "navigation/library",
            Self::NavigationBrowse => "navigation/browse",
            Self::NavigationProject => "navigation/project",
            Self::NavigationInstanceOverview => "navigation/instance/overview",
            Self::NavigationInstanceContent => "navigation/instance/content",
            Self::NavigationInstanceLogs => "navigation/instance/logs",
            Self::NavigationServers => "navigation/servers",
            Self::NavigationServerManage => "navigation/server/manage",
            Self::NavigationServerContent => "navigation/server/content",
            Self::InstanceRefreshUser => "instance/refresh/user",
            Self::InstanceRefreshFilesystemWatch => {
                "instance/refresh/filesystem_watch"
            }
            Self::InstanceUpdateAll => "instance/update/all",
            Self::InstanceUpdateSingle => "instance/update/single",
            Self::InstanceInstall => "instance/install",
            Self::CacheRevalidate => "cache/revalidate",
            Self::AuthSessionRefresh => "auth/session_refresh",
            Self::BackgroundFriends => "background/friends",
            Self::MinecraftLaunch => "minecraft/launch",
            Self::AppUpdateCheck => "app/update_check",
            Self::Unattributed => "unattributed",
        }
    }
}

impl fmt::Display for OperationCause {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("invalid operation cause: {0}")]
pub struct InvalidOperationCause(String);

impl FromStr for OperationCause {
    type Err = InvalidOperationCause;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let cause = match value {
            "app/startup" => Self::AppStartup,
            "navigation/home" => Self::NavigationHome,
            "navigation/library" => Self::NavigationLibrary,
            "navigation/browse" => Self::NavigationBrowse,
            "navigation/project" => Self::NavigationProject,
            "navigation/instance/overview" => Self::NavigationInstanceOverview,
            "navigation/instance/content" => Self::NavigationInstanceContent,
            "navigation/instance/logs" => Self::NavigationInstanceLogs,
            "navigation/servers" => Self::NavigationServers,
            "navigation/server/manage" => Self::NavigationServerManage,
            "navigation/server/content" => Self::NavigationServerContent,
            "instance/refresh/user" => Self::InstanceRefreshUser,
            "instance/refresh/filesystem_watch" => {
                Self::InstanceRefreshFilesystemWatch
            }
            "instance/update/all" => Self::InstanceUpdateAll,
            "instance/update/single" => Self::InstanceUpdateSingle,
            "instance/install" => Self::InstanceInstall,
            "cache/revalidate" => Self::CacheRevalidate,
            "auth/session_refresh" => Self::AuthSessionRefresh,
            "background/friends" => Self::BackgroundFriends,
            "minecraft/launch" => Self::MinecraftLaunch,
            "app/update_check" => Self::AppUpdateCheck,
            "unattributed" => Self::Unattributed,
            _ => return Err(InvalidOperationCause(value.to_string())),
        };

        Ok(cause)
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvocationContext {
    pub cause: OperationCause,
}

impl InvocationContext {
    pub fn into_operation_context(self) -> OperationContext {
        OperationContext::new(self.cause)
    }
}

#[derive(Clone, Debug)]
pub struct OperationContext {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub cause: OperationCause,
}

impl OperationContext {
    pub fn new(cause: OperationCause) -> Self {
        Self {
            id: Uuid::new_v4(),
            parent_id: None,
            cause,
        }
    }

    pub fn child(&self, cause: OperationCause) -> Self {
        Self {
            id: Uuid::new_v4(),
            parent_id: Some(self.id),
            cause,
        }
    }

    pub const fn cause(&self) -> OperationCause {
        self.cause
    }

    pub const fn request_context_header(&self) -> &'static str {
        self.cause.as_str()
    }

    pub fn referer(&self) -> String {
        format!("https://tauri.modrinth.app/_rc/{}", self.cause.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn causes_accept_only_the_stable_taxonomy() {
        assert_eq!(
            "navigation/instance/content".parse(),
            Ok(OperationCause::NavigationInstanceContent)
        );
        assert!("Navigation/Project".parse::<OperationCause>().is_err());
        assert!(
            "navigation/project/user-value"
                .parse::<OperationCause>()
                .is_err()
        );
        assert!("navigation//project".parse::<OperationCause>().is_err());
        assert!("navigation/project/".parse::<OperationCause>().is_err());
        assert!("a".repeat(256).parse::<OperationCause>().is_err());
    }

    #[test]
    fn serde_does_not_normalize_invalid_causes() {
        assert_eq!(
            serde_json::from_str::<OperationCause>(
                r#""navigation/instance/content""#
            )
            .unwrap(),
            OperationCause::NavigationInstanceContent
        );
        assert!(
            serde_json::from_str::<OperationCause>(
                r#""Navigation/Instance/Content""#
            )
            .is_err()
        );
    }

    #[test]
    fn child_context_links_to_its_parent() {
        let parent = OperationContext::new(OperationCause::InstanceInstall);
        let child = parent.child(OperationCause::CacheRevalidate);

        assert_ne!(parent.id, child.id);
        assert_eq!(child.parent_id, Some(parent.id));
        assert_eq!(child.cause(), OperationCause::CacheRevalidate);
    }

    #[test]
    fn invocation_context_accepts_only_a_cause() {
        assert!(
            serde_json::from_str::<InvocationContext>(
                r#"{"cause":"navigation/home","id":"frontend-id"}"#
            )
            .is_err()
        );
    }

    #[test]
    fn headers_have_exact_semantic_values() {
        let context =
            OperationContext::new(OperationCause::NavigationInstanceContent);

        assert_eq!(
            context.referer(),
            "https://tauri.modrinth.app/_rc/navigation/instance/content"
        );
        assert_eq!(
            context.request_context_header(),
            "navigation/instance/content"
        );
    }

    #[test]
    fn concurrent_roots_remain_distinct() {
        let first = OperationContext::new(OperationCause::NavigationBrowse);
        let second = OperationContext::new(OperationCause::MinecraftLaunch);

        assert_ne!(first.id, second.id);
        assert_ne!(first.cause(), second.cause());
        assert_eq!(first.parent_id, None);
        assert_eq!(second.parent_id, None);
    }

    #[tokio::test]
    async fn spawned_watcher_and_cache_work_retains_explicit_contexts() {
        let watcher = OperationContext::new(
            OperationCause::InstanceRefreshFilesystemWatch,
        );
        let cache = watcher.child(OperationCause::CacheRevalidate);
        let watcher_id = watcher.id;

        let watcher_task = tokio::spawn(async move {
            (watcher.id, watcher.parent_id, watcher.cause())
        });
        let cache_task =
            tokio::spawn(async move { (cache.parent_id, cache.cause()) });

        assert_eq!(
            watcher_task.await.unwrap(),
            (
                watcher_id,
                None,
                OperationCause::InstanceRefreshFilesystemWatch
            )
        );
        assert_eq!(
            cache_task.await.unwrap(),
            (Some(watcher_id), OperationCause::CacheRevalidate)
        );
    }
}
