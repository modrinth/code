//! Compatibility utilities for V3 API.

use crate::models::exp::ProjectSerial;

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
        project_types.push(MINECRAFT_JAVA_SERVER.into());
    }
}
