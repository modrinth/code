import { open } from '@tauri-apps/plugin-dialog'

export function pickFile(): Promise<string | null> {
	return open({ multiple: false, directory: false })
}
