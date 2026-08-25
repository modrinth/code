import {
	type ImageViewerEditorData,
	type ImageViewerEditorSource,
	provideImageViewerEditor,
} from '@modrinth/ui'
import { readFile } from '@tauri-apps/plugin-fs'

import { release_ads_window_hold, take_ads_window_hold } from '@/helpers/ads.js'
import { get_screenshot_editor_data } from '@/helpers/instance'

export function setupImageViewerEditorProvider() {
	provideImageViewerEditor({
		async loadEditorData(source: ImageViewerEditorSource): Promise<ImageViewerEditorData> {
			const [instanceId, fileName] = JSON.parse(source.id) as [string, string]
			const editorData = await get_screenshot_editor_data({
				instance_id: instanceId,
				file_name: fileName,
			})
			const [sourceBytes, backgroundBytes] = await Promise.all([
				readFile(source.path),
				editorData.editor_state ? readFile(editorData.background_path) : undefined,
			])

			return {
				source: new Blob([sourceBytes], { type: 'image/png' }),
				background: backgroundBytes
					? new Blob([backgroundBytes], { type: 'image/png' })
					: undefined,
				editorState: editorData.editor_state ?? null,
				isEdited: source.isEdited,
			}
		},
		onShow: take_ads_window_hold,
		onHide: release_ads_window_hold,
	})
}
