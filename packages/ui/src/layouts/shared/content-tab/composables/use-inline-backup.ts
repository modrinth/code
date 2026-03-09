import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { onBeforeRouteLeave } from 'vue-router'

import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '#ui/providers/'

export function useInlineBackup(backupName: string | (() => string)) {
	const serverCtx = injectModrinthServerContext(null)

	if (!serverCtx) {
		return {
			available: false as const,
			isBackingUp: ref(false),
			backupFailed: ref(false),
			backupComplete: ref(false),
			startBackup: async () => {},
			disableClose: computed(() => false),
		}
	}

	const client = injectModrinthClient()
	const { addNotification } = injectNotificationManager()
	const { serverId, worldId, backupsState } = serverCtx

	const isBackingUp = ref(false)
	const backupFailed = ref(false)
	const backupComplete = ref(false)
	const createdBackupId = ref<string | null>(null)

	const disableClose = computed(() => isBackingUp.value)

	// Watch backupsState for websocket progress events from Kyros
	watch(
		() => {
			if (!createdBackupId.value) return null
			return backupsState.get(createdBackupId.value)
		},
		(entry) => {
			if (!entry?.create) return

			if (entry.create.state === 'done') {
				isBackingUp.value = false
				backupComplete.value = true
			} else if (entry.create.state === 'failed' || entry.create.state === 'cancelled') {
				isBackingUp.value = false
				backupFailed.value = true
			}
		},
		{ deep: true },
	)

	// Fallback: poll the REST API in case websocket events don't arrive
	let pollTimer: ReturnType<typeof setInterval> | null = null

	function stopPolling() {
		if (pollTimer !== null) {
			clearInterval(pollTimer)
			pollTimer = null
		}
	}

	async function pollBackupStatus(backupId: string) {
		if (!isBackingUp.value) {
			stopPolling()
			return
		}

		try {
			const backup = await client.archon.backups_v1.get(serverId, worldId.value!, backupId)

			if (!backup.ongoing) {
				stopPolling()

				if (backup.interrupted) {
					isBackingUp.value = false
					backupFailed.value = true
				} else {
					isBackingUp.value = false
					backupComplete.value = true
				}
			}
		} catch {
			stopPolling()
			isBackingUp.value = false
			backupFailed.value = true
		}
	}

	async function startBackup() {
		if (!worldId.value) return

		const name = typeof backupName === 'function' ? backupName() : backupName

		isBackingUp.value = true
		backupFailed.value = false
		backupComplete.value = false
		createdBackupId.value = null

		try {
			const { id } = await client.archon.backups_v1.create(serverId, worldId.value, { name })
			createdBackupId.value = id

			stopPolling()
			pollTimer = setInterval(() => pollBackupStatus(id), 3000)
		} catch (error) {
			isBackingUp.value = false
			backupFailed.value = true

			const message = error instanceof Error ? error.message : String(error)
			const isRateLimit = message.includes('429')
			addNotification({
				type: 'error',
				title: 'Error creating backup',
				text: isRateLimit ? "You're creating backups too fast." : message,
			})
		}
	}

	function handleBeforeUnload(e: BeforeUnloadEvent) {
		if (isBackingUp.value) {
			e.preventDefault()
			return ''
		}
	}

	if (typeof window !== 'undefined') {
		watch(isBackingUp, (operating) => {
			if (operating) {
				window.addEventListener('beforeunload', handleBeforeUnload)
			} else {
				window.removeEventListener('beforeunload', handleBeforeUnload)
			}
		})

		onBeforeUnmount(() => {
			window.removeEventListener('beforeunload', handleBeforeUnload)
			stopPolling()
		})

		onBeforeRouteLeave(() => {
			if (isBackingUp.value) {
				return window.confirm('A backup is being created. Are you sure you want to leave?')
			}
			return true
		})
	}

	return {
		available: true as const,
		isBackingUp,
		backupFailed,
		backupComplete,
		startBackup,
		disableClose,
	}
}
