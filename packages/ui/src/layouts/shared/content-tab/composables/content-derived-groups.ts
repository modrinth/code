import dayjs from 'dayjs'

import { defineMessages, type VIntlFormatters } from '#ui/composables/i18n'
import { formatCategory, loaderMessages } from '#ui/utils/tag-messages'

import type { ContentItem } from '../types'

export type ContentDerivedGroupBy = 'category' | 'author' | 'date' | 'license'

export interface ContentDerivedSection {
	id: string
	name: string
	items: ContentItem[]
}

const messages = defineMessages({
	uncategorized: { id: 'content.groups.derived.uncategorized', defaultMessage: 'Uncategorized' },
	unknownAuthor: { id: 'content.groups.derived.unknown-author', defaultMessage: 'Unknown author' },
	unknownLicense: {
		id: 'content.groups.derived.unknown-license',
		defaultMessage: 'Unknown license',
	},
	unknownDate: { id: 'content.groups.derived.unknown-date', defaultMessage: 'Unknown date' },
	today: { id: 'content.groups.derived.today', defaultMessage: 'Today' },
	yesterday: { id: 'content.groups.derived.yesterday', defaultMessage: 'Yesterday' },
	thisWeek: { id: 'content.groups.derived.this-week', defaultMessage: 'This week' },
	thisMonth: { id: 'content.groups.derived.this-month', defaultMessage: 'This month' },
})

interface DerivedKey {
	key: string | null
	label: string
	order?: number
}

export interface DerivedGroupFormatters {
	formatMessage: VIntlFormatters['formatMessage']
	formatMonth: (date: Date) => string
}

function categoryKey(item: ContentItem, { formatMessage }: DerivedGroupFormatters): DerivedKey {
	const category = item.project?.categories?.find((candidate) => !(candidate in loaderMessages))
	return category
		? { key: category, label: formatCategory(formatMessage, category) }
		: { key: null, label: formatMessage(messages.uncategorized) }
}

function authorKey(item: ContentItem, { formatMessage }: DerivedGroupFormatters): DerivedKey {
	return item.owner
		? { key: item.owner.id, label: item.owner.name }
		: { key: null, label: formatMessage(messages.unknownAuthor) }
}

function licenseLabel(license: { id: string; name?: string | null }) {
	if (license.name?.trim()) return license.name
	return license.id.replace(/^LicenseRef-/, '').replace(/-/g, ' ')
}

function licenseKey(item: ContentItem, { formatMessage }: DerivedGroupFormatters): DerivedKey {
	const license = item.project?.license
	return license?.id
		? { key: license.id, label: licenseLabel(license) }
		: { key: null, label: formatMessage(messages.unknownLicense) }
}

function dateKey(item: ContentItem, formatters: DerivedGroupFormatters): DerivedKey {
	const added = item.date_added ? dayjs(item.date_added) : null
	if (!added?.isValid()) {
		return { key: null, label: formatters.formatMessage(messages.unknownDate) }
	}
	const now = dayjs()
	const order = -added.valueOf()
	if (added.isSame(now, 'day')) {
		return { key: 'today', label: formatters.formatMessage(messages.today), order }
	}
	if (added.isSame(now.subtract(1, 'day'), 'day')) {
		return { key: 'yesterday', label: formatters.formatMessage(messages.yesterday), order }
	}
	if (added.isSame(now, 'week')) {
		return { key: 'this-week', label: formatters.formatMessage(messages.thisWeek), order }
	}
	if (added.isSame(now, 'month')) {
		return { key: 'this-month', label: formatters.formatMessage(messages.thisMonth), order }
	}
	return {
		key: `month:${added.format('YYYY-MM')}`,
		label: formatters.formatMonth(added.toDate()),
		order,
	}
}

const keyFunctions: Record<
	ContentDerivedGroupBy,
	(item: ContentItem, formatters: DerivedGroupFormatters) => DerivedKey
> = {
	category: categoryKey,
	author: authorKey,
	date: dateKey,
	license: licenseKey,
}

export function buildDerivedSections(
	groupBy: ContentDerivedGroupBy,
	items: ContentItem[],
	formatters: DerivedGroupFormatters,
): ContentDerivedSection[] {
	const sections = new Map<string, ContentDerivedSection & { order: number; unknown: boolean }>()
	for (const item of items) {
		const { key, label, order } = keyFunctions[groupBy](item, formatters)
		const id = `${groupBy}:${key ?? 'unknown'}`
		const section = sections.get(id) ?? {
			id,
			name: label,
			items: [],
			order: order ?? 0,
			unknown: key === null,
		}
		section.items.push(item)
		if (order !== undefined) section.order = Math.min(section.order, order)
		sections.set(id, section)
	}

	return [...sections.values()]
		.sort((a, b) => {
			if (a.unknown !== b.unknown) return a.unknown ? 1 : -1
			if (groupBy === 'date') return a.order - b.order
			return a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: 'base' })
		})
		.map(({ id, name, items: sectionItems }) => ({ id, name, items: sectionItems }))
}

export function contentDerivedSectionId(
	groupBy: ContentDerivedGroupBy,
	item: ContentItem,
	formatters: DerivedGroupFormatters,
) {
	return `${groupBy}:${keyFunctions[groupBy](item, formatters).key ?? 'unknown'}`
}
