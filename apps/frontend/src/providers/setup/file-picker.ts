import { provideFilePicker } from '@modrinth/ui'

function pickFile(accept: string): Promise<{ file: File; previewUrl: string } | null> {
	return new Promise((resolve) => {
		const input = document.createElement('input')
		input.type = 'file'
		input.accept = accept
		input.onchange = () => {
			const file = input.files?.[0]
			if (!file) return resolve(null)
			resolve({ file, previewUrl: URL.createObjectURL(file) })
		}
		input.oncancel = () => resolve(null)
		input.click()
	})
}

export function setupFilePickerProvider() {
	provideFilePicker({
		pickImage: () =>
			pickFile('image/png,image/jpeg,image/jpg,image/svg+xml,image/webp,image/gif'),
		pickModpackFile: () => pickFile('.mrpack'),
	})
}
