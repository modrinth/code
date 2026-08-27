import { defineMessage, useVIntl } from '@modrinth/ui'
import type { ElementNode, MarkdownDocument, Node } from '@modrinth/utils'
import { visit } from '@modrinth/utils'

import type { Nag, NagContext } from '../../types/nags'

export const MIN_DESCRIPTION_CHARS = 200
export const MAX_HEADER_LENGTH = 80
export const MIN_SUMMARY_CHARS = 30
export const MIN_CHARS_PER_IMAGE = 60

function collectText(node: Node, out: string[]): void {
	if (typeof node === 'string') {
		out.push(node)
		return
	}
	const tag = node[0]
	if (tag === null || tag === 'code' || tag === 'pre') return
	for (const child of node.slice(2) as Node[]) collectText(child, out)
}

function getElementText(node: ElementNode): string {
	const out: string[] = []
	for (const child of node.slice(2) as Node[]) collectText(child, out)
	return out.join('')
}

export function analyzeHeaderLength(document: MarkdownDocument | null): {
	hasLongHeaders: boolean
	longHeaders: string[]
} {
	if (!document) return { hasLongHeaders: false, longHeaders: [] }

	const longHeaders: string[] = []
	visit(
		document,
		(node) => Array.isArray(node) && /^h[1-3]$/.test(node[0] as string),
		(node) => {
			const headerText = getElementText(node as ElementNode).trim()
			const sentences = headerText.split(/[.!?]+/g).filter((s) => s.trim().length > 0)

			if (headerText.length > MAX_HEADER_LENGTH || sentences.length > 1) {
				longHeaders.push(headerText)
			}
		},
	)

	return {
		hasLongHeaders: longHeaders.length > 0,
		longHeaders,
	}
}

export function analyzeImageContent(document: MarkdownDocument | null): {
	imageHeavy: boolean
	hasEmptyAltText: boolean
} {
	if (!document) return { imageHeavy: false, hasEmptyAltText: false }

	let totalImages = 0
	let hasEmptyAltText = false

	visit(
		document,
		(node) => Array.isArray(node) && node[0] === 'img',
		(node) => {
			totalImages++
			const alt = (node as ElementNode)[1]?.alt
			if (typeof alt !== 'string' || !alt.trim()) hasEmptyAltText = true
		},
	)

	if (totalImages === 0) return { imageHeavy: false, hasEmptyAltText: false }

	const textLength = countText(document)
	const recommendedTextLength = MIN_CHARS_PER_IMAGE * totalImages
	const imageHeavy =
		recommendedTextLength > MIN_DESCRIPTION_CHARS && textLength < recommendedTextLength

	return { imageHeavy, hasEmptyAltText }
}

export function countText(document: MarkdownDocument | null): number {
	if (!document) return 0

	const out: string[] = []
	for (const node of document.nodes) collectText(node, out)
	return out.join('').replace(/\s+/g, ' ').trim().length
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
			const readableLength = countText(context.descriptionDocument)

			return formatMessage(
				defineMessage({
					id: 'nags.description-too-short.description',
					defaultMessage:
						'Your description is {length, plural, one {# readable character} other {# readable characters}}. At least {minChars, plural, one {# character} other {# characters}} is recommended to create a clear and informative description.',
				}),
				{
					length: readableLength,
					minChars: MIN_DESCRIPTION_CHARS,
				},
			)
		},
		status: 'warning',
		shouldShow: (context: NagContext) => {
			const readableLength = countText(context.descriptionDocument)
			return readableLength < MIN_DESCRIPTION_CHARS && readableLength > 0
		},
		link: {
			path: 'settings/description',
			title: defineMessage({
				id: 'nags.edit-description.title',
				defaultMessage: 'Edit description',
			}),
			shouldShow: (context: NagContext) =>
				context.currentRoute !== 'type-project-settings-description',
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
			const { longHeaders } = analyzeHeaderLength(context.descriptionDocument)
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
			const { hasLongHeaders } = analyzeHeaderLength(context.descriptionDocument)
			return hasLongHeaders
		},
		link: {
			path: 'settings/description',
			title: defineMessage({
				id: 'nags.edit-description.title',
				defaultMessage: 'Edit description',
			}),
			shouldShow: (context: NagContext) =>
				context.currentRoute !== 'type-project-settings-description',
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
						'Your summary is {length, plural, one {# character} other {# characters}}. At least {minChars, plural, one {# character} other {# characters}} is recommended to create an informative and enticing summary.',
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
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-project-settings',
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
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-project-settings',
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
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-project-settings',
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
				"Keeping your project's Name clean makes it memorable and easier to find. Version and loader information is automatically displayed alongside your project.",
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
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-project-settings',
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
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-project-settings',
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
			const { imageHeavy } = analyzeImageContent(context.descriptionDocument)
			return imageHeavy
		},
		link: {
			path: 'settings/description',
			title: defineMessage({
				id: 'nags.edit-description.title',
				defaultMessage: 'Edit description',
			}),
			shouldShow: (context: NagContext) =>
				context.currentRoute !== 'type-project-settings-description',
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
			const { hasEmptyAltText } = analyzeImageContent(context.descriptionDocument)
			return hasEmptyAltText
		},
		link: {
			path: 'settings/description',
			title: defineMessage({
				id: 'nags.edit-description.title',
				defaultMessage: 'Edit description',
			}),
			shouldShow: (context: NagContext) =>
				context.currentRoute !== 'type-project-settings-description',
		},
	},
]
