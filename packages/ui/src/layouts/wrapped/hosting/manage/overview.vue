<template>
	<div class="relative flex select-none flex-col gap-6" data-pyro-server-manager-root>
		<div class="flex flex-col gap-4">
			<ServerManageStats
				:data="!isWsAuthIncorrect ? stats : undefined"
				:loading="isWsAuthIncorrect"
			/>

			<div class="flex min-h-[700px] flex-col gap-2">
				<span class="text-2xl font-semibold text-contrast">Console</span>

				<ConsolePageLayout />
			</div>
		</div>

		<div
			v-if="isWsAuthIncorrect"
			class="absolute inset-0 flex flex-col items-center justify-center bg-bg"
		>
			<h2>Could not connect to the server.</h2>
			<p>
				An error occurred while attempting to connect to your server. Please try refreshing the
				page. (WebSocket Authentication Failed)
			</p>
		</div>

		<button
			v-if="showAdvancedDebugInfo"
			class="self-start rounded-lg bg-surface-3 px-3 py-1 text-sm text-contrast hover:brightness-125"
			@click="downloadLog4jDebug"
		>
			Download WS debug JSON
		</button>
	</div>
</template>

<script setup lang="ts">
// No ReadyTransition wrapper: console and ServerManageStats own their loading UX; there is no single TanStack "ready" gate for this tab.
import type { Mclogs } from '@modrinth/api-client'
import { useStorage } from '@vueuse/core'
import { computed, ref, watch } from 'vue'

import { useModrinthServersConsole } from '#ui/composables'
import { ConsolePageLayout, provideConsoleManager } from '#ui/layouts/shared/console'
import { injectModrinthClient, injectModrinthServerContext } from '#ui/providers'

import ServerManageStats from './components/ServerManageStats.vue'

const props = withDefaults(
	defineProps<{
		showAdvancedDebugInfo?: boolean
	}>(),
	{
		showAdvancedDebugInfo: false,
	},
)

const client = injectModrinthClient()
const {
	server: _serverData,
	serverId,
	isConnected,
	isWsAuthIncorrect,
	stats,
	powerState: serverPowerState,
	powerStateDetails: _powerStateDetails,
} = injectModrinthServerContext()
const modrinthServersConsole = useModrinthServersConsole()

watch(
	() => props.showAdvancedDebugInfo,
	(enabled) => {
		modrinthServersConsole.setWsEventCaptureEnabled(enabled)
	},
	{ immediate: true },
)

const crashAnalysis = ref<Mclogs.Insights.v1.InsightsResponse | null>(null)
const DISMISS_DURATION_MS = 30 * 60 * 1000
const dismissedUntil = useStorage(`modrinth-crash-dismissed-${serverId}`, 0)

const isDismissed = () => Date.now() < dismissedUntil.value

const inspectError = async () => {
	if (isDismissed()) return

	try {
		const blob = await client.kyros.files_v0.downloadFile('/logs/latest.log')
		const log = await blob.text()
		if (!log) return

		const data = await client.mclogs.insights_v1.analyse(log)
		if (data.analysis?.problems?.length) {
			crashAnalysis.value = data
		} else {
			crashAnalysis.value = null
		}
	} catch (error) {
		console.error('Failed to analyze logs:', error)
		crashAnalysis.value = null
	}
}

const dismissCrash = () => {
	dismissedUntil.value = Date.now() + DISMISS_DURATION_MS
	crashAnalysis.value = null
}

provideConsoleManager({
	logLines: modrinthServersConsole.output,
	sendCommand: (cmd: string) => {
		try {
			client.archon.sockets.send(serverId, { event: 'command', cmd })
		} catch (error) {
			console.error('Error sending command:', error)
		}
	},
	showCommandInput: true,
	disableCommandInput: computed(() => serverPowerState.value !== 'running'),
	loading: computed(() => !isConnected.value || isWsAuthIncorrect.value),
	onClear: async () => {
		modrinthServersConsole.clear()
		try {
			await client.kyros.logs_v1.clear()
		} catch (error) {
			console.error('Failed to clear server logs:', error)
		}
	},
	shareDisabled: computed(() => !isConnected.value),
	emptyStateType: 'server',
	crashAnalysis,
	onDismissCrash: dismissCrash,
})

watch(
	() => serverPowerState.value,
	(newVal) => {
		if (newVal === 'crashed') {
			void inspectError()
		} else {
			crashAnalysis.value = null
		}
	},
)

if (serverPowerState.value === 'crashed') {
	void inspectError()
}

const downloadLog4jDebug = () => {
	const events = modrinthServersConsole.getWsEventHistory()
	const blob = new Blob([JSON.stringify(events, null, 2)], { type: 'application/json' })
	const url = URL.createObjectURL(blob)
	const a = document.createElement('a')
	a.href = url
	a.download = `ws-debug-${serverId}-${Date.now()}.json`
	document.body.appendChild(a)
	a.click()
	document.body.removeChild(a)
	URL.revokeObjectURL(url)
}
</script>
