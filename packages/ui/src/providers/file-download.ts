import { createContext } from './create-context'

export interface FileDownload {
	type: 'server-file'
	filename: string
	serverId: string
	serverName: string
	nodeUrlHost: string
	worldId: string
	path: string
}

export interface FileDownloadProvider {
	download: (file: FileDownload) => Promise<boolean>
}

export const [injectFileDownload, provideFileDownload] = createContext<FileDownloadProvider>(
	'FileDownloadProvider',
	'fileDownload',
)
