<script setup lang="ts">
import { CogIcon, PlayIcon, ServerStackIcon } from '@modrinth/assets'
import { Button, defineMessages, injectNotificationManager, useVIntl } from '@modrinth/ui'
import { basename, join, tempDir } from '@tauri-apps/api/path'
import { readFile, remove } from '@tauri-apps/plugin-fs'
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'

import {
	fetchOwyxCatalog,
	getOwyxClientKey,
	getOwyxDemoFlag,
	getOwyxLocalApiFallback,
	getStoredOwyxApiBase,
	isSafeExternalHttpsUrl,
	sanitizeOwyxApiBase,
	setOwyxClientKey,
	setOwyxDemoFlag,
	setOwyxLocalApiFallback,
	setStoredOwyxApiBase,
	type OwyxServerEntry,
} from '@/helpers/owyx-api'
import { publishLibraryPackToCatalog } from '@/helpers/owyx-friends'
import { install_create_instance, installJobInstanceId } from '@/helpers/install'
import {
	export_instance_mrpack,
	get_pack_export_candidates,
	list as listInstances,
} from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'
import { useRootBreadcrumb } from '@/providers/breadcrumbs'
import { injectOwyxSiteSession } from '@/providers/owyx-site-session'

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const router = useRouter()
const owyx = injectOwyxSiteSession()

const messages = defineMessages({
	title: {
		id: 'owyx.servers.title',
		defaultMessage: 'Owyx Servers',
	},
	subtitle: {
		id: 'owyx.servers.subtitle',
		defaultMessage: 'Curated private servers from the Owyx control plane.',
	},
	empty: {
		id: 'owyx.servers.empty',
		defaultMessage: 'No servers published yet. Check back later or ask an admin.',
	},
	unreachable: {
		id: 'owyx.servers.unreachable',
		defaultMessage: 'Could not reach the Owyx API. Showing demo entry if enabled.',
	},
	play: { id: 'owyx.servers.play', defaultMessage: 'Play' },
	settings: { id: 'owyx.servers.settings', defaultMessage: 'Settings' },
	copyAddress: { id: 'owyx.servers.copy-address', defaultMessage: 'Copy address' },
	apiBase: { id: 'owyx.servers.api-base', defaultMessage: 'API base URL' },
	clientKey: {
		id: 'owyx.servers.client-key',
		defaultMessage: 'Client key (X-Owyx-Client-Key)',
	},
	demoToggle: {
		id: 'owyx.servers.demo-toggle',
		defaultMessage: 'Show demo seed when API is unreachable',
	},
	localFallbackToggle: {
		id: 'owyx.servers.local-fallback-toggle',
		defaultMessage: 'Also try http://127.0.0.1:3001 (dev only)',
	},
	saveSettings: { id: 'owyx.servers.save-settings', defaultMessage: 'Save API settings' },
	refresh: { id: 'owyx.servers.refresh', defaultMessage: 'Refresh' },
	version: { id: 'owyx.servers.version', defaultMessage: 'MC {version}' },
	demoBadge: { id: 'owyx.servers.demo-badge', defaultMessage: 'demo' },
	adminTools: { id: 'owyx.servers.admin-tools', defaultMessage: 'Admin tools' },
	adminHint: {
		id: 'owyx.servers.admin-hint',
		defaultMessage: 'Publish a library instance as a catalog pack, or open the site admin.',
	},
	openAdmin: {
		id: 'owyx.servers.open-admin',
		defaultMessage: 'Open catalog on owyx.site',
	},
	playing: { id: 'owyx.servers.playing', defaultMessage: 'Preparing…' },
	needsAccount: {
		id: 'owyx.servers.needs-account',
		defaultMessage: 'Sign in to Owyx to play on this server.',
	},
	publishTitle: {
		id: 'owyx.servers.publish-title',
		defaultMessage: 'Publish library instance to catalog',
	},
	publishInstance: {
		id: 'owyx.servers.publish-instance',
		defaultMessage: 'Instance',
	},
	publishServer: {
		id: 'owyx.servers.publish-server',
		defaultMessage: 'Bind to server (optional)',
	},
	publishNone: {
		id: 'owyx.servers.publish-none',
		defaultMessage: '— no bind —',
	},
	publishGo: {
		id: 'owyx.servers.publish-go',
		defaultMessage: 'Export & publish',
	},
	publishing: {
		id: 'owyx.servers.publishing',
		defaultMessage: 'Publishing…',
	},
	publishOk: {
		id: 'owyx.servers.publish-ok',
		defaultMessage: 'Published pack {id}',
	},
})

