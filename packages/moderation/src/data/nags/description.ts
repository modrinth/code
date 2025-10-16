import { renderHighlightedString } from '@modrinth/utils'
import { defineMessage, useVIntl } from '@vintl/vintl'

import type { Nag, NagContext } from '../../types/nags'

export const MIN_DESCRIPTION_CHARS = 200
export const MAX_HEADER_LENGTH = 80
export const MIN_SUMMARY_CHARS = 30
export const MIN_CHARS_PER_IMAGE = 60

export function analyzeHeaderLength(markdown: string): {
	hasLongHeaders: boolean
	longHeaders: string[]
} {
	if (!markdown) return { hasLongHeaders: false, longHeaders: [] }

	const withoutCodeBlocks = markdown.replace(/```[\s\S]*?```/g, '').replace(/`[^`]*`/g, '')

	const headerRegex = /^(#{1,3})\s+(.+)$/gm
	const headers = [...withoutCodeBlocks.matchAll(headerRegex)]

	const longHeaders: string[] = []

	headers.forEach((match) => {
		const headerText = match[2].trim()
		const sentenceEnders = /[.!?]+/g
		const sentences = headerText.split(sentenceEnders).filter((s) => s.trim().length > 0)

		const isVeryLong = headerText.length > MAX_HEADER_LENGTH
		const hasMultipleSentences = sentences.length > 1

		if (isVeryLong || hasMultipleSentences) {
			longHeaders.push(headerText)
		}
	})

	return {
		hasLongHeaders: longHeaders.length > 0,
		longHeaders,
	}
}

export function analyzeImageContent(markdown: string): {
	imageHeavy: boolean
	hasEmptyAltText: boolean
} {
	if (!markdown) return { imageHeavy: false, hasEmptyAltText: false }

	const withoutCodeBlocks = markdown.replace(/```[\s\S]*?```/g, '').replace(/`[^`]*`/g, '')

	const imageRegex = /!\[([^\]]*)\]\([^)]+\)/g
	const images = [...withoutCodeBlocks.matchAll(imageRegex)]

	const htmlImageRegex = /<img[^>]*>/gi
	const htmlImages = [...withoutCodeBlocks.matchAll(htmlImageRegex)]

	const totalImages = images.length + htmlImages.length
	if (totalImages === 0) return { imageHeavy: false, hasEmptyAltText: false }

	const textLength = countText(withoutCodeBlocks)
	const recommendedTextLength = MIN_CHARS_PER_IMAGE * totalImages
	const imageHeavy =
		recommendedTextLength > MIN_DESCRIPTION_CHARS && textLength < recommendedTextLength

	const hasEmptyAltText =
		images.some((match) => !match[1]?.trim()) ||
		htmlImages.some((match) => {
			const altMatch = match[0].match(/alt\s*=\s*["']([^"']*)["']/i)
			return !altMatch || !altMatch[1]?.trim()
		})

	return { imageHeavy, hasEmptyAltText }
}

export function countText(markdown: string): number {
	if (!markdown) return 0

	const fallback = (md: string): number => {
		const withoutCode = md.replace(/```[\s\S]*?```/g, '').replace(/`[^`]*`/g, '')
		const withoutImagesAndLinks = withoutCode
			.replace(/!\[[^\]]*]\([^)]+\)/g, ' ')
			.replace(/\[[^\]]*]\([^)]+\)/g, ' ')
		const withoutHtml = withoutImagesAndLinks.replace(/<[^>]+>/g, ' ')
		const withoutMdSyntax = withoutHtml
			.replace(/^>{1}\s?.*$/gm, ' ')
			.replace(/^#{1,6}\s+/gm, ' ')
			.replace(/[*_~`>-]/g, ' ')
			.replace(/\|/g, ' ')
		return withoutMdSyntax.replace(/\s+/g, ' ').trim().length
	}

	if (typeof window === 'undefined' || typeof globalThis.DOMParser === 'undefined') {
		console.warn(`[Moderation] SSR: no window/DOMParser, falling back for countText`)
		return fallback(markdown)
	}

	try {
		const htmlString = renderHighlightedString(markdown)
		const parser = new DOMParser()
		const doc = parser.parseFromString(htmlString, 'text/html')
		const walker = doc.createTreeWalker(doc.body || doc, NodeFilter.SHOW_TEXT)

		const textList: string[] = []
		let node = walker.nextNode()
		while (node) {
			if (node.textContent) textList.push(node.textContent)
			node = walker.nextNode()
		}
		return textList.join(' ').replace(/\s+/g, ' ').trim().length
	} catch {
		return fallback(markdown)
	}
}

