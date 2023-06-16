/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank Profile object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/tauri'
import { create } from './profile'

// Installs pack from a version ID
export async function install(projectId, versionId, packTitle, packIcon) {
  const location = {
    type: 'FromVersionId',
    projectId: projectId,
    versionId: versionId,
    packTitle: packTitle,
    packIcon: packIcon,
  }
  const profile_creator = await invoke('pack_get_profile_from_pack', { location })
  const profile = await create(
    profile_creator.name,
    profile_creator.gameVersion,
    profile_creator.modloader,
    profile_creator.loaderVersion,
    profile_creator.icon
  )

  return await invoke('pack_install', { location, profile })
}

// Installs pack from a path
export async function install_from_file(path) {
  const location = {
    type: 'FromFile',
    path: path,
  }
  const profile_creator = await invoke('pack_get_profile_from_pack', { location })
  const profile = await create(
    profile_creator.name,
    profile_creator.gameVersion,
    profile_creator.modloader,
    profile_creator.loaderVersion,
    profile_creator.icon
  )
  return await invoke('pack_install', { location, profile })
}
