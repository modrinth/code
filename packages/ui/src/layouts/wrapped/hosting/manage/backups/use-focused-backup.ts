import { computed, nextTick, onBeforeUnmount, type Ref, ref, watch } from 'vue'
import { useRoute } from 'vue-router'

const HIGHLIGHT_DURATION_MS = 5_000

export function useFocusedBackup(backupIds: Ref<string[]>) {
	const route = useRoute()
	const focusedBackupId = computed(() =>
		typeof route.query.backup === 'string' ? route.query.backup : null,
	)

	const highlightedBackupId = ref<string | null>(null)
	let handledBackupId: string | null = null
	let highlightTimeout: ReturnType<typeof setTimeout> | undefined

	function clearHighlight() {
		highlightedBackupId.value = null
		clearTimeout(highlightTimeout)
	}

	watch(
		[focusedBackupId, backupIds],
		async ([backupId, ids]) => {
			if (!backupId) {
				handledBackupId = null
				clearHighlight()
				return
			}
			if (backupId === handledBackupId || !ids.includes(backupId)) return

			handledBackupId = backupId
			highlightedBackupId.value = backupId
			clearTimeout(highlightTimeout)
			highlightTimeout = setTimeout(clearHighlight, HIGHLIGHT_DURATION_MS)

			if (typeof document === 'undefined') return
			await nextTick()
			const target = [
				...document.querySelectorAll(`[data-backup-id="${CSS.escape(backupId)}"]`),
			].find((element) => element.getClientRects().length > 0)
			target?.scrollIntoView({ block: 'center', behavior: 'smooth' })
		},
		{ immediate: true },
	)

	onBeforeUnmount(() => clearTimeout(highlightTimeout))

	return { highlightedBackupId }
}