useRootBreadcrumb({
	slot: 'root',
	id: 'owyx-servers',
	label: formatMessage(messages.title),
	to: '/owyx-servers',
	visual: { type: 'icon', component: ServerStackIcon },
})

const servers = ref<OwyxServerEntry[]>([])
const loading = ref(false)
const playingId = ref<string | null>(null)
const apiError = ref(false)
const apiBase = ref(getStoredOwyxApiBase())
const clientKey = ref(getOwyxClientKey())
const demoEnabled = ref(getOwyxDemoFlag())
const localFallback = ref(getOwyxLocalApiFallback())
const copiedId = ref<string | null>(null)
const adminOpen = ref(false)
const instances = ref<GameInstance[]>([])
const publishInstanceId = ref('')
const publishServerId = ref('')
const publishing = ref(false)
const publishMsg = ref('')

const hasServers = computed(() => servers.value.length > 0)
const isAdmin = computed(() => owyx.isAdmin.value)

async function loadCatalog() {
	loading.value = true
	apiError.value = false
	try {
		const result = await fetchOwyxCatalog({
			baseUrl: sanitizeOwyxApiBase(apiBase.value),
			clientKey: clientKey.value,
			authToken: owyx.session.value?.token,
			demoFallback: demoEnabled.value,
			allowLocalFallback: localFallback.value,
		})
		servers.value = result.servers
		apiError.value = result.fromFallback
	} catch (e) {
		apiError.value = true
		servers.value = []
		handleError(e)
	} finally {
		loading.value = false
	}
}

async function loadInstances() {
	try {
		instances.value = await listInstances()
		if (!publishInstanceId.value && instances.value[0]) {
			publishInstanceId.value = instances.value[0].id
		}
	} catch {
		instances.value = []
	}
}

function saveSettings() {
	setStoredOwyxApiBase(sanitizeOwyxApiBase(apiBase.value))
	apiBase.value = getStoredOwyxApiBase()
	setOwyxClientKey(clientKey.value.trim())
	setOwyxDemoFlag(demoEnabled.value)
	setOwyxLocalApiFallback(localFallback.value)
	void loadCatalog()
}

async function copyAddress(server: OwyxServerEntry) {
	try {
		await navigator.clipboard.writeText(server.address)
		copiedId.value = server.id
		setTimeout(() => {
			if (copiedId.value === server.id) copiedId.value = null
		}, 1500)
	} catch (e) {
		handleError(e)
	}
}

async function openServerSettings(server: OwyxServerEntry) {
	const name = server.name
	await router.push({
		path: '/',
		query: { owyxServer: server.id, owyxServerName: name },
	})
	try {
		playingId.value = server.id
		const job = await install_create_instance({
			name,
			gameVersion: server.mcVersion || '1.21.1',
			loader: mapLoader(server.loader),
			loaderVersion: 'latest',
			iconPath: null,
		})
		const instanceId = installJobInstanceId(job)
		if (instanceId) {
			await router.push(`/instance/${encodeURIComponent(instanceId)}/options`)
		}
	} catch (e) {
		handleError(e)
	} finally {
		playingId.value = null
	}
}

function mapLoader(loader?: string): 'vanilla' | 'fabric' | 'forge' | 'quilt' | 'neoforge' {
	const l = (loader || 'vanilla').toLowerCase()
	if (l === 'fabric' || l === 'forge' || l === 'quilt' || l === 'neoforge') return l
	return 'vanilla'
}

async function playServer(server: OwyxServerEntry) {
	if (server.requiresAccount && !owyx.isSignedIn.value) {
		await owyx.signIn()
		if (!owyx.isSignedIn.value) return
	}
	playingId.value = server.id
	try {
		await navigator.clipboard.writeText(server.address).catch(() => undefined)
		const job = await install_create_instance({
			name: server.name,
			gameVersion: server.mcVersion || '1.21.1',
			loader: mapLoader(server.loader),
			loaderVersion: 'latest',
			iconPath: null,
		})
		const instanceId = installJobInstanceId(job)
		if (instanceId) {
			await router.push(`/instance/${encodeURIComponent(instanceId)}`)
		}
	} catch (e) {
		handleError(e)
	} finally {
		playingId.value = null
	}
}

function openAdminCatalog() {
	window.open('https://owyx.site/admin', '_blank', 'noopener,noreferrer')
}

