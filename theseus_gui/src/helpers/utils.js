import { add_project_from_version as installMod } from '@/helpers/profile'
import { ofetch } from 'ofetch'

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

export const checkInstalled = (profile, projectId) => {
  return Object.values(profile.projects).some((p) => p.metadata?.project?.id === projectId)
}

export const installVersionDependencies = async (profile, version) => {
  for (const dep of version.dependencies) {
    if (dep.version_id) {
      if (checkInstalled(profile, dep.project_id)) continue
      await installMod(profile.path, dep.version_id)
    } else {
      if (checkInstalled(profile, dep.project_id)) continue
      const depVersions = await ofetch(
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
