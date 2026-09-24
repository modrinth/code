import { computed, ref, watch } from 'vue'

import type { UpdateAllItem, UpdateAllSelection } from './update-all-modal-types'

export function useUpdateAllSelection(items: () => UpdateAllItem[]) {
	const state = ref(new Map<string, { selected: boolean; versionId?: string }>())

	function reconcile(reset = false) {
		state.value = new Map<string, { selected: boolean; versionId?: string }>(
			items().map((item) => {
				const previous = reset ? undefined : state.value.get(item.id)
				const versions = item.versions.filter((version) => version.id !== item.currentVersion.id)
				const version =
					versions.find((version) => version.id === previous?.versionId) ??
					versions.find((version) => version.id === item.initialVersionId) ??
					versions[0]

				return [
					item.id,
					{
						selected: previous?.selected ?? item.initiallySelected ?? true,
						versionId: version?.id,
					},
				]
			}),
		)
	}

	watch(items, () => reconcile(), { immediate: true, deep: true })

	const rows = computed(() =>
		items().map((item) => {
			const versions = item.versions.filter((version) => version.id !== item.currentVersion.id)
			const selection = state.value.get(item.id)
			return {
				...item,
				versions,
				version: versions.find((version) => version.id === selection?.versionId),
				selected: !!selection?.selected && versions.length > 0,
			}
		}),
	)
	const availableRows = computed(() => rows.value.filter((row) => row.version))
	const selections = computed<UpdateAllSelection[]>(() =>
		rows.value.flatMap((row) =>
			row.selected && row.version
				? [{ id: row.id, projectId: row.project.id, version: row.version }]
				: [],
		),
	)
	const allSelected = computed(
		() => availableRows.value.length > 0 && selections.value.length === availableRows.value.length,
	)
	const indeterminate = computed(() => selections.value.length > 0 && !allSelected.value)

	function selectItem(id: string, selected: boolean) {
		const selection = state.value.get(id)
		if (selection?.versionId) selection.selected = selected
	}

	function selectAll(selected: boolean) {
		for (const row of availableRows.value) selectItem(row.id, selected)
	}

	function selectVersion(id: string, versionId: string) {
		const selection = state.value.get(id)
		if (selection && rows.value.find((row) => row.id === id)?.versions.some((v) => v.id === versionId)) {
			selection.versionId = versionId
			selection.selected = true
		}
	}

	return {
		rows,
		selections,
		allSelected,
		indeterminate,
		selectItem,
		selectAll,
		selectVersion,
		reset: () => reconcile(true),
	}
}
