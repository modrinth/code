<template>
	<div class="relative flex select-none flex-col gap-6" data-pyro-server-manager-root>
		<Admonition
			v-if="inspectingError && isConnected && !isWsAuthIncorrect"
			data-pyro-servers-inspecting-error
			type="critical"
			:header="`${serverData?.name} shut down unexpectedly.`"
			dismissible
			@dismiss="dismissError"
		>
			<template v-if="inspectingError.analysis.problems.length">
				<p class="m-0 text-sm opacity-80">
					We automatically analyzed the logs and found the following:
				</p>
				<div class="mt-2 flex flex-col gap-2">
					<div
						v-for="problem in inspectingError.analysis.problems"
						:key="problem.message"
						class="bg-raised-bg/30 rounded-xl px-3 py-2"
					>
						<p class="m-0 text-sm font-semibold">{{ problem.message }}</p>
						<ul v-if="problem.solutions.length" class="m-0 ml-4 mt-1.5 flex flex-col gap-1">
							<li
								v-for="solution in problem.solutions"
								:key="solution.message"
								class="text-sm opacity-80"
							>
								{{ solution.message }}
							</li>
						</ul>
					</div>
				</div>
			</template>
			<template v-else-if="serverPowerState === 'crashed'">
				<template v-if="powerStateDetails?.oom_killed">
					The server stopped because it ran out of memory. There may be a memory leak caused by a
					mod or plugin, or you may need to upgrade your Modrinth Server.
				</template>
				<template v-else-if="powerStateDetails?.exit_code !== undefined">
					Your server exited with code {{ powerStateDetails.exit_code }}.
					<template v-if="powerStateDetails.exit_code === 1">
						There may be a mod or plugin causing the issue, or an issue with your server
						configuration.
					</template>
				</template>
				<template v-else> We could not determine the specific cause of the crash. </template>
				<p class="m-0 mt-2">You can try restarting the server.</p>
			</template>
			<template v-else>
				We could not find any specific problems, but you can try restarting the server.
			</template>
		</Admonition>

		<div class="flex flex-col-reverse gap-6 md:flex-col">
			<ServerManageStats
				:data="!isWsAuthIncorrect ? stats : undefined"
				:loading="isWsAuthIncorrect"
			/>

			<div class="flex h-[700px] flex-col gap-4">
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
	</div>
</template>

<script setup lang="ts">
import { useStorage } from '@vueuse/core'
import { computed, ref, watch } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import { useModrinthServersConsole } from '#ui/composables'
import { ConsolePageLayout, provideConsoleManager } from '#ui/layouts/shared/console'
import { injectModrinthClient, injectModrinthServerContext } from '#ui/providers'

import ServerManageStats from './components/ServerManageStats.vue'

const client = injectModrinthClient()
const {
	server: serverData,
	serverId,
	isConnected,
	isWsAuthIncorrect,
	stats,
	powerState: serverPowerState,
	powerStateDetails,
} = injectModrinthServerContext()
const modrinthServersConsole = useModrinthServersConsole()

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
	loading: computed(() => !isConnected.value || isWsAuthIncorrect.value),
	onClear: () => {
		modrinthServersConsole.clear()
	},
	shareDisabled: computed(() => !isConnected.value),
})

interface ErrorData {
	id: string
	name: string
	type: string
	version: string
	title: string
	analysis: {
		problems: Array<{
			message: string
			counter: number
			entry: {
				level: number
				time: string | null
				prefix: string
				lines: Array<{ number: number; content: string }>
			}
			solutions: Array<{ message: string }>
		}>
		information: Array<{
			message: string
			counter: number
			label: string
			value: string
			entry: {
				level: number
				time: string | null
				prefix: string
				lines: Array<{ number: number; content: string }>
			}
		}>
	}
}

const DISMISS_DURATION_MS = 30 * 60 * 1000
const inspectingError = ref<ErrorData | null>(null)
const dismissedUntil = useStorage(`modrinth-crash-dismissed-${serverId}`, 0)

const isDismissed = () => Date.now() < dismissedUntil.value

const inspectError = async () => {
	if (isDismissed()) return

	try {
		const blob = await client.kyros.files_v0.downloadFile('/logs/latest.log')
		const log = await blob.text()
		if (!log) return

		const response = await fetch('https://api.mclo.gs/1/analyse', {
			method: 'POST',
			headers: {
				'Content-Type': 'application/x-www-form-urlencoded',
			},
			body: new URLSearchParams({
				content: log,
			}),
		})

		if (!response.ok) {
			inspectingError.value = null
			return
		}

		const data = (await response.json()) as Partial<ErrorData>
		if (data.analysis && Array.isArray(data.analysis.problems)) {
			inspectingError.value = data as ErrorData
		} else {
			inspectingError.value = null
		}
	} catch (error) {
		console.error('Failed to analyze logs:', error)
		inspectingError.value = null
	}
}

const dismissError = () => {
	dismissedUntil.value = Date.now() + DISMISS_DURATION_MS
	inspectingError.value = null
}

const clearError = () => {
	inspectingError.value = null
}

watch(
	() => serverPowerState.value,
	(newVal) => {
		if (newVal === 'crashed' && !powerStateDetails.value?.oom_killed) {
			void inspectError()
		} else {
			clearError()
		}
	},
)

if (serverPowerState.value === 'crashed' && !powerStateDetails.value?.oom_killed) {
	void inspectError()
}
</script>
