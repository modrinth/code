import { provideFilePicker } from '@modrinth/ui'
import { convertFileSrc } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

export function setupFilePickerProvider() {
	provideFilePicker({
		async pickImage() {
			const result = await open({
				multiple: false,
				filters: [
					{ name: 'Image', extensions: ['png', 'jpeg', 'jpg', 'svg', 'webp', 'gif'] },
				],
			})
			if (!result) return null
			const path = result.path ?? result
			if (!path) return null
			const name = path.split(/[\\/]/).pop() || 'icon'
			const file = new File([], name)
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
			const name = path.split(/[\\/]/).pop() || 'modpack.mrpack'
			const file = new File([], name)
			return { file, path, previewUrl: '' }
		},
	})
}
