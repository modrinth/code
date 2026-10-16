import type { Labrinth } from '@modrinth/api-client'

import type { ReviewContext } from './types'

export function galleryImageReviewKey(
	image: Labrinth.Projects.v3.GalleryItem,
	sendIssuesAsThreadMessage = false,
) {
	return sendIssuesAsThreadMessage
		? image.url
		: image.id !== undefined
			? String(image.id)
			: undefined
}

export function selectedGalleryImages(context: ReviewContext) {
	const keys = new Set((context.selected.items['gallery-image'] ?? []).map(({ key }) => key))
	return context.projectV3.gallery.filter((image) => {
		const key = galleryImageReviewKey(image, context.sendIssuesAsThreadMessage)
		return key !== undefined && keys.has(key)
	})
}
