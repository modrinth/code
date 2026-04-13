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
	const { serverId, server, powerState, busyReasons } = injectModrinthServerContext()
	const { addNotification } = injectNotificationManager()

	const isInstalling = computed(() => server.value.status === 'installing')
	const isRunning = computed(() => powerState.value === 'running')
	const isStopping = computed(() => powerState.value === 'stopping')
	const isStarting = computed(() => powerState.value === 'starting')
	const isTransitioning = computed(() => isStarting.value || isStopping.value)
	const showStopButton = computed(() => isRunning.value || isStarting.value)

	const busyTooltip = computed(() => {
		if (isStarting.value) return 'Your server is starting'
		return busyReasons.value.length > 0 ? formatMessage(busyReasons.value[0].reason) : undefined
	})

	const canTakeAction = computed(
		() => !isTransitioning.value && !options?.disabled?.value && busyReasons.value.length === 0,
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
		if (!canTakeAction.value) return
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
		showStopButton,
		busyTooltip,
		canTakeAction,
		primaryActionText,
		sendPowerAction,
		initiateAction,
		handlePrimaryAction,
	}
}
