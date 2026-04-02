import { useStorage } from '@vueuse/core'
import { computed, type Ref, ref } from 'vue'

import { useVIntl } from '#ui/composables/i18n'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '#ui/providers'

export type PowerAction = 'Start' | 'Stop' | 'Restart' | 'Kill'

export type PanelActionConfirmModalController = {
	show: () => void
	hide: () => void
}

export function useServerPowerAction(options?: {
	disabled?: Ref<boolean>
	confirmModalRef?: Ref<PanelActionConfirmModalController | null>
}) {
	const { formatMessage } = useVIntl()
	const client = injectModrinthClient()
	const { serverId, server, powerState, busyReasons } = injectModrinthServerContext()
	const { addNotification } = injectNotificationManager()
	const pendingAction = ref<PowerAction | null>(null)
	const dontAskAgain = ref(false)

	const userPreferences = useStorage(`pyro-server-${serverId}-preferences`, {
		powerDontAskAgain: false,
	})

	const isInstalling = computed(() => server.value.status === 'installing')
	const isRunning = computed(() => powerState.value === 'running')
	const isStopping = computed(() => powerState.value === 'stopping')
	const isTransitioning = computed(
		() => powerState.value === 'starting' || powerState.value === 'stopping',
	)
	const showStopButton = computed(() => isRunning.value || isStopping.value)

	const busyTooltip = computed(() =>
		busyReasons.value.length > 0 ? formatMessage(busyReasons.value[0].reason) : undefined,
	)

	const canTakeAction = computed(
		() => !isTransitioning.value && !options?.disabled?.value && busyReasons.value.length === 0,
	)

	const primaryActionText = computed(() => {
		switch (powerState.value) {
			case 'starting':
				return 'Starting...'
			case 'stopping':
				return 'Stopping...'
			case 'running':
				return 'Restart'
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

		if (action === 'Start') {
			void sendPowerAction(action)
			return
		}

		pendingAction.value = action

		if (userPreferences.value.powerDontAskAgain) {
			executePendingAction()
		} else {
			options?.confirmModalRef?.value?.show()
		}
	}

	function handlePrimaryAction() {
		initiateAction(isRunning.value ? 'Restart' : 'Start')
	}

	function executePendingAction() {
		if (!pendingAction.value) return

		if (!canTakeAction.value) {
			resetPendingAction()
			return
		}

		void sendPowerAction(pendingAction.value)

		if (dontAskAgain.value) {
			userPreferences.value.powerDontAskAgain = true
		}

		resetPendingAction()
	}

	function resetPendingAction() {
		options?.confirmModalRef?.value?.hide()
		pendingAction.value = null
		dontAskAgain.value = false
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
		pendingAction,
		dontAskAgain,
		sendPowerAction,
		initiateAction,
		handlePrimaryAction,
		executePendingAction,
		resetPendingAction,
	}
}
