import { computed } from 'vue'

import { useVIntl } from '#ui/composables/i18n'
import { useServerPermissions } from '#ui/composables/server-permissions'
import { injectModrinthServerContext } from '#ui/providers'

const installingReasonIds = new Set<string | undefined>([
	'servers.busy.installing',
	'servers.busy.syncing-content',
])
const backupReasonIds = new Set<string | undefined>([
	'servers.busy.backup-creating',
	'servers.busy.backup-restoring',
])

export type ContentBusyState = ReturnType<typeof useContentBusyState>

export function useContentBusyState() {
	const { formatMessage } = useVIntl()
	const { server, busyReasons } = injectModrinthServerContext()
	const { canSetup, permissionDeniedMessage } = useServerPermissions()

	const actionDisabled = computed(() => !canSetup.value || busyReasons.value.length > 0)

	const isInstallingContent = computed(
		() =>
			server.value?.status === 'installing' ||
			busyReasons.value.some((r) => installingReasonIds.has(r.reason.id)),
	)

	const setupBusyMessage = computed(() => {
		if (!canSetup.value) return permissionDeniedMessage.value
		const reason = busyReasons.value.find(
			(r) =>
				!(isInstallingContent.value && installingReasonIds.has(r.reason.id)) &&
				!backupReasonIds.has(r.reason.id),
		)
		return reason ? formatMessage(reason.reason) : null
	})

	const contentBusyMessage = computed(() => {
		if (!canSetup.value) return permissionDeniedMessage.value
		return busyReasons.value.length > 0 ? formatMessage(busyReasons.value[0].reason) : null
	})

	return {
		canSetup,
		permissionDeniedMessage,
		actionDisabled,
		isInstallingContent,
		setupBusyMessage,
		contentBusyMessage,
	}
}
