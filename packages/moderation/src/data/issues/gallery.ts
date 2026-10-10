import type { Labrinth } from '@modrinth/api-client'
import { ImageIcon } from '@modrinth/assets'
import { md } from '@modrinth/utils'

import insufficientMessage from '../messages/checklist/messages/gallery/insufficient.md'
import notRelevantMessage from '../messages/checklist/messages/gallery/not-relevant.md'
import showcaseClarityMessage from '../messages/checklist/messages/gallery/showcase-clarity.md'
import { issue, panel, toggle } from './component-builders/builders'
import { issueLocation } from './component-builders/locations'
import { IssuePriority } from './component-builders/priority'
import { rulesAiImagesIssue } from './rules'

const escapeHtml = md().utils.escapeHtml

export const galleryInsufficientIssue = issue({
	id: 'gallery-insufficient',
	title: 'Insufficient gallery images',
	category: 'Gallery',
	actions: [
		({ projectV3 }) => ({
			type: 'add_gallery_images',
			value: { original_count: projectV3.gallery.length },
		}),
	],
	message: insufficientMessage,
	suggestedStatus: 'flagged',
	priority: IssuePriority.Rules + 1,
})

export const galleryNotRelevantIssue = issue({
	id: 'gallery-not-relevant',
	title: 'Irrelevant gallery images',
	category: 'Gallery',
	actions: ({ projectV3, selected }) => {
		const keys = new Set((selected.items['gallery-image'] ?? []).map(({ key }) => key))
		const imageIds = projectV3.gallery.flatMap((image) =>
			image.id !== undefined && keys.has(String(image.id)) ? [image.id] : [],
		)
		return imageIds.length
			? [() => ({ type: 'remove_gallery_images', value: { image_ids: imageIds } })]
			: []
	},
	message: ({ projectV3, selected }) => {
		const keys = new Set((selected.items['gallery-image'] ?? []).map(({ key }) => key))
		const entries = projectV3.gallery
			.filter((image) => image.id !== undefined && keys.has(String(image.id)))
			.map(
				(image) =>
					`<image-viewer src="${escapeHtml(image.raw_url || image.url)}" alt="${escapeHtml(image.name ?? '')}" />`,
			)
		const list = entries.length ? `Please remove the following images:\n${entries.join('\n')}` : ''
		return notRelevantMessage.replace('%GALLERY_IMAGES%', list).trimEnd()
	},
	suggestedStatus: 'flagged',
})

export const galleryShowcaseClarityIssue = issue({
	id: 'gallery-showcase-clarity',
	locations: [issueLocation('gallery')],
	title: 'Unclear gallery showcase',
	category: 'Gallery',
	message: showcaseClarityMessage,
	suggestedStatus: 'rejected',
	priority: IssuePriority.Rules,
})

export function galleryImageReviewPanel(
	image: Labrinth.Projects.v3.GalleryItem,
	imageNumber: number,
) {
	if (!image.id) throw new Error('Gallery image review requires an image ID')

	return panel({
		parent: 'gallery',
		title: image.name ?? `Gallery image ${imageNumber}`,
		hint: "Are this project's gallery images sufficient?",
		icon: ImageIcon,
		guidanceUrl:
			'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf08096828bd1c3f24d8b8e',
	}).content(
		toggle({
			issue: galleryNotRelevantIssue,
			label: 'Not relevant',
		}),

		toggle({
			issue: rulesAiImagesIssue,
			label: 'AI Images',
		}),
	)
}

export const galleryReviewPanel = panel({
	title: 'Gallery',
	hint: "Are this project's gallery images sufficient?",
	icon: ImageIcon,
	guidanceUrl:
		'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf08096828bd1c3f24d8b8e',
}).content(
	toggle({ issue: galleryInsufficientIssue, label: 'Insufficient' }),
	toggle({
		issue: galleryShowcaseClarityIssue,
		label: 'Showcase Clarity',
	}),
)
