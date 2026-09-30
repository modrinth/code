import { createContext } from './create-context'

export type FileDownload = { filename: string; serverId?: string; serverName?: string } & (
	| { type: 'server-file'; serverId: string; path: string }
	| { type: 'server-backup'; serverId: string; backupId: string }
	| { type: 'server-world'; nodeUrlHost: string; worldId: string }
	| { type: 'mrpack'; instanceId: string; version: number }
)

export interface FileDownloadProvider {
	download: (file: FileDownload) => Promise<boolean>
}

export const [injectFileDownload, provideFileDownload] = createContext<FileDownloadProvider>(
	'FileDownloadProvider',
	'fileDownload',
)
