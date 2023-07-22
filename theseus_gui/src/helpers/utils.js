import { add_project_from_version as installMod, check_installed } from '@/helpers/profile'
import { useFetch } from '@/helpers/fetch.js'
import { handleError } from '@/store/notifications.js'
import { invoke } from '@tauri-apps/api/tauri'

export async function isDev() {
  return await invoke('is_dev')
}

export async function showInFolder(path) {
  return await invoke('plugin:utils|show_in_folder', { path })
}

export const releaseColor = (releaseType) => {
  switch (releaseType) {
    case 'release':
      return 'green'
    case 'beta':
      return 'orange'
    case 'alpha':
      return 'red'
    default:
      return ''
  }
}

export const installVersionDependencies = async (profile, version) => {
  for (const dep of version.dependencies) {
    if (dep.dependency_type !== 'required') continue
    // disallow fabric api install on quilt
    if (dep.project_id === 'P7dR8mSH' && profile.metadata.loader === 'quilt') continue
    if (dep.version_id) {
      if (
        dep.project_id &&
        (await check_installed(profile.path, dep.project_id).catch(handleError))
      )
        continue
      await installMod(profile.path, dep.version_id)
    } else {
      if (
        dep.project_id &&
        (await check_installed(profile.path, dep.project_id).catch(handleError))
      )
        continue
      const depVersions = await useFetch(
        `https://api.modrinth.com/v2/project/${dep.project_id}/version`,
        'dependency versions'
      )
      const latest = depVersions.find(
        (v) =>
          v.game_versions.includes(profile.metadata.game_version) &&
          v.loaders.includes(profile.metadata.loader)
      )
      if (latest) {
        await installMod(profile.path, latest.id).catch(handleError)
      }
    }
  }
}

export const openLink = (url) => {
  window.__TAURI_INVOKE__('tauri', {
    __tauriModule: 'Shell',
    message: {
      cmd: 'open',
      path: url,
    },
  })
}
