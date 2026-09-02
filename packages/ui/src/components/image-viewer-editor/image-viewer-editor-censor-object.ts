import type { ScreenshotEditorSourceRect } from './image-viewer-editor-types'

export function renderCensorRegion(sourceRect: ScreenshotEditorSourceRect, solidColor: string) {
	const width = Math.max(1, Math.round(sourceRect.width))
	const height = Math.max(1, Math.round(sourceRect.height))
	const output = document.createElement('canvas')
	output.width = width
	output.height = height
	const outputContext = output.getContext('2d')
	if (!outputContext) throw new Error('Could not create censor canvas')

	outputContext.fillStyle = solidColor
	outputContext.fillRect(0, 0, width, height)
	return output
}
