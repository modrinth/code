import {
	type ImageViewerEditorData,
	type ImageViewerEditorSource,
	provideImageViewerEditor,
} from '@modrinth/ui'
import { readFile } from '@tauri-apps/plugin-fs'

import { release_ads_window_hold, take_ads_window_hold } from '@/helpers/ads.js'

export function setupImageViewerEditorProvider() {
	provideImageViewerEditor({
		async loadEditorData(source: ImageViewerEditorSource): Promise<ImageViewerEditorData> {
			return {
				source: new Blob([await readFile(source.path)], { type: 'image/png' }),
			}
		},
		onShow: take_ads_window_hold,
		onHide: release_ads_window_hold,
	})
}
