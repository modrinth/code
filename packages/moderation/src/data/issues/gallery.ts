import { ImageIcon } from '@modrinth/assets'

import insufficientMessage from '../messages/checklist/messages/gallery/insufficient.md'
import notRelevantMessage from '../messages/checklist/messages/gallery/not-relevant.md'
import showcaseClarityMessage from '../messages/checklist/messages/gallery/showcase-clarity.md'
import { issue, panel, toggle } from './component-builders/builders'
import { rulesAiImagesIssue } from './rules'

export const galleryInsufficientIssue = issue({
	id: 'gallery-insufficient',
	title: 'Insufficient gallery images',
	category: 'Gallery',
	message: insufficientMessage,
	suggestedStatus: 'flagged',
})

export const galleryNotRelevantIssue = issue({
	id: 'gallery-not-relevant',
	title: 'Irrelevant gallery images',
	category: 'Gallery',
	message: notRelevantMessage,
	suggestedStatus: 'flagged',
})

export const galleryShowcaseClarityIssue = issue({
	id: 'gallery-showcase-clarity',
	title: 'Unclear gallery showcase',
	category: 'Gallery',
	message: showcaseClarityMessage,
	suggestedStatus: 'rejected',
})

export const galleryReviewPanel = panel({
	hint: "Are this project's gallery images sufficient?",
	icon: ImageIcon,
	guidanceUrl:
		'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf08096828bd1c3f24d8b8e',
}).content(
	toggle({
		issue: galleryInsufficientIssue,
		label: 'Insufficient',
	}),
	toggle({
		issue: galleryNotRelevantIssue,
		label: 'Not relevant',
		shown: ({ ProjectV3 }) => ProjectV3.gallery.length > 0,
	}),
	toggle({
		issue: galleryShowcaseClarityIssue,
		label: 'Showcase Clarity',
	}),
	toggle({
		issue: rulesAiImagesIssue,
		label: 'AI Images',
	}),
)
