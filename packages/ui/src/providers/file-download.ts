import { createContext } from './create-context'

interface ServerDownloadBase {
	filename: string
	serverId: string
	serverName: string
	nodeUrlHost: string
	worldId: string
}

export interface ServerFileDownload extends ServerDownloadBase {
	type: 'server-file'
	path: string
}

export interface ServerBackupDownload extends ServerDownloadBase {
	type: 'server-backup'
	backupId: string
}

export type FileDownload = ServerFileDownload | ServerBackupDownload

export interface FileDownloadProvider {
	download: (file: FileDownload) => Promise<boolean>
}

export const [injectFileDownload, provideFileDownload] = createContext<FileDownloadProvider>(
	'FileDownloadProvider',
	'fileDownload',
)
