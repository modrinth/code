import { useLocalStorage } from '@vueuse/core'
import type { Ref } from 'vue'
import { computed } from 'vue'

import type { FileSortField } from '../types'

export const FILE_COLUMNS_ORDER = ['size', 'items', 'created', 'modified'] as const

export type FileColumn = (typeof FILE_COLUMNS_ORDER)[number]

export interface FileColumnDefinition {
	id: FileColumn
	/** Column width in px, shared by the header and the rows so they line up. */
	width: number
	sortField: FileSortField
}

export const FILE_COLUMNS: FileColumnDefinition[] = [
	{ id: 'size', width: 100, sortField: 'size' },
	{ id: 'items', width: 90, sortField: 'items' },
	{ id: 'created', width: 150, sortField: 'created' },
	{ id: 'modified', width: 150, sortField: 'modified' },
]

/** The order columns are dropped in when there isn't room for them next to the name. */
const FILE_COLUMN_DROP_ORDER: FileColumn[] = ['created', 'items', 'size', 'modified']

/** Space always kept for a file's name before any detail column is shown. */
const NAME_MIN_WIDTH = 280
/** Row padding, selection checkbox, icon and actions menu. */
const ROW_CHROME_WIDTH = 170
const COLUMN_GAP = 24

function columnsWidth(columns: FileColumn[]) {
	return columns.reduce(
		(total, id) =>
			total + (FILE_COLUMNS.find((column) => column.id === id)?.width ?? 0) + COLUMN_GAP,
		0,
	)
}

/**
 * Which detail columns the file listing shows. Users pick the columns (persisted to local
 * storage) and can switch details off entirely; enabled columns are then dropped, least
 * important first, whenever they would squeeze file names below a readable width.
 */
export function useFileColumns(containerWidth: Ref<number | undefined>) {
	const detailsEnabled = useLocalStorage('files-details-enabled', true)
	const enabledColumns = useLocalStorage<FileColumn[]>('files-visible-columns', [
		'size',
		'items',
		'modified',
		'created',
	])

	const shownColumns = computed<FileColumn[]>(() => {
		if (!detailsEnabled.value || containerWidth.value == null) return []

		const available = containerWidth.value - ROW_CHROME_WIDTH - NAME_MIN_WIDTH
		let columns = FILE_COLUMNS.map((column) => column.id).filter((id) =>
			enabledColumns.value.includes(id),
		)

		for (const id of FILE_COLUMN_DROP_ORDER) {
			if (columnsWidth(columns) <= available) break
			columns = columns.filter((column) => column !== id)
		}
		return columns
	})

	/** Whether some details aren't visible as columns, so rows should offer them in a tooltip. */
	const hasHiddenDetails = computed(() => shownColumns.value.length < FILE_COLUMNS.length)

	function toggleColumn(id: FileColumn) {
		enabledColumns.value = enabledColumns.value.includes(id)
			? enabledColumns.value.filter((column) => column !== id)
			: [...enabledColumns.value, id]
	}

	return {
		detailsEnabled,
		enabledColumns,
		shownColumns,
		hasHiddenDetails,
		toggleColumn,
	}
}
