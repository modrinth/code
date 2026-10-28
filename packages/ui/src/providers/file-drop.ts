import type { LocalFile } from '@modrinth/api-client'

import { createContext } from '.'

export type NativeFileDropEvent = {
	type: 'enter' | 'over' | 'drop' | 'leave'
	paths: string[]
	position: {
		x: number
		y: number
	}
}

export interface FileDropProvider {
	listenNativeFileDrop: (
		handler: (event: NativeFileDropEvent) => void | Promise<void>,
	) => Promise<() => void>
	createFilesFromNativePaths: (paths: string[]) => Promise<File[]>
	/** Describes dropped paths as local files, for uploads that read them straight from disk. */
	resolveLocalFiles?: (paths: string[]) => Promise<LocalFile[]>
}

export const [injectFileDrop, provideFileDrop] = createContext<FileDropProvider>('FileDrop')
