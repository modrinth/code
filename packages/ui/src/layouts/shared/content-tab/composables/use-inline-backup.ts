import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { onBeforeRouteLeave } from 'vue-router'

import {
	injectAppBackup,
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '#ui/providers/'

export function useInlineBackup(backupName: string | (() => string)) {
	const serverCtx = injectModrinthServerContext(null)
	const appBackup = injectAppBackup(null)

	if (!serverCtx) {
		if (appBackup) {
			const isBackingUp = ref(false)
			const backupFailed = ref(false)
			const backupComplete = ref(false)

			return {
				available: true as const,
				isBackingUp,
				isCancelling: ref(false),
				backupFailed,
				backupComplete,
				backupCancelled: ref(false),
				externalBackupInProgress: computed(() => false),
				startBackup: async () => {
					isBackingUp.value = true
					backupFailed.value = false
					backupComplete.value = false
					try {
						await appBackup.createBackup()
						backupComplete.value = true
					} catch {
						backupFailed.value = true
					} finally {
						isBackingUp.value = false
					}
				},
				cancelBackup: async () => {},
			}
		}

		return {
			available: false as const,
			isBackingUp: ref(false),
			isCancelling: ref(false),
			backupFailed: ref(false),
			backupComplete: ref(false),
			backupCancelled: ref(false),
			externalBackupInProgress: ref(false),
			startBackup: async () => {},
			cancelBackup: async () => {},
		}
	}

	const client = injectModrinthClient()
	const { addNotification } = injectNotificationManager()
	const { serverId, worldId, backupsState, markBackupCancelled } = serverCtx

	const isBackingUp = ref(false)
	const backupFailed = ref(false)
	const backupComplete = ref(false)
	const backupCancelled = ref(false)
	const isCancelling = ref(false)
	const createdBackupId = ref<string | null>(null)

	const externalBackupInProgress = computed(() => {
		for (const [id, entry] of backupsState.entries()) {
			if (id !== createdBackupId.value && entry.create?.state === 'ongoing') return true
		}
		return false
	})

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
			} else if (entry.create.state === 'cancelled') {
				isBackingUp.value = false
				isCancelling.value = false
				backupCancelled.value = true
			} else if (entry.create.state === 'failed') {
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
		backupCancelled.value = false
		isCancelling.value = false
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

	async function cancelBackup() {
		if (!worldId.value || !createdBackupId.value || !isBackingUp.value) return

		isCancelling.value = true
		stopPolling()
		markBackupCancelled(createdBackupId.value)

		try {
			await client.archon.backups_v1.delete(serverId, worldId.value, createdBackupId.value)
			addNotification({
				type: 'info',
				title: 'Backup cancelled',
				text: 'The backup has been cancelled. You can create a new one or proceed without a backup.',
			})
		} catch {
			isCancelling.value = false
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
		isCancelling,
		backupFailed,
		backupComplete,
		backupCancelled,
		externalBackupInProgress,
		startBackup,
		cancelBackup,
	}
}
