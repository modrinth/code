import { mockIPC } from '@tauri-apps/api/mocks'
import { ofetch } from 'ofetch'
import {
  Tags,
  get_categories,
  get_donation_platforms,
  get_game_versions,
  get_loaders,
  get_report_types,
} from './helpers/tags'
import { Settings } from './helpers/settings'
import { JavaVersion } from './helpers/jre'
import { Profile } from './helpers/profile'

const settings: { value: Settings } = {
  value: {
    collapsed_navigation: false,
    custom_env_args: [],
    custom_java_args: [],
    game_resolution: [0, 0],
    hooks: {},
    java_globals: new Map(),
    max_concurrent_downloads: -1,
    memory: {
      maximum: 4096,
    },
    version: 0,
  },
}

export const createWebTauri = () => {
  mockIPC(async (cmd, args) => {
    switch (cmd) {
      case 'initialize_state':
        break

      case 'progress_bars_list':
        return []

      case 'profile_create_empty':
        return '/'

      case 'profile_create':
        return '/'

      case 'profile_remove':
        return

      case 'profile_get':
        return {
          game_version: '1.20',
          loader_version: 'latest',
          modloader: 'quilt',
          name: 'Test',
        } as Profile

      case 'profile_get_optimal_jre_key':
        return '/'

      case 'profile_list':
        return {
          '/': {
            game_version: '1.20',
            loader_version: 'latest',
            modloader: 'quilt',
            name: 'Test',
          },
        } as unknown as Map<string, Profile>

      case 'profile_install':
      case 'profile_update_all':
      case 'profile_update_project':
      case 'profile_add_project_from_version':
      case 'profile_add_project_from_path':
        return '/'

      case 'profile_run':
      case 'profile_run_wait':
      case 'profile_run_credentials':
      case 'profile_run_wait_credentials':
      case 'profile_edit':
      case 'profile_edit_icon':
      case 'profile_check_installed':
        break

      case 'pack_install_version_id':
      case 'pack_install_file':
        break

      case 'auth_authenticate_begin_flow':
      case 'auth_authenticate_await_completion':
      case 'auth_cancel_flow':
      case 'auth_refresh':
      case 'auth_remove_user':
      case 'auth_has_user':
      case 'auth_users':
      case 'auth_get_user':
        return []

      case 'tags_get_categories':
        return await ofetch('/v2/tag/category')

      case 'tags_get_donation_platforms':
        return await ofetch('/v2/tag/donation_platform')

      case 'tags_get_game_versions':
        return await ofetch('/v2/tag/game_version')

      case 'tags_get_loaders':
        return await ofetch('/v2/tag/loader')

      case 'tags_get_report_types':
        return await ofetch('/v2/tag/report_type')

      case 'tags_get_tag_bundle':
        return {
          categories: await get_categories(),
          donation_platforms: await get_donation_platforms(),
          game_versions: await get_game_versions(),
          loaders: await get_loaders(),
          report_types: await get_report_types(),
        } as Tags

      case 'settings_get':
        return settings.value

      case 'settings_set':
        settings.value = args[0] as Settings
        return

      case 'jre_get_all_jre':
        return []

      case 'jre_autodetect_java_globals':
        return new Map()

      case 'jre_find_jre_18plus_jres':
        return []

      case 'jre_find_jre_17_jres':
        return []

      case 'jre_find_jre_8_jres':
        return []

      case 'jre_validate_globals':
        return true

      case 'jre_get_jre':
        return {
          path: '/',
          version: '1.8',
        } as JavaVersion

      case 'jre_auto_install_java':
        return '/'

      case 'jre_get_max_memory':
        return 8192

      case 'process_get_all_uuids':
      case 'process_get_all_running_uuids':
      case 'process_get_uuids_by_profile_path':
      case 'process_get_all_running_profile_paths':
      case 'process_get_all_running_profiles':
      case 'process_get_exit_status_by_uuid':
      case 'process_has_finished_by_uuid':
      case 'process_get_stderr_by_uuid':
      case 'process_get_stdout_by_uuid':
      case 'process_kill_by_uuid':
      case 'process_wait_for_by_uuid':
        break

      case 'metadata_get_game_versions':
      case 'metadata_get_fabric_versions':
      case 'metadata_get_forge_versions':
      case 'metadata_get_quilt_versions':
        return []

      case 'logs_get_logs':
      case 'logs_get_logs_by_datetime':
        return []

      case 'logs_get_stdout_by_datetime':
      case 'logs_get_stderr_by_datetime':
        return ''

      case 'logs_delete_logs':
      case 'logs_delete_logs_by_datetime':
        break

      case 'show_in_folder':
      case 'should_disable_mouseover':
        return false

      default:
        return
    }
  })
}
