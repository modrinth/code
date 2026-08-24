export type ScreenshotEditorTool =
	| 'select'
	| 'pen'
	| 'highlight'
	| 'eraser'
	| 'text'
	| 'arrow'
	| 'rectangle'
	| 'ellipse'
	| 'censor'

export type ScreenshotCensorMode = 'blur' | 'solid'

export type ScreenshotEraserMode = 'element' | 'area'

export type ScreenshotEditorObjectKind =
	| 'annotation'
	| 'arrow'
	| 'background'
	| 'censor'
	| 'ellipse'
	| 'highlight'
	| 'pen'
	| 'rectangle'
	| 'text'

export type ScreenshotEditorPropertyKind = Exclude<
	ScreenshotEditorObjectKind,
	'annotation' | 'background'
>

export type ScreenshotEditorSourceRect = {
	left: number
	top: number
	width: number
	height: number
}

export type ScreenshotEditorObjectState = Record<string, unknown> & {
	editorKind?: ScreenshotEditorObjectKind
	sourceRect?: ScreenshotEditorSourceRect
	censorMode?: ScreenshotCensorMode
	censorColor?: string
}

export type EditorHistoryEntry = {
	objects: ScreenshotEditorObjectState[]
}

export type ScreenshotEditorDocument = EditorHistoryEntry & {
	version: 1
}
