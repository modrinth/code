import { createContext } from './create-context'

export type FileDownload = { filename: string; serverId?: string; serverName?: string } & (
	| { type: 'server-file'; serverId: string; nodeUrlHost: string; worldId: string; path: string }
	| {
			type: 'server-backup'
			serverId: string
			nodeUrlHost: string
			worldId: string
			backupId: string
	  }
	| { type: 'server-world'; nodeUrlHost: string; worldId: string }
)

export interface FileDownloadProvider {
	download: (file: FileDownload) => Promise<boolean>
}

export const [injectFileDownload, provideFileDownload] = createContext<FileDownloadProvider>(
	'FileDownloadProvider',
	'fileDownload',
)
