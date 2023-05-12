import { add_project_from_version as installMod, check_installed } from '@/helpers/profile'
import useFetch from '@/helpers/fetch'

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
    if (dep.version_id) {
      if (await check_installed(profile.path, dep.project_id)) continue
      await installMod(profile.path, dep.version_id)
    } else {
      if (await check_installed(profile.path, dep.project_id)) continue
      const depVersions = await useFetch(
        `https://api.modrinth.com/v2/project/${dep.project_id}/version`
      )
      const latest = depVersions.find(
        (v) =>
          v.game_versions.includes(profile.metadata.game_version) &&
          v.loaders.includes(profile.metadata.loader)
      )
      await installMod(profile.path, latest.id)
    }
  }
}
