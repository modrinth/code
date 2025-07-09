import type { Project } from '../../types'
import type { ServerNotice } from './common'

export interface ServerGeneral {
  server_id: string
  name: string
  net: {
    ip: string
    port: number
    domain: string
  }
  game: string
  backup_quota: number
  used_backup_quota: number
  status: string
  suspension_reason: 'moderated' | 'paymentfailed' | 'cancelled' | 'upgrading' | 'other' | string
  loader: string
  loader_version: string
  mc_version: string
  upstream: {
    kind: 'modpack' | 'mod' | 'resourcepack'
    version_id: string
    project_id: string
  } | null
  motd?: string
  image?: string
  project?: Project
  sftp_username: string
  sftp_password: string
  sftp_host: string
  datacenter?: string
  notices?: ServerNotice[]
  node: {
    token: string
    instance: string
  }
  flows?: {
    intro?: boolean
  }
}

export interface Allocation {
  port: number
  name: string
}

export interface Startup {
  invocation: string
  original_invocation: string
  jdk_version: 'lts8' | 'lts11' | 'lts17' | 'lts21'
  jdk_build: 'corretto' | 'temurin' | 'graal'
}

export type PowerAction = 'Start' | 'Stop' | 'Restart' | 'Kill'
export type JDKVersion = 'lts8' | 'lts11' | 'lts17' | 'lts21'
export type JDKBuild = 'corretto' | 'temurin' | 'graal'

export type Loaders =
  | 'Fabric'
  | 'Quilt'
  | 'Forge'
  | 'NeoForge'
  | 'Paper'
  | 'Spigot'
  | 'Bukkit'
  | 'Vanilla'
  | 'Purpur'

export type ServerState =
  | 'starting'
  | 'running'
  | 'restarting'
  | 'stopping'
  | 'stopped'
  | 'crashed'
  | 'unknown'
