import { computed, ref } from 'vue'

import type { UnsavedChangesController } from './types'

const emptyState: Record<string, unknown> = {}

export function useUnsavedChangesHost() {
	const popup = ref<{ nudge: () => void } | null>(null)
	const controller = ref<UnsavedChangesController | null>(null)

	const original = computed(() => controller.value?.getOriginal() ?? emptyState)
	const modified = computed(() => controller.value?.getModified() ?? emptyState)
	const saving = computed(() => controller.value?.isSaving() ?? false)
	const hasUnsavedChanges = computed(
		() => (controller.value?.hasChanges() ?? false) || saving.value,
	)

	function canLeave(): boolean {
		if (!hasUnsavedChanges.value) return true
		popup.value?.nudge()
		return false
	}

	function register(next: UnsavedChangesController | null): void {
		controller.value = next
	}

	function reset(): void {
		controller.value?.reset()
	}

	function save(): void {
		void controller.value?.save()
	}

	return { popup, original, modified, saving, hasUnsavedChanges, canLeave, register, reset, save }
}
