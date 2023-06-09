import { add_project_from_version as installMod, check_installed } from '@/helpers/profile'
import { useFetch } from '@/helpers/fetch.js'
import { handleError } from '@/store/notifications.js'
import { invoke } from '@tauri-apps/api/tauri'
import { Hooks, JavaSettings, MemorySettings, WindowSize } from './settings'
import { LoaderVersion } from './manifest'

export type ModLoader = 'vanilla' | 'forge' | 'fabric' | 'quilt'

export interface LinkedData {
  project_id?: string
  version_id?: string
}

export interface ProfileMetadata {
  name: string
  icon?: string
  icon_url?: string
  groups: string[]

  game_version: string
  loader: ModLoader
  loader_version?: LoaderVersion

  linked_data?: LinkedData

  date_created: number
  date_modified: number
  last_played?: number
}

export interface Profile {
  // TODO: will be used in restructure to refer to profiles
  uuid: string
  install_stage: 'installed' | 'installing' | 'PackInstalling' | 'NotInstalled'
  path: string
  metadata: ProfileMetadata
  java?: JavaSettings
  memory?: MemorySettings
  resolution?: WindowSize
  hooks?: Hooks
  // TODO: Add the `Project` type as an interface
  projects: Map<string, any>
}

export interface ModrinthVersionFile {
  hashes: Map<string, string>
  url: string
  filename: string
  primary: boolean
  size: number
  file_type?: 'RequiredResourcePack' | 'OptionalResourcePack' | unknown
}

export interface Dependency {
  version_id?: string
  project_id?: string
  file_name?: string
  dependency_type: 'required' | 'optional' | 'incompatible' | 'embedded'
}

export interface ModrinthVersion {
  id: string
  project_id: string
  author_id: string

  featured: boolean

  name: string
  version_number: string
  changelog: string
  changelog_url?: string

  date_published: number
  downloads: number
  version_type: string

  files: ModrinthVersionFile[]
  dependencies: Dependency[]
  game_versions: string[]
  loaders: string[]
}

export async function showInFolder(path: string) {
  return await invoke('show_in_folder', { path })
}

export const releaseColor = (releaseType: 'release' | 'beta' | 'alpha' | string): string => {
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

export const installVersionDependencies = async (profile: Profile, version: ModrinthVersion) => {
  for (const dep of version.dependencies) {
    if (dep.dependency_type !== 'required') continue
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
        (v: ModrinthVersion) =>
          v.game_versions.includes(profile.metadata.game_version) &&
          v.loaders.includes(profile.metadata.loader)
      )
      await installMod(profile.path, latest.id).catch(handleError)
    }
  }
}

export const getBaseUrl = () => {
  return import.meta.env.TAURI_WEB_DEV ? "" : "https://api.modrinth.com"
};
