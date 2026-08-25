import type {
	ImageViewerEditorData,
	ImageViewerEditorSource,
} from '#ui/components/image-viewer-editor/types'

import { createContext } from '.'

export interface ImageViewerEditorContext {
	loadEditorData: (source: ImageViewerEditorSource) => Promise<ImageViewerEditorData>
	onShow?: () => void
	onHide?: () => void
}

export const [injectImageViewerEditor, provideImageViewerEditor] =
	createContext<ImageViewerEditorContext>('ImageViewerEditor')
