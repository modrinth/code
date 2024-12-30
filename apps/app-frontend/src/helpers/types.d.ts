import type { ModrinthId } from '@modrinth/utils'

type GameInstance = {
  path: string
  install_stage: InstallStage

  name: string
  icon_path?: string

  game_version: string
  loader: InstanceLoader
  loader_version?: string

  groups: string[]

  linked_data?: LinkedData

  created: Date
  modified: Date
  last_played?: Date

  submitted_time_played: number
  recent_time_played: number

  java_path?: string
  extra_launch_args?: string[]
  custom_env_vars?: [string, string][]

  memory?: MemorySettings
  force_fullscreen?: boolean
  game_resolution?: [number, number]
  hooks: Hooks
}

type InstallStage = 'installed' | 'installing' | 'pack_installing' | 'not_installed'

type LinkedData = {
  project_id: ModrinthId
  version_id: ModrinthId

  locked: boolean
}

type InstanceLoader = 'vanilla' | 'forge' | 'fabric' | 'quilt' | 'neoforge'

type MemorySettings = {
  maximum: number
}

type WindowSize = {
  width: number
  height: number
}

type Hooks = {
  pre_launch?: string
  wrapper?: string
  post_exit?: string
}

type Manifest = {
  gameVersions: ManifestGameVersion[]
}

type ManifestGameVersion = {
  id: string
  stable: boolean
  loaders: ManifestLoaderVersion[]
}

type ManifestLoaderVersion = {
  id: string
  url: string
  stable: boolean
}

type AppSettings = {
  max_concurrent_downloads: number
  max_concurrent_writes: number

  theme: 'dark' | 'light' | 'oled'
  default_page: 'Home' | 'Library'
  collapsed_navigation: boolean
  advanced_rendering: boolean
  native_decorations: boolean

  telemetry: boolean
  discord_rpc: boolean
  developer_mode: boolean
  personalized_ads: boolean

  onboarded: boolean

  extra_launch_args: string[]
  custom_env_vars: [string, string][]
  memory: MemorySettings
  force_fullscreen: boolean
  game_resolution: [number, number]
  hide_on_process_start: boolean
  hooks: Hooks

  custom_dir?: string
  prev_custom_dir?: string
  migrated: boolean
}

export type InstanceSettingsTabProps = {
  instance: GameInstance
  offline?: boolean
}
