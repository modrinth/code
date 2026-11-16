import { useElementSize } from '@vueuse/core'
import type { Ref } from 'vue'
import { computed } from 'vue'

import type { ContentCardTableItem } from '../../types'
import type { ContentTableAction, ContentTableLayout } from './types'

const ACTION_WIDTHS = {
	synced: 42,
	version: 36,
	toggle: 48,
	remove: 36,
	menu: 36,
}
const ACTION_GAP = 8
const COLUMN_GAP = 16
const ROW_PADDING = 24
const MIN_PROJECT_WIDTH = 240
const MIN_VERSION_WIDTH = 288

export function showsVersionAction(row: ContentCardTableItem, actions: Set<ContentTableAction>) {
	return (
		!!row.locked ||
		(actions.has('update') && !!row.hasUpdate) ||
		(actions.has('switchVersion') && !!row.version && !row.hideSwitchVersion)
	)
}

export function showsToggleAction(row: ContentCardTableItem, actions: Set<ContentTableAction>) {
	return actions.has('toggle') && row.enabled !== undefined && !row.hideToggle
}

export function showsRemoveAction(row: ContentCardTableItem, actions: Set<ContentTableAction>) {
	return actions.has('remove') && !row.hideDelete
}

function rowActionsWidth(
	row: ContentCardTableItem,
	actions: Set<ContentTableAction>,
	extraWidths: number[],
) {
	const widths = [...extraWidths]
	if (row.synced) widths.push(ACTION_WIDTHS.synced)
	if (showsVersionAction(row, actions)) widths.push(ACTION_WIDTHS.version)
	if (showsToggleAction(row, actions)) widths.push(ACTION_WIDTHS.toggle)
	if (showsRemoveAction(row, actions)) widths.push(ACTION_WIDTHS.remove)
	if (row.overflowOptions?.length) widths.push(ACTION_WIDTHS.menu)
	const total = widths.reduce((sum, width) => sum + width, 0)
	return total + Math.max(0, widths.length - 1) * ACTION_GAP
}

export interface ContentTableLayoutOptions {
	rows: Ref<ContentCardTableItem[]>
	actions: Ref<ContentTableAction[] | undefined>
	showVersion: Ref<boolean>
	showEnabledFor: Ref<boolean>
	extraActionWidths: (row: ContentCardTableItem) => number[]
	tableRef: Ref<HTMLElement | null>
	enabledForMeasureRef: Ref<HTMLElement | null>
	actionsLabelRef: Ref<HTMLElement | null>
}

export function useContentTableLayout(options: ContentTableLayoutOptions) {
	const { width: tableWidth } = useElementSize(options.tableRef)
	const { width: enabledForWidth } = useElementSize(options.enabledForMeasureRef)
	const { width: actionsLabelWidth } = useElementSize(options.actionsLabelRef)

	const hasActionsColumn = computed(() => options.actions.value !== undefined)
	const actionSet = computed(() => new Set(options.actions.value ?? []))

	const actionsWidth = computed(() =>
		options.rows.value.reduce(
			(maximum, row) =>
				Math.max(
					maximum,
					rowActionsWidth(row, actionSet.value, options.extraActionWidths(row).slice()),
				),
			actionsLabelWidth.value,
		),
	)

	const controlColumns = computed(() => [
		...(options.showEnabledFor.value ? [Math.ceil(enabledForWidth.value)] : []),
		...(hasActionsColumn.value ? [Math.ceil(actionsWidth.value)] : []),
	])

	const layout = computed<ContentTableLayout>(() => {
		const controls = controlColumns.value
		const controlBudget = controls.reduce((sum, width) => sum + width, 0)
		const compactBudget =
			ROW_PADDING + controlBudget + controls.length * COLUMN_GAP + MIN_PROJECT_WIDTH
		const wideBudget =
			compactBudget + (options.showVersion.value ? MIN_VERSION_WIDTH + COLUMN_GAP : 0)
		if (tableWidth.value >= wideBudget) return 'wide'
		if (tableWidth.value >= compactBudget) return 'compact'
		if (
			tableWidth.value >=
			ROW_PADDING + controlBudget + Math.max(0, controls.length - 1) * COLUMN_GAP
		) {
			return 'stacked'
		}
		return 'narrow'
	})

	const separateVersion = computed(() => options.showVersion.value && layout.value === 'wide')
	const stacked = computed(() => layout.value === 'stacked' || layout.value === 'narrow')

	const rowHeight = computed(() => {
		if (!stacked.value) return 74
		const controlRows =
			layout.value === 'narrow'
				? controlColumns.value.length
				: Math.min(1, controlColumns.value.length)
		return 25 + 48 + controlRows * 44
	})

	const columnStyles = computed(() => {
		const controls = controlColumns.value.map((width) => `${width}px`)
		let columns: string[]
		if (layout.value === 'narrow') {
			columns = ['minmax(0, 1fr)']
		} else if (stacked.value) {
			columns = controls.length > 1 ? [controls[0], 'minmax(0, 1fr)'] : ['minmax(0, 1fr)']
		} else {
			columns = [
				'minmax(0, 1.2fr)',
				...(options.showEnabledFor.value ? [`minmax(${controls[0]}, 0.4fr)`] : []),
				...(separateVersion.value ? ['minmax(0, 1fr)'] : []),
				...(hasActionsColumn.value ? [`${Math.ceil(actionsWidth.value)}px`] : []),
			]
		}
		return {
			'--content-columns': columns.join(' '),
			'--content-row-height': `${rowHeight.value}px`,
		}
	})

	return {
		layout,
		stacked,
		separateVersion,
		hasActionsColumn,
		actionSet,
		rowHeight,
		columnStyles,
	}
}
