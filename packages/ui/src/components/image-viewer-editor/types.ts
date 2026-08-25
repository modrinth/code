export type ImageViewerEditorSource = {
	id: string
	path: string
	isEdited: boolean
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
	background?: Blob
	editorState: string | null
	isEdited: boolean
}

export type ImageViewerEditorSavePayload = {
	item: ImageViewerEditorItem
	pngBytes: Uint8Array
	editorState: string | null
	mode: 'create_copy' | 'replace_edit'
}