async function publishInstance() {
	const inst = instances.value.find((i) => i.id === publishInstanceId.value)
	if (!inst) return
	publishing.value = true
	publishMsg.value = ''
	let exportPath = ''
	try {
		const candidates = await get_pack_export_candidates(inst.id)
		const included = candidates.filter((c) => c.defaultSelected).map((c) => c.path)
		const excluded = candidates.filter((c) => !c.defaultSelected).map((c) => c.path)
		const dir = await tempDir()
		exportPath = await join(dir, `owyx-publish-${Date.now()}.mrpack`)
		await export_instance_mrpack(
			inst.id,
			exportPath,
			included,
			excluded,
			'1.0.0',
			'Published from Owyx launcher',
			inst.name,
		)
		const bytes = await readFile(exportPath)
		const blob = new Blob([bytes], { type: 'application/zip' })
		const fileName = (await basename(exportPath)) || 'pack.mrpack'
		const loader = String(inst.loader || 'vanilla').toLowerCase()
		const result = await publishLibraryPackToCatalog({
			name: inst.name,
			minecraft: inst.game_version || '1.21.1',
			loader,
			description: `Exported from library instance ${inst.name}`,
			file: blob,
			fileName,
			serverId: publishServerId.value || null,
		})
		publishMsg.value = formatMessage(messages.publishOk, { id: result.packId })
		await loadCatalog()
	} catch (e) {
		handleError(e)
		publishMsg.value = e instanceof Error ? e.message : String(e)
	} finally {
		if (exportPath) {
			await remove(exportPath).catch(() => undefined)
		}
		publishing.value = false
	}
}

onMounted(() => {
	void loadCatalog()
	if (isAdmin.value) void loadInstances()
})
</script>

