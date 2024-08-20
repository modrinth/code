use tauri_build::{DefaultPermissionRule, InlinedPlugin};

fn main() {
    // ToDo: Cleanup this & reduce boilerplate.

    // ToDo: Cleanup commands for other plugins than auth, import
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .codegen(tauri_build::CodegenContext::new())
            .plugin(
                "auth",
                InlinedPlugin::new().commands(&["login", "get_default_user", "set_default_user", "remove_user", "get_users"]).default_permission(DefaultPermissionRule::AllowAllCommands),
            )
            .plugin(
                "import",
                InlinedPlugin::new().commands(&["get_importable_instances", "import_instance", "is_valid_importable_instance", "get_default_launcher_path"]).default_permission(DefaultPermissionRule::AllowAllCommands),
            )
            .plugin(
                "jre",
                InlinedPlugin::new().commands(&["jre_find_filtered_jres", "jre_get_jre", "jre_test_jre", "jre_auto_install_java", "jre_get_max_memory", "get_java_versions"]).default_permission(DefaultPermissionRule::AllowAllCommands),
            )
            .plugin(
                "logs",
                InlinedPlugin::new().commands(&["logs_get_logs", "logs_get_logs_by_filename", "logs_get_output_by_filename", "logs_delete_logs", "logs_delete_logs_by_filename", "logs_get_latest_log_cursor"]).default_permission(DefaultPermissionRule::AllowAllCommands),
            )
            .plugin(
                "metadata",
                InlinedPlugin::new().commands(&["metadata_get_game_versions", "metadata_get_loader_versions"]).default_permission(DefaultPermissionRule::AllowAllCommands)
            )
            .plugin(
                "mr-auth",
                InlinedPlugin::new().commands(&["authenticate_begin_flow", "authenticate_await_completion", "cancel_flow", "login_pass", "login_2fa", "create_account", "refresh", "logout", "get"]).default_permission(DefaultPermissionRule::AllowAllCommands)
            )
            .plugin(
                "pack",
                InlinedPlugin::new().commands(&["pack_install", "pack_get_profile_from_pack"]).default_permission(DefaultPermissionRule::AllowAllCommands)
            )
            .plugin(
                "process",
                InlinedPlugin::new().commands(&["process_get_all", "process_get_by_profile_path", "process_kill", "process_wait_for"]).default_permission(DefaultPermissionRule::AllowAllCommands)
            )
            .plugin(
                "profile",
                InlinedPlugin::new().commands(&["profile_remove", "profile_get", "profile_get_optimal_jre_key", "profile_get_full_path", "profile_get_mod_full_path", "profile_list", "profile_check_installed", "profile_install", "profile_update_all", "profile_update_project", "profile_add_project_from_version", "profile_add_project_from_path", "profile_toggle_disable_project", "profile_remove_project", "profile_update_managed_modrinth_version", "profile_repair_managed_modrinth", "profile_run", "profile_run_credentials", "profile_kill", "profile_edit", "profile_edit_icon", "profile_export_mrpack", "profile_get_pack_export_candidates", "profile_get_many", "profile_get_projects"]).default_permission(DefaultPermissionRule::AllowAllCommands)
            )
            .plugin(
                "profile-create",
                InlinedPlugin::new().commands(&["profile_create", "profile_duplicate"]).default_permission(DefaultPermissionRule::AllowAllCommands)
            )
            .plugin(
                "settings",
                InlinedPlugin::new().commands(&["settings_get", "settings_set", "settings_change_config_dir", "settings_is_dir_writeable"]).default_permission(DefaultPermissionRule::AllowAllCommands)
            )
            .plugin(
                "tags",
                InlinedPlugin::new().commands(&["tags_get_categories", "tags_get_report_types", "tags_get_loaders", "tags_get_game_versions", "tags_get_donation_platforms", "tags_get_tag_bundle"]).default_permission(DefaultPermissionRule::AllowAllCommands)
            )
            .plugin(
                "utils",
                InlinedPlugin::new().commands(&["get_os", "should_disable_mouseover", "show_in_folder", "show_launcher_logs_folder", "progress_bars_list", "safety_check_safe_loading_bars", "get_opening_command", "await_sync", "is_offline", "refresh_offline"]).default_permission(DefaultPermissionRule::AllowAllCommands)
            )
            .plugin(
                "cache",
                InlinedPlugin::new().commands(&["get_project", "get_project_many", "get_version", "get_version_many", "get_user", "get_user_many", "get_team", "get_team_many", "get_organization", "get_organization_many", "get_search_results", "get_search_results_many", "purge_cache_bytes"]).default_permission(DefaultPermissionRule::AllowAllCommands)
            )
    )
        .expect("Failed to run tauri-build");
}
