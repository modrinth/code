<script setup lang="ts">
import {
	DownloadIcon,
	LeftArrowIcon,
	PlayIcon,
	SaveIcon,
	ServerIcon,
	SpinnerIcon,
	StopCircleIcon,
	TerminalSquareIcon,
} from '@modrinth/assets'
import {
	ButtonStyled,
	commonMessages,
	defineMessages,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { computed, nextTick, onMounted, onUnmounted, ref, useTemplateRef, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import {
	get as getServer,
	getConfig,
	getLog,
	install as installServer,
	isRunning,
	onServerLog,
	onServerProcess,
	sendCommand,
	SERVER_SOFTWARE,
	type ServerInstance,
	setConfig,
	start as startServer,
	stop as stopServer,
} from '@/helpers/server'

const { formatMessage } = useVIntl()
const { addNotification, handleError } = injectNotificationManager()
const route = useRoute()
const router = useRouter()

const id = computed(() => String(route.params.id))

const CONFIG_FILES = [
	'server.properties',
	'bukkit.yml',
	'spigot.yml',
	'paper-global.yml',
	'purpur.yml',
	'fabric-server-launcher.properties',
]

const messages = defineMessages({
	back: { id: 'app.local-server.back', defaultMessage: 'All servers' },
	start: { id: 'app.local-server.start', defaultMessage: 'Start' },
	stop: { id: 'app.local-server.stop', defaultMessage: 'Stop' },
	download: { id: 'app.local-server.download', defaultMessage: 'Download server' },
	retry: { id: 'app.local-server.retry', defaultMessage: 'Retry download' },
	consoleTab: { id: 'app.local-server.tab.console', defaultMessage: 'Console' },
	configTab: { id: 'app.local-server.tab.config', defaultMessage: 'Configuration' },
	commandPlaceholder: {
		id: 'app.local-server.command-placeholder',
		defaultMessage: 'Type a command and press Enter…',
	},
	send: { id: 'app.local-server.send', defaultMessage: 'Send' },
	notInstalled: {
		id: 'app.local-server.not-installed',
		defaultMessage: 'This server has not been downloaded yet.',
	},
	installing: { id: 'app.local-server.installing', defaultMessage: 'Downloading server…' },
	running: { id: 'app.local-server.running', defaultMessage: 'Running' },
	stopped: { id: 'app.local-server.stopped', defaultMessage: 'Stopped' },
	configFileLabel: { id: 'app.local-server.config-file', defaultMessage: 'File' },
	configEmpty: {
		id: 'app.local-server.config-empty',
		defaultMessage: 'This file does not exist yet. Saving will create it.',
	},
	configSaved: { id: 'app.local-server.config-saved', defaultMessage: 'Configuration saved' },
})

const server = ref<ServerInstance | null>(null)
const running = ref(false)
const installing = ref(false)
const activeTab = ref<'console' | 'config'>('console')

const consoleLines = ref<string[]>([])
const consoleEl = useTemplateRef<HTMLDivElement>('consoleEl')
const command = ref('')

const selectedFile = ref(CONFIG_FILES[0])
const configContents = ref('')
const savingConfig = ref(false)

let unlistenLog: (() => void) | undefined
let unlistenProcess: (() => void) | undefined

const softwareLabel = computed(
	() =>
		SERVER_SOFTWARE.find((s) => s.value === server.value?.software)?.label ??
		server.value?.software,
)

const installed = computed(() => server.value?.install_stage === 'installed')

async function scrollConsoleToBottom() {
	await nextTick()
	if (consoleEl.value) {
		consoleEl.value.scrollTop = consoleEl.value.scrollHeight
	}
}

async function refreshServer() {
	try {
		server.value = await getServer(id.value)
		running.value = await isRunning(id.value)
		installing.value = server.value?.install_stage === 'installing'
	} catch (e) {
		handleError(e)
	}
}

async function reloadConsole() {
	consoleLines.value = await getLog(id.value)
	scrollConsoleToBottom()
}

async function loadConfig() {
	try {
		configContents.value = await getConfig(id.value, selectedFile.value)
	} catch (e) {
		handleError(e)
	}
}

watch(selectedFile, loadConfig)

async function doInstall() {
	installing.value = true
	try {
		await installServer(id.value)
		await refreshServer()
	} catch (e) {
		handleError(e)
	} finally {
		installing.value = false
		await refreshServer()
	}
}

async function doStart() {
	try {
		await startServer(id.value)
		consoleLines.value = []
		setTimeout(refreshServer, 300)
	} catch (e) {
		handleError(e)
	}
}

async function doStop() {
	try {
		await stopServer(id.value)
		setTimeout(refreshServer, 300)
	} catch (e) {
		handleError(e)
	}
}

async function submitCommand() {
	const value = command.value.trim()
	if (!value) return
	try {
		await sendCommand(id.value, value)
		command.value = ''
	} catch (e) {
		handleError(e)
	}
}

async function saveConfig() {
	savingConfig.value = true
	try {
		await setConfig(id.value, selectedFile.value, configContents.value)
		addNotification({
			type: 'success',
			title: formatMessage(messages.configSaved),
			text: selectedFile.value,
		})
	} catch (e) {
		handleError(e)
	} finally {
		savingConfig.value = false
	}
}

onMounted(async () => {
	await refreshServer()
	await reloadConsole()
	await loadConfig()

	unlistenLog = await onServerLog((payload) => {
		if (payload.server_id !== id.value) return
		consoleLines.value.push(payload.line)
		if (consoleLines.value.length > 10_000) {
			consoleLines.value.splice(0, consoleLines.value.length - 10_000)
		}
		scrollConsoleToBottom()
	})

	unlistenProcess = await onServerProcess((payload) => {
		if (payload.server_id !== id.value) return
		running.value = payload.event === 'launched'
		refreshServer()
	})
})

onUnmounted(() => {
	unlistenLog?.()
	unlistenProcess?.()
})
</script>

<template>
	<div class="flex h-full flex-col p-6">
		<button
			class="mb-4 flex w-fit items-center gap-1 border-0 bg-transparent text-secondary hover:text-contrast"
			@click="router.push('/servers')"
		>
			<LeftArrowIcon /> {{ formatMessage(messages.back) }}
		</button>

		<div v-if="server" class="flex flex-col gap-4">
			<div class="flex flex-wrap items-center justify-between gap-4">
				<div class="flex items-center gap-3">
					<div class="flex h-14 w-14 items-center justify-center rounded-xl bg-button-bg">
						<ServerIcon class="h-7 w-7 text-contrast" />
					</div>
					<div>
						<h1 class="m-0 text-2xl font-extrabold text-contrast">{{ server.name }}</h1>
						<p class="m-0 text-secondary">
							{{ softwareLabel }} · {{ server.minecraft_version }}
							<span :class="running ? 'text-green' : 'text-secondary'">
								· {{ formatMessage(running ? messages.running : messages.stopped) }}
							</span>
						</p>
					</div>
				</div>

				<div class="flex items-center gap-2">
					<template v-if="!installed">
						<ButtonStyled color="brand">
							<button :disabled="installing" @click="doInstall">
								<SpinnerIcon v-if="installing" class="animate-spin" />
								<DownloadIcon v-else />
								{{
									formatMessage(
										installing
											? messages.installing
											: server.install_stage === 'failed'
												? messages.retry
												: messages.download,
									)
								}}
							</button>
						</ButtonStyled>
					</template>
					<template v-else>
						<ButtonStyled v-if="!running" color="brand">
							<button @click="doStart"><PlayIcon /> {{ formatMessage(messages.start) }}</button>
						</ButtonStyled>
						<ButtonStyled v-else color="red">
							<button @click="doStop"><StopCircleIcon /> {{ formatMessage(messages.stop) }}</button>
						</ButtonStyled>
					</template>
				</div>
			</div>

			<div class="flex gap-2">
				<button
					class="flex items-center gap-1 rounded-lg border-0 px-3 py-2 font-semibold"
					:class="
						activeTab === 'console' ? 'bg-button-bg text-contrast' : 'bg-transparent text-secondary'
					"
					@click="activeTab = 'console'"
				>
					<TerminalSquareIcon /> {{ formatMessage(messages.consoleTab) }}
				</button>
				<button
					class="flex items-center gap-1 rounded-lg border-0 px-3 py-2 font-semibold"
					:class="
						activeTab === 'config' ? 'bg-button-bg text-contrast' : 'bg-transparent text-secondary'
					"
					@click="activeTab = 'config'"
				>
					<SaveIcon /> {{ formatMessage(messages.configTab) }}
				</button>
			</div>

			<div v-if="activeTab === 'console'" class="flex flex-col gap-2">
				<div
					ref="consoleEl"
					class="h-[26rem] overflow-y-auto whitespace-pre-wrap break-words rounded-xl bg-bg-black p-3 font-mono text-sm text-primary"
				>
					<p v-if="!installed" class="text-secondary">
						{{ formatMessage(messages.notInstalled) }}
					</p>
					<div v-for="(line, index) in consoleLines" :key="index">{{ line }}</div>
				</div>
				<form class="flex gap-2" @submit.prevent="submitCommand">
					<input
						v-model="command"
						type="text"
						:disabled="!running"
						:placeholder="formatMessage(messages.commandPlaceholder)"
						class="w-full rounded-lg bg-button-bg px-3 py-2 font-mono text-contrast"
					/>
					<ButtonStyled color="brand">
						<button type="submit" :disabled="!running || !command.trim()">
							{{ formatMessage(messages.send) }}
						</button>
					</ButtonStyled>
				</form>
			</div>

			<div v-else class="flex flex-col gap-2">
				<div class="flex items-center gap-2">
					<label class="text-sm font-semibold text-contrast" for="config-file">
						{{ formatMessage(messages.configFileLabel) }}
					</label>
					<select
						id="config-file"
						v-model="selectedFile"
						class="rounded-lg bg-button-bg px-3 py-2 text-contrast"
					>
						<option v-for="file in CONFIG_FILES" :key="file" :value="file">{{ file }}</option>
					</select>
				</div>
				<p v-if="!configContents" class="m-0 text-sm text-secondary">
					{{ formatMessage(messages.configEmpty) }}
				</p>
				<textarea
					v-model="configContents"
					spellcheck="false"
					class="h-[24rem] w-full resize-none rounded-xl bg-bg-black p-3 font-mono text-sm text-primary"
				></textarea>
				<div class="flex justify-end">
					<ButtonStyled color="brand">
						<button :disabled="savingConfig" @click="saveConfig">
							<SpinnerIcon v-if="savingConfig" class="animate-spin" />
							<SaveIcon v-else />
							{{ formatMessage(commonMessages.saveButton) }}
						</button>
					</ButtonStyled>
				</div>
			</div>
		</div>

		<div v-else class="flex flex-1 items-center justify-center">
			<SpinnerIcon class="h-8 w-8 animate-spin text-secondary" />
		</div>
	</div>
</template>
