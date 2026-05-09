import { computed, type Ref } from 'vue'

import { useVIntl } from '#ui/composables/i18n'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '#ui/providers'

export type PowerAction = 'Start' | 'Stop' | 'Restart' | 'Kill'

export function useServerPowerAction(options?: { disabled?: Ref<boolean> }) {
	const { formatMessage } = useVIntl()
	const client = injectModrinthClient()
	const { serverId, server, powerState, isSyncingContent, busyReasons } =
		injectModrinthServerContext()
	const { addNotification } = injectNotificationManager()

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

	const isBlockedByPropsOrBusy = computed(
		() => Boolean(options?.disabled?.value) || busyReasons.value.length > 0,
	)

	const busyTooltip = computed(() => {
		if (isStarting.value) return 'Your server is starting'
		return busyReasons.value.length > 0 ? formatMessage(busyReasons.value[0].reason) : undefined
	})

	const canTakeAction = computed(() => !isTransitioning.value && !isBlockedByPropsOrBusy.value)

	const canKill = computed(
		() =>
			!isBlockedByPropsOrBusy.value && (isStopping.value || isRunning.value || isStarting.value),
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

	async function sendPowerAction(action: PowerAction) {
		try {
			await client.archon.servers_v0.power(serverId, action)
		} catch (error) {
			console.error(`Error performing ${action} on server:`, error)
			addNotification({
				type: 'error',
				title: `Failed to ${action.toLowerCase()} server`,
				text: 'An error occurred while performing this action.',
			})
		}
	}

	function initiateAction(action: PowerAction) {
		if (action === 'Kill') {
			if (!canKill.value) return
		} else {
			if (!canTakeAction.value) return
		}
		void sendPowerAction(action)
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
