import type { Archon } from '@modrinth/api-client'
import type { ComputedRef, Ref } from 'vue'
import { computed, ref, watch } from 'vue'

type BackupQueueBackup = Archon.BackupsQueue.v1.BackupQueueBackup

export function useBackupsSelection(
	visibleBackups: Ref<BackupQueueBackup[]>,
	displayOrderedBackups: ComputedRef<BackupQueueBackup[]>,
) {
	const selectedIds = ref<Set<string>>(new Set())

	watch(visibleBackups, () => {
		const ids = new Set(visibleBackups.value.map((b) => b.id))
		const next = new Set<string>()
		for (const id of selectedIds.value) {
			if (ids.has(id)) next.add(id)
		}
		if (next.size !== selectedIds.value.size) {
			selectedIds.value = next
		}
	})

	function toggleSelection(id: string) {
		const next = new Set(selectedIds.value)
		if (next.has(id)) next.delete(id)
		else next.add(id)
		selectedIds.value = next
	}

	function selectAll() {
		selectedIds.value = new Set(visibleBackups.value.map((b) => b.id))
	}

	function deselectAll() {
		selectedIds.value = new Set()
	}

	function toggleSelectAll() {
		if (allSelected.value) deselectAll()
		else selectAll()
	}

	const allSelected = computed(
		() =>
			visibleBackups.value.length > 0 &&
			visibleBackups.value.every((b) => selectedIds.value.has(b.id)),
	)

	const someSelected = computed(() => {
		const vis = visibleBackups.value
		if (vis.length === 0) return false
		let n = 0
		for (const b of vis) {
			if (selectedIds.value.has(b.id)) n++
		}
		return n > 0 && n < vis.length
	})

	const selectedBackups = computed(() =>
		displayOrderedBackups.value.filter((b) => selectedIds.value.has(b.id)),
	)

	return {
		selectedIds,
		toggleSelection,
		selectAll,
		deselectAll,
		toggleSelectAll,
		allSelected,
		someSelected,
		selectedBackups,
	}
}
