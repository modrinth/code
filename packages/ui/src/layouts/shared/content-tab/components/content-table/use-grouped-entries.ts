import type { Ref } from 'vue'
import { computed, watch } from 'vue'

import { useScrollViewport } from '#ui/composables/virtual-scroll'

import type { ContentCardTableItem } from '../../types'
import type { ContentTableSection } from './types'

export const SECTION_HEADER_HEIGHT = 48
export const SECTION_EMPTY_HEIGHT = 56

export type GroupedTableEntry =
	| {
			kind: 'header'
			key: string
			section: ContentTableSection
			top: number
	  }
	| {
			kind: 'empty'
			key: string
			section: ContentTableSection
			top: number
	  }
	| {
			kind: 'row'
			key: string
			row: ContentCardTableItem
			section: ContentTableSection
			indexInSection: number
			rowIndex: number
			top: number
	  }

export interface GroupedEntriesOptions {
	sections: Ref<ContentTableSection[]>
	rowHeight: Ref<number>
	enabled: Ref<boolean>
	bufferSize?: number
}

export function useGroupedEntries(options: GroupedEntriesOptions) {
	const bufferSize = options.bufferSize ?? 5
	const {
		listContainer,
		relativeScrollTop,
		viewportHeight,
		scrollContainer,
		containerOffset,
		syncScrollState,
	} = useScrollViewport()

	const layout = computed(() => {
		const entries: GroupedTableEntry[] = []
		const rows: ContentCardTableItem[] = []
		let top = 0
		for (const section of options.sections.value) {
			entries.push({ kind: 'header', key: `header:${section.id}`, section, top })
			top += SECTION_HEADER_HEIGHT
			if (section.collapsed) continue
			if (section.items.length === 0 && section.emptyLabel) {
				entries.push({ kind: 'empty', key: `empty:${section.id}`, section, top })
				top += SECTION_EMPTY_HEIGHT
				continue
			}
			section.items.forEach((row, indexInSection) => {
				entries.push({
					kind: 'row',
					key: `row:${row.id}`,
					row,
					section,
					indexInSection,
					rowIndex: rows.length,
					top,
				})
				rows.push(row)
				top += options.rowHeight.value
			})
		}
		return { entries, rows, totalHeight: top }
	})

	function heightOf(entry: GroupedTableEntry) {
		if (entry.kind === 'header') return SECTION_HEADER_HEIGHT
		if (entry.kind === 'empty') return SECTION_EMPTY_HEIGHT
		return options.rowHeight.value
	}

	function entryIndexAt(offset: number) {
		const entries = layout.value.entries
		let low = 0
		let high = entries.length - 1
		while (low < high) {
			const middle = Math.ceil((low + high) / 2)
			if (entries[middle].top <= offset) low = middle
			else high = middle - 1
		}
		return low
	}

	const visibleRange = computed(() => {
		const count = layout.value.entries.length
		if (!options.enabled.value) return { start: 0, end: count }
		if (!listContainer.value || !scrollContainer.value) {
			return { start: 0, end: Math.min(count, 20) }
		}
		const first = entryIndexAt(relativeScrollTop.value)
		const last = entryIndexAt(relativeScrollTop.value + viewportHeight.value)
		return {
			start: Math.max(0, first - bufferSize),
			end: Math.min(count, last + bufferSize + 1),
		}
	})

	const visibleEntries = computed(() =>
		layout.value.entries.slice(visibleRange.value.start, visibleRange.value.end),
	)
	const visibleTop = computed(() => visibleEntries.value[0]?.top ?? 0)

	function scrollToRow(rowId: string) {
		const entry = layout.value.entries.find(
			(candidate) => candidate.kind === 'row' && candidate.row.id === rowId,
		)
		if (!entry) return
		syncScrollState()
		if (!listContainer.value || !scrollContainer.value) return
		const height = heightOf(entry)
		const top = containerOffset.value + entry.top - (viewportHeight.value - height) / 2
		scrollContainer.value.scrollTo({ top: Math.max(0, top), behavior: 'instant' })
		syncScrollState()
	}

	watch(() => layout.value.totalHeight, syncScrollState)

	return {
		listContainer,
		visibleEntries,
		visibleTop,
		totalHeight: computed(() => layout.value.totalHeight),
		rows: computed(() => layout.value.rows),
		scrollToRow,
	}
}
