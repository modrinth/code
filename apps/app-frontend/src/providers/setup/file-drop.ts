import { provideFileDrop } from '@modrinth/ui'
import { invoke } from '@tauri-apps/api/core'
import type { DragDropEvent } from '@tauri-apps/api/webview'
import { getCurrentWebview } from '@tauri-apps/api/webview'

function getFileName(path: string) {
	return path.split(/[\\/]/).pop() || 'file'
}

function toLogicalPosition(position: { x: number; y: number }) {
	const scale = window.devicePixelRatio || 1
	return {
		x: position.x / scale,
		y: position.y / scale,
	}
}

async function readDraggedFile(path: string) {
	const data = await invoke<number[]>('plugin:files|file_read_dragged_file', { path })
	return new Uint8Array(data)
}

export function setupFileDropProvider() {
	let nativeFileDropPaths: string[] = []

	provideFileDrop({
		async listenNativeFileDrop(handler) {
			return await getCurrentWebview().onDragDropEvent((event: { payload: DragDropEvent }) => {
				const payload = event.payload

				if (payload.type === 'leave') {
					nativeFileDropPaths = []
					void handler({
						type: 'leave',
						paths: [],
						position: { x: 0, y: 0 },
					})
					return
				}

				if (payload.type === 'enter' || payload.type === 'drop') {
					nativeFileDropPaths = payload.paths
				}

				void handler({
					type: payload.type,
					paths: nativeFileDropPaths,
					position: toLogicalPosition(payload.position),
				})

				if (payload.type === 'drop') {
					nativeFileDropPaths = []
				}
			})
		},
		async createFilesFromNativePaths(paths) {
			return await Promise.all(
				paths.map(async (path) => new File([await readDraggedFile(path)], getFileName(path))),
			)
		},
	})
}
