import { normalizeProjectType } from '#ui/utils/common-messages'

import type { ContentItem, ManagedContentSummaryItem, ManagedContentSummaryType } from '../types'

const summaryOrder: ManagedContentSummaryType[] = [
	'mod',
	'plugin',
	'datapack',
	'resourcepack',
	'shader',
]

function normalizeSummaryType(type: string): ManagedContentSummaryType | null {
	const normalized = normalizeProjectType(type)
	switch (normalized) {
		case 'mod':
		case 'plugin':
		case 'datapack':
		case 'resourcepack':
		case 'shader':
			return normalized
		default:
			return null
	}
}

export function dedupeManagedContentItems(items: ContentItem[]): ContentItem[] {
	const seenKeys = new Set<string>()
	return items.filter((item) => {
		const key = item.file_path
			? `path:${item.file_path}`
			: item.file_name
				? `name:${item.file_name}`
				: `id:${item.id}`
		if (seenKeys.has(key)) return false

		seenKeys.add(key)
		return true
	})
}

export function summarizeManagedContent(items: ContentItem[]): ManagedContentSummaryItem[] {
	const counts = new Map<ManagedContentSummaryType, number>()
	for (const item of dedupeManagedContentItems(items)) {
		const type = normalizeSummaryType(item.project_type)
		if (type) counts.set(type, (counts.get(type) ?? 0) + 1)
	}

	return summaryOrder.flatMap((type) => {
		const count = counts.get(type) ?? 0
		return count > 0 ? [{ type, count }] : []
	})
}
