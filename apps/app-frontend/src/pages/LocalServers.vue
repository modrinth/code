<script setup lang="ts">
import {
	PlayIcon,
	PlusIcon,
	ServerIcon,
	SpinnerIcon,
	StopCircleIcon,
	TrashIcon,
} from '@modrinth/assets'
import {
	ButtonStyled,
	commonMessages,
	ConfirmModal,
	defineMessages,
	injectNotificationManager,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { onMounted, onUnmounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'

import {
	create as createServer,
	getRunning,
	getVersions,
	install as installServer,
	list as listServers,
	onServerProcess,
	remove as removeServer,
	SERVER_SOFTWARE,
	type ServerInstance,
	type ServerSoftware,
	start as startServer,
	stop as stopServer,
} from '@/helpers/server'

const { formatMessage } = useVIntl()
const { addNotification, handleError } = injectNotificationManager()
const router = useRouter()

const messages = defineMessages({
	title: { id: 'app.local-servers.title', defaultMessage: 'Servers' },
	subtitle: {
		id: 'app.local-servers.subtitle',
		defaultMessage: 'Host dedicated Minecraft servers on this computer.',
	},
	newServer: { id: 'app.local-servers.new-server', defaultMessage: 'New server' },
	emptyTitle: { id: 'app.local-servers.empty.title', defaultMessage: 'No servers yet' },
	emptyDescription: {
		id: 'app.local-servers.empty.description',
		defaultMessage:
			'Create a dedicated server to run on your own machine, with no port forwarding setup required for LAN play.',
	},
	createTitle: { id: 'app.local-servers.create.title', defaultMessage: 'New self-hosted server' },
	nameLabel: { id: 'app.local-servers.create.name', defaultMessage: 'Server name' },
	softwareLabel: { id: 'app.local-servers.create.software', defaultMessage: 'Server software' },
	versionLabel: { id: 'app.local-servers.create.version', defaultMessage: 'Minecraft version' },
	loadingVersions: {
		id: 'app.local-servers.create.loading-versions',
		defaultMessage: 'Loading versions…',
	},
	createAndInstall: {
		id: 'app.local-servers.create.submit',
		defaultMessage: 'Create & download',
	},
	installing: { id: 'app.local-servers.create.installing', defaultMessage: 'Downloading server…' },
	statusRunning: { id: 'app.local-servers.status.running', defaultMessage: 'Running' },
	statusStopped: { id: 'app.local-servers.status.stopped', defaultMessage: 'Stopped' },
	statusInstalling: { id: 'app.local-servers.status.installing', defaultMessage: 'Installing' },
	statusFailed: { id: 'app.local-servers.status.failed', defaultMessage: 'Install failed' },
	statusNotInstalled: {
		id: 'app.local-servers.status.not-installed',
		defaultMessage: 'Not installed',
	},
	deleteTitle: { id: 'app.local-servers.delete.title', defaultMessage: 'Delete server' },
	deleteDescription: {
		id: 'app.local-servers.delete.description',
		defaultMessage:
			'Are you sure you want to delete this server? All of its files and worlds will be permanently removed.',
	},
})

const servers = ref<ServerInstance[]>([])
const running = ref<Set<string>>(new Set())

const createModal = ref<InstanceType<typeof NewModal>>()
const confirmModal = ref<InstanceType<typeof ConfirmModal>>()
const pendingDelete = ref<ServerInstance | null>(null)

const newName = ref('')
const newSoftware = ref<ServerSoftware>('paper')
const newVersion = ref('')
const versions = ref<string[]>([])
const loadingVersions = ref(false)
const submitting = ref(false)

let unlistenProcess: (() => void) | undefined

async function refresh() {
	try {
		servers.value = await listServers()
		running.value = new Set((await getRunning()).map((r) => r.id))
	} catch (e) {
		handleError(e)
	}
}

async function loadVersions() {
	loadingVersions.value = true
	newVersion.value = ''
	try {
		versions.value = await getVersions(newSoftware.value)
		newVersion.value = versions.value[0] ?? ''
	} catch (e) {
		versions.value = []
		handleError(e)
	} finally {
		loadingVersions.value = false
	}
}

watch(newSoftware, () => loadVersions())

function openCreate() {
	newName.value = ''
	newSoftware.value = 'paper'
	createModal.value?.show()
	loadVersions()
}

async function submitCreate() {
	if (!newName.value.trim() || !newVersion.value) return
	submitting.value = true
	try {
		const server = await createServer(newName.value.trim(), newSoftware.value, newVersion.value)
		await refresh()
		createModal.value?.hide()
		// Kick off the download; navigate to the detail page to watch progress.
		router.push(`/servers/${encodeURIComponent(server.id)}`)
		installServer(server.id)
			.then(() => refresh())
			.catch((e) => handleError(e))
	} catch (e) {
		handleError(e)
	} finally {
		submitting.value = false
	}
}

async function toggleRunning(server: ServerInstance) {
	try {
		if (running.value.has(server.id)) {
			await stopServer(server.id)
		} else {
			await startServer(server.id)
		}
		setTimeout(refresh, 500)
	} catch (e) {
		handleError(e)
	}
}

function confirmDelete(server: ServerInstance) {
	pendingDelete.value = server
	confirmModal.value?.show()
}

async function doDelete() {
	if (!pendingDelete.value) return
	try {
		await removeServer(pendingDelete.value.id)
		addNotification({
			type: 'success',
			title: formatMessage(messages.deleteTitle),
			text: pendingDelete.value.name,
		})
		await refresh()
	} catch (e) {
		handleError(e)
	} finally {
		pendingDelete.value = null
	}
}

function statusFor(server: ServerInstance) {
	if (running.value.has(server.id)) return { label: messages.statusRunning, color: 'text-green' }
	switch (server.install_stage) {
		case 'installing':
			return { label: messages.statusInstalling, color: 'text-blue' }
		case 'failed':
			return { label: messages.statusFailed, color: 'text-red' }
		case 'installed':
			return { label: messages.statusStopped, color: 'text-secondary' }
		default:
			return { label: messages.statusNotInstalled, color: 'text-secondary' }
	}
}

function softwareLabel(software: ServerSoftware) {
	return SERVER_SOFTWARE.find((s) => s.value === software)?.label ?? software
}

onMounted(async () => {
	await refresh()
	unlistenProcess = await onServerProcess(() => refresh())
})

onUnmounted(() => {
	unlistenProcess?.()
})
</script>

<template>
	<div class="p-6">
		<div class="flex items-center justify-between gap-4 mb-6">
			<div>
				<h1 class="m-0 text-2xl font-extrabold text-contrast">
					{{ formatMessage(messages.title) }}
				</h1>
				<p class="m-0 mt-1 text-secondary">{{ formatMessage(messages.subtitle) }}</p>
			</div>
			<ButtonStyled color="brand">
				<button @click="openCreate"><PlusIcon /> {{ formatMessage(messages.newServer) }}</button>
			</ButtonStyled>
		</div>

		<div
			v-if="servers.length === 0"
			class="flex flex-col items-center justify-center gap-3 rounded-2xl bg-bg-raised p-12 text-center"
		>
			<ServerIcon class="h-12 w-12 text-secondary" />
			<h2 class="m-0 text-lg font-bold text-contrast">
				{{ formatMessage(messages.emptyTitle) }}
			</h2>
			<p class="m-0 max-w-md text-secondary">{{ formatMessage(messages.emptyDescription) }}</p>
			<ButtonStyled color="brand">
				<button @click="openCreate"><PlusIcon /> {{ formatMessage(messages.newServer) }}</button>
			</ButtonStyled>
		</div>

		<div v-else class="grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-3">
			<div
				v-for="server in servers"
				:key="server.id"
				class="flex cursor-pointer flex-col gap-3 rounded-2xl bg-bg-raised p-4 transition-transform hover:scale-[1.01]"
				@click="router.push(`/servers/${encodeURIComponent(server.id)}`)"
			>
				<div class="flex items-center gap-3">
					<div class="flex h-12 w-12 items-center justify-center rounded-xl bg-button-bg">
						<ServerIcon class="h-6 w-6 text-contrast" />
					</div>
					<div class="min-w-0">
						<h3 class="m-0 truncate text-base font-bold text-contrast">{{ server.name }}</h3>
						<p class="m-0 truncate text-sm text-secondary">
							{{ softwareLabel(server.software) }} · {{ server.minecraft_version }}
						</p>
					</div>
				</div>

				<div class="flex items-center justify-between">
					<span class="text-sm font-semibold" :class="statusFor(server).color">
						{{ formatMessage(statusFor(server).label) }}
					</span>
					<div class="flex items-center gap-2" @click.stop>
						<ButtonStyled :color="running.has(server.id) ? 'red' : 'brand'" circular>
							<button
								:disabled="server.install_stage !== 'installed'"
								@click="toggleRunning(server)"
							>
								<StopCircleIcon v-if="running.has(server.id)" />
								<SpinnerIcon
									v-else-if="server.install_stage === 'installing'"
									class="animate-spin"
								/>
								<PlayIcon v-else />
							</button>
						</ButtonStyled>
						<ButtonStyled circular>
							<button @click="confirmDelete(server)"><TrashIcon /></button>
						</ButtonStyled>
					</div>
				</div>
			</div>
		</div>

		<NewModal ref="createModal">
			<template #title>
				<span class="text-lg font-extrabold text-contrast">
					{{ formatMessage(messages.createTitle) }}
				</span>
			</template>

			<div class="flex w-[28rem] max-w-full flex-col gap-4">
				<div class="flex flex-col gap-1">
					<label class="text-sm font-semibold text-contrast" for="server-name">
						{{ formatMessage(messages.nameLabel) }}
					</label>
					<input
						id="server-name"
						v-model="newName"
						type="text"
						class="w-full rounded-lg bg-button-bg px-3 py-2 text-contrast"
						maxlength="64"
					/>
				</div>

				<div class="flex flex-col gap-1">
					<span class="text-sm font-semibold text-contrast">
						{{ formatMessage(messages.softwareLabel) }}
					</span>
					<div class="grid grid-cols-2 gap-2">
						<button
							v-for="option in SERVER_SOFTWARE"
							:key="option.value"
							class="flex flex-col gap-0.5 rounded-lg border-2 border-solid p-2 text-left transition-colors"
							:class="
								newSoftware === option.value
									? 'border-brand bg-brand-highlight'
									: 'border-transparent bg-button-bg'
							"
							@click="newSoftware = option.value"
						>
							<span class="font-semibold text-contrast">{{ option.label }}</span>
							<span class="text-xs text-secondary">{{ option.description }}</span>
						</button>
					</div>
				</div>

				<div class="flex flex-col gap-1">
					<label class="text-sm font-semibold text-contrast" for="server-version">
						{{ formatMessage(messages.versionLabel) }}
					</label>
					<select
						id="server-version"
						v-model="newVersion"
						:disabled="loadingVersions"
						class="w-full rounded-lg bg-button-bg px-3 py-2 text-contrast"
					>
						<option v-if="loadingVersions" value="">
							{{ formatMessage(messages.loadingVersions) }}
						</option>
						<option v-for="version in versions" :key="version" :value="version">
							{{ version }}
						</option>
					</select>
				</div>

				<div class="flex justify-end gap-2">
					<ButtonStyled>
						<button @click="createModal?.hide()">
							{{ formatMessage(commonMessages.cancelButton) }}
						</button>
					</ButtonStyled>
					<ButtonStyled color="brand">
						<button :disabled="submitting || !newName.trim() || !newVersion" @click="submitCreate">
							<SpinnerIcon v-if="submitting" class="animate-spin" />
							{{ formatMessage(submitting ? messages.installing : messages.createAndInstall) }}
						</button>
					</ButtonStyled>
				</div>
			</div>
		</NewModal>

		<ConfirmModal
			ref="confirmModal"
			:title="formatMessage(messages.deleteTitle)"
			:description="formatMessage(messages.deleteDescription)"
			:proceed-label="formatMessage(commonMessages.deleteLabel)"
			@proceed="doDelete"
		/>
	</div>
</template>
