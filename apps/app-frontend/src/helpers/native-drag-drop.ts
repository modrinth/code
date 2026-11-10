import type { UnlistenFn } from '@tauri-apps/api/event'
import type { DragDropEvent } from '@tauri-apps/api/webview'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import { platform } from '@tauri-apps/plugin-os'

export type NativeDragDropEvent =
	| { type: 'enter'; paths: string[]; position: { x: number; y: number } }
	| { type: 'over'; position: { x: number; y: number } }
	| { type: 'drop'; paths: string[]; position: { x: number; y: number } }
	| { type: 'leave' }

/**
 * Listens for native OS file drag & drop on the main window, with positions in CSS pixels.
 *
 * This must listen as a `WebviewWindow`. Depending on how the main webview is attached, Tauri emits
 * drag events either to `Webview` targets or to `Window` targets, and only a `WebviewWindow`
 * listener matches both.
 *
 * Tauri types the position as physical, but only Windows reports physical pixels. macOS and Linux
 * already report logical pixels, so only Windows is scaled down.
 */
export async function listenNativeDragDrop(
	handler: (event: NativeDragDropEvent) => void | Promise<void>,
): Promise<UnlistenFn> {
	const scale = platform() === 'windows' ? window.devicePixelRatio || 1 : 1

	return await getCurrentWebviewWindow().onDragDropEvent((event: { payload: DragDropEvent }) => {
		const payload = event.payload

		if (payload.type === 'leave') {
			void handler({ type: 'leave' })
			return
		}

		const position = { x: payload.position.x / scale, y: payload.position.y / scale }

		if (payload.type === 'over') {
			void handler({ type: 'over', position })
			return
		}

		void handler({ type: payload.type, paths: payload.paths, position })
	})
}
