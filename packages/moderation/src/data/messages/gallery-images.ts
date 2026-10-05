import type { ReviewContext } from '../issues/component-builders/types'

function escapeHtml(value: string): string {
	return value.replace(/[&<>"']/g, (character) => {
		switch (character) {
			case '&':
				return '&amp;'
			case '<':
				return '&lt;'
			case '>':
				return '&gt;'
			case '"':
				return '&quot;'
			default:
				return '&#39;'
		}
	})
}

export function removalImageEntry(title: string, imageUrl: string): string {
	const label = escapeHtml(title)
	const url = escapeHtml(imageUrl)
	return `<li class="review-card-image-entry"><a href="${url}" target="_blank" rel="noopener"><img src="${url}" alt="${label}" class="review-card-gallery-image"></a>${label ? `<span class="review-card-image-caption">${label}</span>` : ''}</li>`
}

export function galleryImagesMessage(
	message: string,
	{ projectV3, selected }: ReviewContext,
): string {
	const keys = new Set((selected.items['gallery-image'] ?? []).map(({ key }) => key))
	const entries = [
		...projectV3.gallery
			.filter((image) => image.id !== undefined && keys.has(String(image.id)))
			.map((image) => removalImageEntry(image.name ?? '', image.raw_url || image.url)),
	]
	const list = entries.length
		? `Please remove the following images:\n\n<ul class="review-card-image-list">\n${entries.join('\n')}\n</ul>`
		: ''
	return message.replace('%GALLERY_IMAGES%', list).trimEnd()
}
