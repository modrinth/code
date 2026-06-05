<script setup lang="ts">
import {
	injectModrinthClient,
	injectModrinthServerContext,
	type NativeFileDropAdapter,
	ServersManageFilesPage,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import type { DragDropEvent } from '@tauri-apps/api/webview'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { readFile } from '@tauri-apps/plugin-fs'

const client = injectModrinthClient()
const { serverId } = injectModrinthServerContext()
const queryClient = useQueryClient()

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

let nativeFileDropPaths: string[] = []

const nativeFileDrop: NativeFileDropAdapter = {
	async listen(handler) {
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
	async createFiles(paths) {
		return await Promise.all(
			paths.map(async (path) => new File([await readFile(path)], getFileName(path))),
		)
	},
}

try {
	await queryClient.ensureQueryData({
		queryKey: ['files', serverId, '/'],
		queryFn: () => client.kyros.files_v0.listDirectory('/', 1, 2000),
		staleTime: 30_000,
	})
} catch {
	// Let mounted layouts' useQuery surface errors; do not fail route setup.
}
</script>

<template>
	<ServersManageFilesPage :native-file-drop="nativeFileDrop" />
</template>