<template>
	<div class="owyx-servers mx-auto flex w-full max-w-4xl flex-col gap-6 px-4 py-6">
		<header class="flex flex-col gap-2">
			<h1 class="m-0 text-2xl font-semibold text-contrast">{{ formatMessage(messages.title) }}</h1>
			<p class="m-0 text-secondary">{{ formatMessage(messages.subtitle) }}</p>
		</header>

		<section
			v-if="isAdmin"
			class="flex flex-col gap-3 rounded-xl border border-solid border-surface-5 bg-surface-2 p-4"
		>
			<button
				type="button"
				class="m-0 flex w-full items-center justify-between border-none bg-transparent p-0 text-left cursor-pointer"
				@click="adminOpen = !adminOpen; if (adminOpen) void loadInstances()"
			>
				<span class="text-base font-semibold text-contrast">{{ formatMessage(messages.adminTools) }}</span>
				<span class="text-secondary text-sm">{{ adminOpen ? '▾' : '▸' }}</span>
			</button>
			<template v-if="adminOpen">
				<p class="m-0 text-sm text-secondary">{{ formatMessage(messages.adminHint) }}</p>

				<div class="flex flex-col gap-2 rounded-lg border border-solid border-surface-5 bg-surface-3 p-3">
					<p class="m-0 text-sm font-medium text-contrast">{{ formatMessage(messages.publishTitle) }}</p>
					<label class="flex flex-col gap-1 text-sm">
						<span class="text-secondary">{{ formatMessage(messages.publishInstance) }}</span>
						<select
							v-model="publishInstanceId"
							class="rounded-lg border border-solid border-surface-5 bg-surface-2 px-3 py-2 text-primary"
						>
							<option v-for="inst in instances" :key="inst.id" :value="inst.id">
								{{ inst.name }}
							</option>
						</select>
					</label>
					<label class="flex flex-col gap-1 text-sm">
						<span class="text-secondary">{{ formatMessage(messages.publishServer) }}</span>
						<select
							v-model="publishServerId"
							class="rounded-lg border border-solid border-surface-5 bg-surface-2 px-3 py-2 text-primary"
						>
							<option value="">{{ formatMessage(messages.publishNone) }}</option>
							<option v-for="server in servers" :key="server.id" :value="server.id">
								{{ server.name }}
							</option>
						</select>
					</label>
					<Button
						type="colored"
						color="brand"
						:disabled="publishing || !publishInstanceId"
						@click="publishInstance"
					>
						{{ publishing ? formatMessage(messages.publishing) : formatMessage(messages.publishGo) }}
					</Button>
					<p v-if="publishMsg" class="m-0 text-sm text-secondary">{{ publishMsg }}</p>
				</div>

				<label class="flex flex-col gap-1 text-sm">
					<span class="text-secondary">{{ formatMessage(messages.apiBase) }}</span>
					<input
						v-model="apiBase"
						class="rounded-lg border border-solid border-surface-5 bg-surface-3 px-3 py-2 text-primary"
						type="url"
					/>
				</label>
				<label class="flex flex-col gap-1 text-sm">
					<span class="text-secondary">{{ formatMessage(messages.clientKey) }}</span>
					<input
						v-model="clientKey"
						class="rounded-lg border border-solid border-surface-5 bg-surface-3 px-3 py-2 text-primary"
						type="password"
						autocomplete="off"
						placeholder="build-time / local placeholder — do not commit secrets"
					/>
				</label>
				<label class="flex items-center gap-2 text-sm text-secondary">
					<input v-model="demoEnabled" type="checkbox" />
					{{ formatMessage(messages.demoToggle) }}
				</label>
				<label class="flex items-center gap-2 text-sm text-secondary">
					<input v-model="localFallback" type="checkbox" />
					{{ formatMessage(messages.localFallbackToggle) }}
				</label>
				<div class="flex flex-wrap gap-2">
					<Button type="colored" color="brand" @click="saveSettings">
						{{ formatMessage(messages.saveSettings) }}
					</Button>
					<Button class="!bg-button-bg" :disabled="loading" @click="loadCatalog">
						{{ formatMessage(messages.refresh) }}
					</Button>
					<Button class="!bg-button-bg" @click="openAdminCatalog">
						{{ formatMessage(messages.openAdmin) }}
					</Button>
				</div>
				<p v-if="apiError" class="m-0 text-sm text-orange">{{ formatMessage(messages.unreachable) }}</p>
			</template>
		</section>

		<div v-else class="flex justify-end">
			<Button class="!bg-button-bg" :disabled="loading" @click="loadCatalog">
				{{ formatMessage(messages.refresh) }}
			</Button>
		</div>

		<section v-if="loading" class="text-secondary">…</section>

		<section
			v-else-if="!hasServers"
			class="rounded-xl border border-dashed border-surface-5 bg-surface-2 p-8 text-center text-secondary"
		>
			{{ formatMessage(messages.empty) }}
		</section>

		<ul v-else class="m-0 flex list-none flex-col gap-3 p-0">
			<li
				v-for="server in servers"
				:key="server.id"
				class="flex flex-col gap-3 rounded-xl border border-solid border-surface-5 bg-surface-2 p-4 sm:flex-row sm:items-center sm:justify-between"
			>
				<div class="flex min-w-0 items-start gap-3">
					<img
						v-if="server.iconUrl && isSafeExternalHttpsUrl(server.iconUrl)"
						:src="server.iconUrl.startsWith('/') ? `${sanitizeOwyxApiBase(apiBase)}${server.iconUrl}` : server.iconUrl"
						alt=""
						class="h-12 w-12 shrink-0 rounded-lg object-cover"
					/>
					<div
						v-else
						class="flex h-12 w-12 shrink-0 items-center justify-center rounded-lg bg-surface-3 text-brand"
					>
						<ServerStackIcon class="h-6 w-6" />
					</div>
					<div class="min-w-0">
						<div class="flex flex-wrap items-center gap-2">
							<h2 class="m-0 truncate text-lg font-semibold text-contrast">{{ server.name }}</h2>
							<span
								v-if="server.demo"
								class="rounded px-1.5 py-0.5 text-xs uppercase tracking-wide text-brand bg-brand/10"
							>
								{{ formatMessage(messages.demoBadge) }}
							</span>
						</div>
						<p class="m-0 mt-1 text-sm text-secondary">{{ server.description }}</p>
						<p class="m-0 mt-1 font-mono text-xs text-secondary">
							{{ server.address }}
							<span v-if="server.mcVersion">
								· {{ formatMessage(messages.version, { version: server.mcVersion }) }}
							</span>
						</p>
					</div>
				</div>
				<div class="flex flex-wrap gap-2 shrink-0">
					<Button class="!bg-button-bg" @click="copyAddress(server)">
						{{ copiedId === server.id ? '✓' : formatMessage(messages.copyAddress) }}
					</Button>
					<Button class="!bg-button-bg" :disabled="playingId === server.id" @click="openServerSettings(server)">
						<CogIcon class="h-4 w-4" />
						{{ formatMessage(messages.settings) }}
					</Button>
					<Button
						type="colored"
						color="brand"
						:disabled="playingId === server.id"
						@click="playServer(server)"
					>
						<PlayIcon class="h-4 w-4" />
						{{ playingId === server.id ? formatMessage(messages.playing) : formatMessage(messages.play) }}
					</Button>
				</div>
			</li>
		</ul>
	</div>
</template>
