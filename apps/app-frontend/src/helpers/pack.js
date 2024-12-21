/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank Profile object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/core'
import { create } from './profile'

// Installs pack from a version ID
export async function install(projectId, versionId, packTitle, iconUrl) {
  const location = {
    type: 'fromVersionId',
    project_id: projectId,
    version_id: versionId,
    title: packTitle,
    icon_url: iconUrl,
  }
  const profile_creator = await invoke('plugin:pack|pack_get_profile_from_pack', { location })
  const profile = await create(
    profile_creator.name,
    profile_creator.gameVersion,
    profile_creator.modloader,
    profile_creator.loaderVersion,
    null,
    true,
  )

  return await invoke('plugin:pack|pack_install', { location, profile })
}

// Installs pack from a path
export async function install_from_file(path) {
  const location = {
    type: 'fromFile',
    path: path,
  }
  const profile_creator = await invoke('plugin:pack|pack_get_profile_from_pack', { location })
  const profile = await create(
    profile_creator.name,
    profile_creator.gameVersion,
    profile_creator.modloader,
    profile_creator.loaderVersion,
    null,
    true,
  )
  return await invoke('plugin:pack|pack_install', { location, profile })
}
