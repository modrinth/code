import type { IconConfig } from './types'

export async function renderIcon(config: IconConfig, symbolAsset: string): Promise<File> {
	const size = 256
	const canvas = document.createElement('canvas')
	canvas.width = size
	canvas.height = size
	const context = canvas.getContext('2d')
	if (!context) throw new Error('Could not create an icon image.')

	if (config.background.type === 'color') {
		context.fillStyle = config.background.value
	} else {
		const gradient = context.createLinearGradient(0, 0, 0, size - 1)
		gradient.addColorStop(0, config.background.top_color)
		gradient.addColorStop(1, config.background.bottom_color)
		context.fillStyle = gradient
	}
	context.fillRect(0, 0, size, size)

	const symbol = new Image()
	symbol.src = symbolAsset
	await symbol.decode()
	context.drawImage(symbol, 0, 0, size, size)

	const blob = await new Promise<Blob>((resolve, reject) => {
		canvas.toBlob((result) => {
			if (result) resolve(result)
			else reject(new Error('Could not encode the icon image.'))
		}, 'image/png')
	})
	return new File([blob], 'server-icon-original.png', { type: 'image/png' })
}
