export type ImageViewerEditorSource = {
	id: string
	path: string
}

export type ImageViewerEditorItem = {
	id: string
	src: string
	alt: string
	title?: string
	description?: string
	editorSource?: ImageViewerEditorSource
}

export type ImageViewerEditorData = {
	source: Blob
}

export type ImageViewerEditorSavePayload = {
	item: ImageViewerEditorItem
	pngBytes: Uint8Array
	mode: 'create_copy' | 'replace_edit'
}
