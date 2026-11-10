use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct GameVersionName(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameVersion {
    pub version: GameVersionName,
    pub stable: bool,
}

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct LoaderVersionName(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameLoaderProfile {
    pub id: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn game_versions_preserve_upstream_shape() {
        let value = json!({"version": "1.21", "stable": true});
        let game: GameVersion = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(game.version, GameVersionName("1.21".into()));
        assert_eq!(serde_json::to_value(game).unwrap(), value);
        assert!(
            serde_json::from_value::<GameVersion>(json!({"version": "1.21"}))
                .is_err()
        );
    }

    #[test]
    fn profiles_require_id_and_accept_extra_fields() {
        let profile: GameLoaderProfile = serde_json::from_value(json!({
            "id": "fabric-loader-0.19.5-1.21",
            "libraries": []
        }))
        .unwrap();
        assert_eq!(profile.id, "fabric-loader-0.19.5-1.21");
        assert!(
            serde_json::from_value::<GameLoaderProfile>(json!({})).is_err()
        );
    }
}
