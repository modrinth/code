import { provideFilePicker } from '@modrinth/ui'
import { convertFileSrc } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { readFile } from '@tauri-apps/plugin-fs'

function getFileName(path: string, fallback: string) {
	return path.split(/[\\/]/).pop() || fallback
}

async function createFileFromPath(path: string, fallbackName: string, type?: string) {
	const bytes = await readFile(path)
	const name = getFileName(path, fallbackName)
	return new File([bytes], name, type ? { type } : undefined)
}

export function setupFilePickerProvider() {
	provideFilePicker({
		async pickImage() {
			const result = await open({
				multiple: false,
				filters: [{ name: 'Image', extensions: ['png', 'jpeg', 'jpg', 'svg', 'webp', 'gif'] }],
			})
			if (!result) return null
			const path = result.path ?? result
			if (!path) return null
			const file = await createFileFromPath(path, 'icon')
			return { file, path, previewUrl: convertFileSrc(path) }
		},
		async pickModpackFile() {
			const result = await open({
				multiple: false,
				filters: [{ name: 'Modpack', extensions: ['mrpack'] }],
			})
			if (!result) return null
			const path = result.path ?? result
			if (!path) return null
			const file = await createFileFromPath(
				path,
				'modpack.mrpack',
				'application/x-modrinth-modpack+zip',
			)
			return { file, path, previewUrl: '' }
		},
	})
}
