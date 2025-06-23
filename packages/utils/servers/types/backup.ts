import type { WSBackupTask, WSBackupState } from './websocket'
import type { MessageDescriptor } from '@vintl/vintl'

export interface Backup {
  id: string
  name: string
  created_at: string
  locked: boolean
  automated: boolean
  interrupted: boolean
  ongoing: boolean
  task: {
    [K in WSBackupTask]?: {
      progress: number
      state: WSBackupState
    }
  }
}

export type BackupInProgressReason = {
  type: string
  tooltip: MessageDescriptor
}

export interface AutoBackupSettings {
  enabled: boolean
  interval: number
}

export interface ServerBackup {
  id: string
  name: string
  created_at: string
}
