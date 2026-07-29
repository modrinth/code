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
}

export const [injectFileDrop, provideFileDrop] = createContext<FileDropProvider>('FileDrop')
