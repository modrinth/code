//! Compatibility utilities for V3 API.

use crate::models::exp::ProjectSerial;

const MODPACK: &str = "modpack";
const MINECRAFT_JAVA_SERVER: &str = "minecraft_java_server";

/// Adjusts V3 project types based on a project's components.
///
/// The experimental API does not have a concept of project types; instead, a
/// project's "type" is implicit based on what components it has.
/// To reflect this in the V3 API, we manually add `project_types` values
/// for compatibility with stuff like searching.
pub fn correct_project_types(
    components: &ProjectSerial,
    project_types: &mut Vec<String>,
) {
    if components.minecraft_server.is_some() {
        // remove modpack type to reduce burden on frontend
        project_types.retain(|t| t != MODPACK);
        project_types.push(MINECRAFT_JAVA_SERVER.into());
    }
}

#[cfg(test)]
mod tests {
    use super::{MINECRAFT_JAVA_SERVER, MODPACK, correct_project_types};
    use crate::models::exp::{ProjectSerial, minecraft::ServerProject};

    fn server_components() -> ProjectSerial {
        ProjectSerial {
            minecraft_server: Some(ServerProject {
                max_players: None,
                country: None,
                languages: vec![],
                active_version: None,
            }),
            ..ProjectSerial::default()
        }
    }

    #[test]
    fn adds_java_server_type_and_removes_modpack_for_server_projects() {
        let components = server_components();
        let mut project_types =
            vec!["mod".to_string(), MODPACK.to_string(), "plugin".to_string()];

        correct_project_types(&components, &mut project_types);

        assert_eq!(
            project_types,
            vec![
                "mod".to_string(),
                "plugin".to_string(),
                MINECRAFT_JAVA_SERVER.to_string()
            ]
        );
    }

    #[test]
    fn leaves_project_types_unchanged_without_server_component() {
        let components = ProjectSerial::default();
        let mut project_types = vec!["mod".to_string(), MODPACK.to_string()];
        let expected = project_types.clone();

        correct_project_types(&components, &mut project_types);

        assert_eq!(project_types, expected);
    }
}
