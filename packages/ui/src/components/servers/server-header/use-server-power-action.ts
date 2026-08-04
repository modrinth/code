import type { Archon } from '@modrinth/api-client'
import { computed, type Ref } from 'vue'

import { useVIntl } from '#ui/composables/i18n'
import { useServerPermissions } from '#ui/composables/server-permissions'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '#ui/providers'

export type PowerAction = 'Start' | 'Stop' | 'Restart' | 'Kill'

const powerActionMap = {
	Start: 'start',
	Stop: 'stop',
	Restart: 'restart',
	Kill: 'kill',
} as const satisfies Record<PowerAction, Archon.Servers.v1.WorldPowerAction>

export function useServerPowerAction(options?: { disabled?: Ref<boolean> }) {
	const { formatMessage } = useVIntl()
	const client = injectModrinthClient()
	const { serverId, worldId, server, powerState, isSyncingContent, busyReasons } =
		injectModrinthServerContext()
	const { addNotification } = injectNotificationManager()
	const { canUsePowerActions, permissionDeniedMessage } = useServerPermissions()

	const isInstalling = computed(
		() =>
			server.value.status === 'installing' ||
			isSyncingContent.value ||
			busyReasons.value.some(
				(r) =>
					r.reason.id === 'servers.busy.installing' ||
					r.reason.id === 'servers.busy.syncing-content',
			),
	)
	const isRunning = computed(() => powerState.value === 'running')
	const isStopping = computed(() => powerState.value === 'stopping')
	const isStarting = computed(() => powerState.value === 'starting')
	const isTransitioning = computed(() => isStarting.value || isStopping.value)

	const showStopSplit = computed(() => isRunning.value || isStarting.value || isStopping.value)
	const showRestartButton = computed(() => isRunning.value || isStarting.value)

	const isBlockedByPropsBusyOrPermission = computed(
		() =>
			!canUsePowerActions.value ||
			Boolean(options?.disabled?.value) ||
			busyReasons.value.length > 0,
	)

	const busyTooltip = computed(() => {
		if (!canUsePowerActions.value) return permissionDeniedMessage.value
		if (!worldId.value) return 'Your server instance is loading'
		if (isStarting.value) return 'Your server is starting'
		return busyReasons.value.length > 0 ? formatMessage(busyReasons.value[0].reason) : undefined
	})

	const canTakeAction = computed(
		() => !!worldId.value && !isTransitioning.value && !isBlockedByPropsBusyOrPermission.value,
	)

	const canKill = computed(
		() =>
			!!worldId.value &&
			!isBlockedByPropsBusyOrPermission.value &&
			(isStopping.value || isRunning.value || isStarting.value),
	)

	const primaryActionText = computed(() => {
		switch (powerState.value) {
			case 'running':
			case 'starting':
				return 'Restart'
			case 'stopping':
				return 'Stopping'
			default:
				return 'Start'
		}
	})

	async function sendPowerAction(action: PowerAction, targetWorldId = worldId.value) {
		if (!targetWorldId) return

		try {
			await client.archon.servers_v1.powerWorld(serverId, targetWorldId, {
				action: powerActionMap[action],
			})
		} catch (error) {
			console.error(`Error performing ${action} on server:`, error)
			addNotification({
				type: 'error',
				title: `Failed to ${action.toLowerCase()} server`,
				text: 'An error occurred while performing this action.',
			})
		}
	}

	function initiateAction(action: PowerAction, targetWorldId = worldId.value) {
		if (action === 'Kill') {
			if (!canKill.value) return
		} else {
			if (!canTakeAction.value) return
		}
		void sendPowerAction(action, targetWorldId)
	}

	function handlePrimaryAction() {
		initiateAction(isRunning.value ? 'Restart' : 'Start')
	}

	return {
		isInstalling,
		isRunning,
		isStopping,
		isTransitioning,
		showStopSplit,
		showRestartButton,
		busyTooltip,
		canTakeAction,
		canKill,
		primaryActionText,
		sendPowerAction,
		initiateAction,
		handlePrimaryAction,
	}
}
