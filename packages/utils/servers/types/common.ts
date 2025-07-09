import type { Project } from '../../types'
import { Allocation } from './server'
import { ServerBackup } from './backup'
import { Mod } from './content'

export type ServerNotice = {
  id: number
  message: string
  title?: string
  level: 'info' | 'warn' | 'critical' | 'survey'
  dismissable: boolean
  announce_at: string
  expires: string
  assigned: {
    kind: 'server' | 'node'
    id: string
    name: string
  }[]
  dismissed_by: {
    server: string
    dismissed_on: string
  }[]
}

export interface Server {
  server_id: string
  name: string
  status: string
  net: {
    ip: string
    port: number
    domain: string
    allocations: Allocation[]
  }
  game: string
  loader: string | null
  loader_version: string | null
  mc_version: string | null
  backup_quota: number
  used_backup_quota: number
  backups: ServerBackup[]
  mods: Mod[]
  project: Project | null
  suspension_reason: string | null
  image: string | null
  upstream?: {
    kind: 'modpack'
    project_id: string
    version_id: string
  }
  motd: string
  flows: {
    intro?: boolean
  }
}

export interface Servers {
  servers: Server[]
}
