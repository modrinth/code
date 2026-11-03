import { injectModrinthClient, injectModrinthServerContext } from '../providers'
import { defineMessages, useVIntl } from './i18n'
import { useServerPermissions } from './server-permissions'

const START_TIMEOUT_MS = 180_000
const POLL_INTERVAL_MS = 500

const messages = defineMessages({
	busy: {
		id: 'servers.play.busy',
		defaultMessage: 'Wait for the current server operation to finish.',
	},
	startFailed: {
		id: 'servers.play.start-failed',
		defaultMessage:
			'The server stopped before it was ready. Check the console before trying again.',
	},
	startTimeout: {
		id: 'servers.play.start-timeout',
		defaultMessage:
			'The server is taking too long to start. Check the console before trying again.',
	},
})

/** Starts the current server when it is stopped and resolves once it reports running. */
export function useEnsureServerRunning() {
	const client = injectModrinthClient()
	const { serverId, busyReasons, powerState, isConnected } = injectModrinthServerContext()
	const { canUsePowerActions, permissionDeniedMessage } = useServerPermissions()
	const { formatMessage } = useVIntl()

	return async function ensureServerRunning(isCurrent: () => boolean = () => true) {
		const deadline = Date.now() + START_TIMEOUT_MS
		let startRequested = false
		let sawStarting = false
		while (isCurrent()) {
			if (busyReasons.value.length) throw new Error(formatMessage(messages.busy))
			if (isConnected.value) {
				const state = powerState.value
				if (state === 'running') return
				if (state === 'starting') sawStarting = true
				if (sawStarting && (state === 'stopped' || state === 'crashed' || state === 'stopping')) {
					throw new Error(formatMessage(messages.startFailed))
				}
				if (!startRequested && (state === 'stopped' || state === 'crashed')) {
					if (!canUsePowerActions.value) throw new Error(permissionDeniedMessage.value)
					startRequested = true
					await client.archon.servers_v0.power(serverId, 'Start')
				}
			}
			if (Date.now() >= deadline) throw new Error(formatMessage(messages.startTimeout))
			await new Promise((resolve) => setTimeout(resolve, POLL_INTERVAL_MS))
		}
	}
}