export const descriptionNags: Nag[] = [
	{
		id: 'description-too-short',
		title: defineMessage({
			id: 'nags.description-too-short.title',
			defaultMessage: 'Expand the description',
		}),
		description: (context: NagContext) => {
			const { formatMessage } = useVIntl()
			const readableLength = countText(context.project.body || '')

			return formatMessage(
				defineMessage({
					id: 'nags.description-too-short.description',
					defaultMessage:
						'Your description is {length} readable characters. At least {minChars} characters is recommended to create a clear and informative description.',
				}),
				{
					length: readableLength,
					minChars: MIN_DESCRIPTION_CHARS,
				},
			)
		},
		status: 'warning',
		shouldShow: (context: NagContext) => {
			const readableLength = countText(context.project.body || '')
			return readableLength < MIN_DESCRIPTION_CHARS && readableLength > 0
		},
		link: {
			path: 'settings/description',
			title: defineMessage({
				id: 'nags.edit-description.title',
				defaultMessage: 'Edit description',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-description',
		},
	},
	{
		id: 'long-headers',
		title: defineMessage({
			id: 'nags.long-headers.title',
			defaultMessage: 'Shorten headers',
		}),
		description: (context: NagContext) => {
			const { formatMessage } = useVIntl()
			const { longHeaders } = analyzeHeaderLength(context.project.body || '')
			const count = longHeaders.length

			return formatMessage(
				defineMessage({
					id: 'nags.long-headers.description',
					defaultMessage:
						'{count, plural, one {# header} other {# headers}} in your description {count, plural, one {is} other {are}} too long. Headers should be concise and act as section titles, not full sentences.',
				}),
				{
					count,
				},
			)
		},
		status: 'warning',
		shouldShow: (context: NagContext) => {
			const { hasLongHeaders } = analyzeHeaderLength(context.project.body || '')
			return hasLongHeaders
		},
		link: {
			path: 'settings/description',
			title: defineMessage({
				id: 'nags.edit-description.title',
				defaultMessage: 'Edit description',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-description',
		},
	},
	{
		id: 'summary-too-short',
		title: defineMessage({
			id: 'nags.summary-too-short.title',
			defaultMessage: 'Expand the summary',
		}),
		description: (context: NagContext) => {
			const { formatMessage } = useVIntl()

			return formatMessage(
				defineMessage({
					id: 'nags.summary-too-short.description',
					defaultMessage:
						'Your summary is {length} characters. At least {minChars} characters is recommended to create an informative and enticing summary.',
				}),
				{
					length: context.project.description?.length || 0,
					minChars: MIN_SUMMARY_CHARS,
				},
			)
		},
		status: 'warning',
		shouldShow: (context: NagContext) => {
			const summaryLength = context.project.description?.trim()?.length || 0
			return summaryLength < MIN_SUMMARY_CHARS && summaryLength !== 0
		},
		link: {
			path: 'settings',
			title: defineMessage({
				id: 'nags.edit-summary.title',
				defaultMessage: 'Edit summary',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings',
		},
	},
	{
		id: 'summary-special-formatting',
		title: defineMessage({
			id: 'nags.summary-special-formatting.title',
			defaultMessage: 'Clear up the summary',
		}),
		description: defineMessage({
			id: 'nags.summary-special-formatting.description',
			defaultMessage: `Your summary should not contain formatting, line breaks, special characters, or links, since the summary will only display plain text.`,
		}),
		status: 'warning',
		shouldShow: (context: NagContext) => {
			const summary = context.project.description?.trim() || ''
			return Boolean(
				summary.match(/https:\/\//g) ||
					summary.match(/http:\/\//g) ||
					summary.match(/# .*/g) ||
					summary.match(/---/g) ||
					summary.match(/\n/g) ||
					summary.match(/\[.*\]\(.*\)/g) ||
					summary.match(/!\[.*\]/g) ||
					summary.match(/`.*`/g) ||
					summary.match(/\*.*\*/g) ||
					summary.match(/_.*_/g) ||
					summary.match(/~~.*~~/g) ||
					summary.match(/```/g) ||
					summary.match(/> /g),
			)
		},
		link: {
			path: 'settings',
			title: defineMessage({
				id: 'nags.edit-summary.title',
				defaultMessage: 'Edit summary',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings',
		},
	},
	{
		id: 'minecraft-title-clause',
		title: defineMessage({
			id: 'nags.minecraft-title-clause.title',
			defaultMessage: 'Avoid brand infringement',
		}),
		description: defineMessage({
			id: 'nags.minecraft-title-clause.description',
			defaultMessage: `Projects must not use Minecraft's branding or include "Minecraft" as a significant part of the name.`,
		}),
		status: 'warning',
		shouldShow: (context: NagContext) => {
			const title = context.project.title?.toLowerCase() || ''
			const wordsInTitle = title.split(' ').filter((word) => word.length > 0)
			return title.includes('minecraft') && title.length > 0 && wordsInTitle.length <= 3
		},
		link: {
			path: 'settings',
			title: defineMessage({
				id: 'nags.edit-title.title',
				defaultMessage: 'Edit title',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings',
		},
	},
	{
		id: 'title-contains-technical-info',
		title: defineMessage({
			id: 'nags.title-contains-technical-info.title',
			defaultMessage: 'Clean up the name',
		}),
		description: defineMessage({
			id: 'nags.title-contains-technical-info.description',
			defaultMessage:
				"Keeping your project's Name clean and makes it memorable easier to find. Version and loader information is automatically displayed alongside your project.",
		}),
		status: 'warning',
		shouldShow: (context: NagContext) => {
			const title = context.project.title?.toLowerCase() || ''
			if (!title) return false

			const loaderNames =
				context.tags.loaders?.map((loader: { name: string }) => loader.name?.toLowerCase()) || []
			const hasLoader = loaderNames.some((loader) => loader && title.includes(loader.toLowerCase()))
			const versionPatterns = [/\b1\.\d+(\.\d+)?\b/]
			const hasVersionPattern = versionPatterns.some((pattern) => pattern.test(title))

			return hasLoader || hasVersionPattern
		},
		link: {
			path: 'settings',
			title: defineMessage({
				id: 'nags.edit-title.title',
				defaultMessage: 'Edit title',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings',
		},
	},
	{
		id: 'summary-same-as-title',
		title: defineMessage({
			id: 'nags.summary-same-as-title.title',
			defaultMessage: 'Make the summary unique',
		}),
		description: defineMessage({
			id: 'nags.summary-same-as-title.description',
			defaultMessage:
				"Your summary can not be the same as your project's Name. It's important to create an informative and enticing Summary.",
		}),
		status: 'required',
		shouldShow: (context: NagContext) => {
			const title = context.project.title?.trim() || ''
			const summary = context.project.description?.trim() || ''
			return title === summary && title.length > 0 && summary.length > 0
		},
		link: {
			path: 'settings',
			title: defineMessage({
				id: 'nags.edit-summary.title',
				defaultMessage: 'Edit summary',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings',
		},
	},
	{
		// Don't like this one, is this needed?
		id: 'image-heavy-description',
		title: defineMessage({
			id: 'nags.image-heavy-description.title',
			defaultMessage: 'Ensure accessibility',
		}),
		description: defineMessage({
			id: 'nags.image-heavy-description.description',
			defaultMessage:
				'Your Description should contain sufficient plain text or image alt-text, keeping it accessible to those using screen readers or with slow internet connections.',
		}),
		status: 'warning',
		shouldShow: (context: NagContext) => {
			const { imageHeavy } = analyzeImageContent(context.project.body || '')
			return imageHeavy
		},
		link: {
			path: 'settings/description',
			title: defineMessage({
				id: 'nags.edit-description.title',
				defaultMessage: 'Edit description',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-description',
		},
	},
	{
		id: 'missing-alt-text',
		title: defineMessage({
			id: 'nags.missing-alt-text.title',
			defaultMessage: 'Add image alt text',
		}),
		description: defineMessage({
			id: 'nags.missing-alt-text.description',
			defaultMessage:
				'Some of your images are missing alt text, which is important for accessibility, especially for visually impaired users.',
		}),
		status: 'warning',
		shouldShow: (context: NagContext) => {
			const { hasEmptyAltText } = analyzeImageContent(context.project.body || '')
			return hasEmptyAltText
		},
		link: {
			path: 'settings/description',
			title: defineMessage({
				id: 'nags.edit-description.title',
				defaultMessage: 'Edit description',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-description',
		},
	},
]
