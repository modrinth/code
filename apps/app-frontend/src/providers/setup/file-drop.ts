import { provideFileDrop } from '@modrinth/ui'
import { invoke } from '@tauri-apps/api/core'

import { listenNativeDragDrop } from '@/helpers/native-drag-drop'
import { getLocalFileInfo } from '@/platform/app-lib/files/commands'

function getFileName(path: string) {
	return path.split(/[\\/]/).pop() || 'file'
}

async function readDraggedFile(path: string) {
	const data = await invoke<number[]>('plugin:files|file_read_dragged_file', { path })
	return new Uint8Array(data)
}

export function setupFileDropProvider() {
	let nativeFileDropPaths: string[] = []

	provideFileDrop({
		async listenNativeFileDrop(handler) {
			return await listenNativeDragDrop((event) => {
				if (event.type === 'leave') {
					nativeFileDropPaths = []
					void handler({
						type: 'leave',
						paths: [],
						position: { x: 0, y: 0 },
					})
					return
				}

				if (event.type === 'enter' || event.type === 'drop') {
					nativeFileDropPaths = event.paths
				}

				void handler({
					type: event.type,
					paths: nativeFileDropPaths,
					position: event.position,
				})

				if (event.type === 'drop') {
					nativeFileDropPaths = []
				}
			})
		},
		async createFilesFromNativePaths(paths) {
			return await Promise.all(
				paths.map(async (path) => new File([await readDraggedFile(path)], getFileName(path))),
			)
		},
		async resolveLocalFiles(paths) {
			return await Promise.all(
				paths.map(async (path) => ({
					kind: 'local' as const,
					path,
					...(await getLocalFileInfo(path)),
				})),
			)
		},
	})
}
