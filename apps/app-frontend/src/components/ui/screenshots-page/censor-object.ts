import type { ScreenshotCensorMode, ScreenshotEditorSourceRect } from './editor-types'

const BLUR_DOWNSAMPLE = 0.25
const BLUR_SIGMA = 3
const BLUR_RADIUS = Math.ceil(BLUR_SIGMA * 3)
const BLUR_KERNEL = createGaussianKernel()

export function renderCensorRegion(
	source: CanvasImageSource,
	sourceRect: ScreenshotEditorSourceRect,
	mode: ScreenshotCensorMode,
	solidColor: string,
) {
	const width = Math.max(1, Math.round(sourceRect.width))
	const height = Math.max(1, Math.round(sourceRect.height))
	const output = document.createElement('canvas')
	output.width = width
	output.height = height
	const outputContext = output.getContext('2d')
	if (!outputContext) throw new Error('Could not create censor canvas')

	if (mode === 'solid') {
		outputContext.fillStyle = solidColor
		outputContext.fillRect(0, 0, width, height)
		return output
	}

	const sampleWidth = Math.max(1, Math.round(width * BLUR_DOWNSAMPLE))
	const sampleHeight = Math.max(1, Math.round(height * BLUR_DOWNSAMPLE))
	const sample = document.createElement('canvas')
	sample.width = sampleWidth
	sample.height = sampleHeight
	const sampleContext = sample.getContext('2d')
	if (!sampleContext) throw new Error('Could not create blur canvas')

	sampleContext.drawImage(
		source,
		sourceRect.left,
		sourceRect.top,
		sourceRect.width,
		sourceRect.height,
		0,
		0,
		sampleWidth,
		sampleHeight,
	)
	const blurredPixels = gaussianBlur(
		sampleContext.getImageData(0, 0, sampleWidth, sampleHeight),
		sampleWidth,
		sampleHeight,
	)
	sampleContext.putImageData(blurredPixels, 0, 0)
	outputContext.imageSmoothingEnabled = true
	outputContext.imageSmoothingQuality = 'high'
	outputContext.drawImage(sample, 0, 0, sampleWidth, sampleHeight, 0, 0, width, height)
	return output
}

function createGaussianKernel() {
	const kernel = new Float32Array(BLUR_RADIUS * 2 + 1)
	let total = 0
	for (let index = -BLUR_RADIUS; index <= BLUR_RADIUS; index++) {
		const weight = Math.exp(-(index * index) / (2 * BLUR_SIGMA * BLUR_SIGMA))
		kernel[index + BLUR_RADIUS] = weight
		total += weight
	}
	for (let index = 0; index < kernel.length; index++) kernel[index] /= total
	return kernel
}

function gaussianBlur(imageData: ImageData, width: number, height: number) {
	const horizontal = new Float32Array(imageData.data.length)
	const output = new ImageData(width, height)

	for (let y = 0; y < height; y++) {
		for (let x = 0; x < width; x++) {
			for (let offset = -BLUR_RADIUS; offset <= BLUR_RADIUS; offset++) {
				const sampleX = Math.max(0, Math.min(width - 1, x + offset))
				const sourceIndex = (y * width + sampleX) * 4
				const weight = BLUR_KERNEL[offset + BLUR_RADIUS]!
				for (let channel = 0; channel < 4; channel++) {
					horizontal[(y * width + x) * 4 + channel] +=
						imageData.data[sourceIndex + channel]! * weight
				}
			}
		}
	}

	for (let y = 0; y < height; y++) {
		for (let x = 0; x < width; x++) {
			for (let offset = -BLUR_RADIUS; offset <= BLUR_RADIUS; offset++) {
				const sampleY = Math.max(0, Math.min(height - 1, y + offset))
				const sourceIndex = (sampleY * width + x) * 4
				const weight = BLUR_KERNEL[offset + BLUR_RADIUS]!
				for (let channel = 0; channel < 4; channel++) {
					output.data[(y * width + x) * 4 + channel] += horizontal[sourceIndex + channel]! * weight
				}
			}
		}
	}

	return output
}
