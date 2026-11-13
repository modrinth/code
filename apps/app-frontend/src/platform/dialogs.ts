import { open } from '@tauri-apps/plugin-dialog'

export async function pickDirectory(title: string): Promise<string | null> {
	const directory = await open({ multiple: false, directory: true, title })
	return typeof directory === 'string' ? directory : null
}

export function pickFile(): Promise<string | null> {
	return open({ multiple: false, directory: false })
}
