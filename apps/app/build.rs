use tauri_build::InlinedPlugin;

fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .codegen(tauri_build::CodegenContext::new())
            .plugin(
                "auth",
                InlinedPlugin::new().commands(&["auth_get_default_user", "auth_set_default_user", "auth_remove_user", "auth_users", "auth_get_user"]),
            )
            .plugin(
                "import",
                InlinedPlugin::new().commands(&["import_get_importable_instances", "import_import_instance", "import_is_valid_importable_instance", "import_get_default_launcher_path"]),
            )
            .plugin(
                "jre",
                InlinedPlugin::new().commands(&["jre_find_filtered_jres", "jre_get_jre", "jre_test_jre", "jre_auto_install_java", "jre_get_max_memory"]),
            )
            .plugin(
                "logs",
                InlinedPlugin::new().commands(&["logs_get_logs", "logs_get_logs_by_filename", "logs_get_output_by_filename", "logs_delete_logs", "logs_delete_logs_by_filename", "logs_get_latest_log_cursor"]),
            )
            .plugin(
                "metadata",
                InlinedPlugin::new().commands(&["metadata_get_game_versions", "metadata_get_loader_versions"])
            )
            .plugin(
                "mr-auth",
                InlinedPlugin::new().commands(&["authenticate_begin_flow", "authenticate_await_completion", "cancel_flow", "login_pass", "login_2fa", "create_account", "refresh", "logout", "get"])
            )
            .plugin(
                "pack",
                InlinedPlugin::new().commands(&["pack_install", "pack_get_profile_from_pack"])
            )
            .plugin(
                "process",
                tauri_build::InlinedPlugin::new().commands(&["process_get_all", "process_get_by_profile_path", "process_kill", "process_wait_for"])
            )
            .plugin(
                "profile",
                InlinedPlugin::new().commands(&["profile_remove", "profile_get", "profile_get_optimal_jre_key", "profile_get_full_path", "profile_get_mod_full_path", "profile_list", "profile_check_installed", "profile_install", "profile_update_all", "profile_update_project", "profile_add_project_from_version", "profile_add_project_from_path", "profile_toggle_disable_project", "profile_remove_project", "profile_update_managed_modrinth_version", "profile_repair_managed_modrinth", "profile_run", "profile_run_wait", "profile_run_credentials", "profile_run_wait_credentials", "profile_edit", "profile_edit_icon", "profile_export_mrpack", "profile_get_pack_export_candidates", "profile_get_many", "profile_get_projects"])
            )
            .plugin(
                "profile-create",
                InlinedPlugin::new().commands(&["profile_create", "profile_duplicate"])
            )
            .plugin(
                "settings",
                InlinedPlugin::new().commands(&["settings_get", "settings_set", "settings_change_config_dir", "settings_is_dir_writeable"])
            )
            .plugin(
                "tags",
                InlinedPlugin::new().commands(&["tags_get_categories", "tags_get_report_types", "tags_get_loaders", "tags_get_game_versions", "tags_get_donation_platforms", "tags_get_tag_bundle"])
            )
            .plugin(
                "utils",
                InlinedPlugin::new().commands(&["get_os", "should_disable_mouseover", "show_in_folder", "show_launcher_logs_folder", "progress_bars_list", "safety_check_safe_loading_bars", "get_opening_command", "await_sync", "is_offline", "refresh_offline"])
            )
            .plugin(
                "cache",
                InlinedPlugin::new().commands(&["get_project", "get_project_many", "get_version", "get_version_many", "get_user", "get_user_many", "get_team", "get_team_many", "get_organization", "get_organization_many", "get_search_results", "get_search_results_many", "purge_cache_bytes"])
            )
    )
        .expect("Failed to run tauri-build");
}
