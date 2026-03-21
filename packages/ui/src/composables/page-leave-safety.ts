import type { ComputedRef, Ref } from 'vue'
import { onBeforeUnmount, ref, watch } from 'vue'
import { onBeforeRouteLeave } from 'vue-router'

import type ConfirmLeaveModal from '#ui/components/modal/ConfirmLeaveModal.vue'

export function usePageLeaveSafety(dirty: Ref<boolean> | ComputedRef<boolean>) {
	const confirmLeaveModal = ref<InstanceType<typeof ConfirmLeaveModal>>()

	function handleBeforeUnload(e: BeforeUnloadEvent) {
		if (dirty.value) {
			e.preventDefault()
		}
	}

	if (typeof window !== 'undefined') {
		watch(dirty, (isDirty) => {
			if (isDirty) {
				window.addEventListener('beforeunload', handleBeforeUnload)
			} else {
				window.removeEventListener('beforeunload', handleBeforeUnload)
			}
		})

		onBeforeUnmount(() => {
			window.removeEventListener('beforeunload', handleBeforeUnload)
		})

		onBeforeRouteLeave(async () => {
			if (dirty.value) {
				return (await confirmLeaveModal.value?.prompt()) ?? false
			}
			return true
		})
	}

	return { confirmLeaveModal }
}
