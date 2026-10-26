import { useLocalStorage } from '@vueuse/core'
import type { MaybeRefOrGetter, Ref } from 'vue'
import { computed, toValue } from 'vue'

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

const DEFAULT_ENABLED_COLUMNS: FileColumn[] = ['size', 'items', 'modified', 'created']

/**
 * Which detail columns the file listing shows. When `adjustable`, users pick the columns
 * (persisted to local storage) and can switch details off entirely; otherwise every column is
 * enabled. Enabled columns are then dropped, least important first, whenever they would
 * squeeze file names below a readable width.
 */
export function useFileColumns(
	containerWidth: Ref<number | undefined>,
	adjustable: MaybeRefOrGetter<boolean>,
) {
	const storedDetailsEnabled = useLocalStorage('files-details-enabled', true)
	const storedEnabledColumns = useLocalStorage<FileColumn[]>(
		'files-visible-columns',
		DEFAULT_ENABLED_COLUMNS,
	)

	const detailsEnabled = computed({
		get: () => !toValue(adjustable) || storedDetailsEnabled.value,
		set: (value) => (storedDetailsEnabled.value = value),
	})
	const enabledColumns = computed(() =>
		toValue(adjustable) ? storedEnabledColumns.value : DEFAULT_ENABLED_COLUMNS,
	)

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

	function toggleColumn(id: FileColumn) {
		storedEnabledColumns.value = storedEnabledColumns.value.includes(id)
			? storedEnabledColumns.value.filter((column) => column !== id)
			: [...storedEnabledColumns.value, id]
	}

	return {
		detailsEnabled,
		enabledColumns,
		shownColumns,
		toggleColumn,
	}
}
