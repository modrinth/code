<script setup lang="ts">
import { CogIcon, PlayIcon, ServerStackIcon } from '@modrinth/assets'
import {
	Button,
	Combobox,
	defineMessages,
	injectNotificationManager,
	useVIntl,
	type ComboboxOption,
} from '@modrinth/ui'
import { computed, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'

import {
	fetchOwyxCatalog,
	getOwyxClientKey,
	getOwyxDemoFlag,
	getOwyxLocalApiFallback,
	getStoredOwyxApiBase,
	isSafeExternalHttpsUrl,
	resolveOwyxPackUrl,
	sanitizeOwyxApiBase,
	setOwyxClientKey,
	setOwyxDemoFlag,
	setOwyxLocalApiFallback,
	setStoredOwyxApiBase,
	type OwyxServerEntry,
} from '@/helpers/owyx-api'
import { createOwyxCatalogServer, publishLibraryPackToCatalog } from '@/helpers/owyx-friends'
import { install_create_instance, installJobInstanceId } from '@/helpers/install'
import {
	export_instance_mrpack_bytes,
	get_pack_export_candidates,
	list as listInstances,
} from '@/helpers/instance'
import { get_game_versions } from '@/helpers/tags'
import type { GameInstance } from '@/helpers/types'
import { useRootBreadcrumb } from '@/providers/breadcrumbs'
import { injectOwyxSiteSession } from '@/providers/owyx-site-session'

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const router = useRouter()
const owyx = injectOwyxSiteSession()

type AdminTab = 'catalog' | 'api'
type GameVersionTag = { version: string; version_type: string }

const messages = defineMessages({
	title: { id: 'owyx.servers.title', defaultMessage: 'Owyx Servers' },
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
	tabCatalog: { id: 'owyx.servers.tab-catalog', defaultMessage: 'Catalog' },
	tabApi: { id: 'owyx.servers.tab-api', defaultMessage: 'API' },
	addServerTitle: {
		id: 'owyx.servers.add-server-title',
		defaultMessage: 'Add server to catalog',
	},
	serverName: { id: 'owyx.servers.server-name', defaultMessage: 'Name' },
	serverAddress: { id: 'owyx.servers.server-address', defaultMessage: 'Address / IP' },
	serverPort: { id: 'owyx.servers.server-port', defaultMessage: 'Port' },
	serverMc: { id: 'owyx.servers.server-mc', defaultMessage: 'Minecraft version' },
	serverMcSearch: {
		id: 'owyx.servers.server-mc-search',
		defaultMessage: 'Search game version…',
	},
	serverMcPlaceholder: {
		id: 'owyx.servers.server-mc-placeholder',
		defaultMessage: 'Select game version',
	},
	serverLoader: { id: 'owyx.servers.server-loader', defaultMessage: 'Loader' },
	serverLoaderSearch: {
		id: 'owyx.servers.server-loader-search',
		defaultMessage: 'Search loader…',
	},
	serverLoaderPlaceholder: {
		id: 'owyx.servers.server-loader-placeholder',
		defaultMessage: 'Select loader',
	},
	serverDesc: { id: 'owyx.servers.server-desc', defaultMessage: 'Notes (local)' },
	bindInstance: {
		id: 'owyx.servers.bind-instance',
		defaultMessage: 'Attach library instance (export → pack)',
	},
	bindInstanceSearch: {
		id: 'owyx.servers.bind-instance-search',
		defaultMessage: 'Search library instance…',
	},
	bindInstancePlaceholder: {
		id: 'owyx.servers.bind-instance-placeholder',
		defaultMessage: 'Select instance',
	},
	bindNone: { id: 'owyx.servers.bind-none', defaultMessage: '— no pack —' },
	createServer: { id: 'owyx.servers.create-server', defaultMessage: 'Publish server' },
	creating: { id: 'owyx.servers.creating', defaultMessage: 'Publishing…' },
	openAdmin: {
		id: 'owyx.servers.open-admin',
		defaultMessage: 'Open catalog on owyx.site',
	},
	playing: { id: 'owyx.servers.playing', defaultMessage: 'Preparing…' },
	downloadPack: { id: 'owyx.servers.download-pack', defaultMessage: 'Pack' },
	showSnapshots: {
		id: 'owyx.servers.show-snapshots',
		defaultMessage: 'Show all versions',
	},
	hideSnapshots: {
		id: 'owyx.servers.hide-snapshots',
		defaultMessage: 'Hide snapshots',
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
const adminTab = ref<AdminTab>('catalog')
const instances = ref<GameInstance[]>([])
const gameVersions = ref<GameVersionTag[]>([])
const showSnapshots = ref(false)
const busy = ref(false)
const statusMsg = ref('')

const formName = ref('')
const formAddress = ref('')
const formPort = ref('25565')
const formMc = ref<string | null>('1.21.1')
const formLoader = ref<string | null>('vanilla')
const formNotes = ref('')
const formInstanceId = ref<string | null>(null)

const LOADER_OPTIONS: ComboboxOption<string>[] = [
	{ value: 'vanilla', label: 'Vanilla' },
	{ value: 'fabric', label: 'Fabric' },
	{ value: 'forge', label: 'Forge' },
	{ value: 'neoforge', label: 'NeoForge' },
	{ value: 'quilt', label: 'Quilt' },
]

const gameVersionOptions = computed<ComboboxOption<string>[]>(() => {
	const versions = showSnapshots.value
		? gameVersions.value
		: gameVersions.value.filter((v) => v.version_type === 'release')
	return versions.map((v) => ({ value: v.version, label: v.version }))
})

const instanceOptions = computed<ComboboxOption<string | null>[]>(() => [
	{ value: null, label: formatMessage(messages.bindNone) },
	...instances.value.map((inst) => ({
		value: inst.id,
		label: `${inst.name} (${inst.game_version})`,
	})),
])

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
	} catch {
		instances.value = []
	}
}

async function loadGameVersions() {
	try {
		gameVersions.value = (await get_game_versions()) as GameVersionTag[]
	} catch {
		gameVersions.value = []
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

function mapLoader(loader?: string): 'vanilla' | 'fabric' | 'forge' | 'quilt' | 'neoforge' {
	const l = (loader || 'vanilla').toLowerCase()
	if (l === 'fabric' || l === 'forge' || l === 'quilt' || l === 'neoforge') return l
	return 'vanilla'
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

function openPack(server: OwyxServerEntry) {
	const url = resolveOwyxPackUrl(server.packUrl, sanitizeOwyxApiBase(apiBase.value))
	if (!url) return
	window.open(url, '_blank', 'noopener,noreferrer')
}

function openAdminCatalog() {
	window.open('https://owyx.site/admin', '_blank', 'noopener,noreferrer')
}

async function exportInstancePack(inst: GameInstance): Promise<{ blob: Blob; fileName: string }> {
	const candidates = await get_pack_export_candidates(inst.id)
	const included = candidates.filter((c) => c.defaultSelected).map((c) => c.path)
	const excluded = candidates.filter((c) => !c.defaultSelected).map((c) => c.path)
	const bytes = await export_instance_mrpack_bytes(
		inst.id,
		included,
		excluded,
		'1.0.0',
		formNotes.value || 'Published from Owyx launcher',
		inst.name,
	)
	const blob = new Blob([bytes], { type: 'application/zip' })
	const safeName = inst.name.replace(/[^\w.-]+/g, '_').slice(0, 48) || 'pack'
	return { blob, fileName: `${safeName}.mrpack` }
}

async function publishServer() {
	if (!formName.value.trim() || !formAddress.value.trim() || !formMc.value || !formLoader.value) {
		return
	}
	busy.value = true
	statusMsg.value = ''
	try {
		let packId: string | null = null
		const inst = instances.value.find((i) => i.id === formInstanceId.value)
		if (inst) {
			const { blob, fileName } = await exportInstancePack(inst)
			const published = await publishLibraryPackToCatalog({
				name: `${formName.value.trim()} pack`,
				minecraft: formMc.value.trim() || inst.game_version || '1.21.1',
				loader: formLoader.value || String(inst.loader || 'vanilla').toLowerCase(),
				description: formNotes.value || `From library: ${inst.name}`,
				file: blob,
				fileName,
			})
			packId = published.packId
		}
		const server = await createOwyxCatalogServer({
			name: formName.value.trim(),
			address: formAddress.value.trim(),
			port: parseInt(formPort.value, 10) || 25565,
			minecraft: formMc.value.trim() || '1.21.1',
			loader: formLoader.value || 'vanilla',
			packId,
			published: true,
		})
		statusMsg.value = `Published ${server.id}${packId ? ` + pack ${packId}` : ''}`
		formName.value = ''
		formAddress.value = ''
		formNotes.value = ''
		await loadCatalog()
	} catch (e) {
		handleError(e)
		statusMsg.value = e instanceof Error ? e.message : String(e)
	} finally {
		busy.value = false
	}
}

watch(adminOpen, (open) => {
	if (open && isAdmin.value) {
		void loadInstances()
		void loadGameVersions()
	}
})

watch(formInstanceId, (id) => {
	const inst = instances.value.find((i) => i.id === id)
	if (!inst) return
	if (inst.game_version) formMc.value = inst.game_version
	if (inst.loader) formLoader.value = String(inst.loader).toLowerCase()
})

onMounted(() => {
	void loadCatalog()
})
</script>

<template>
	<div class="owyx-servers mx-auto flex w-full max-w-4xl flex-col gap-6 px-4 py-6">
		<header class="flex flex-col gap-2 sm:flex-row sm:items-end sm:justify-between">
			<div>
				<h1 class="m-0 text-2xl font-semibold text-contrast">{{ formatMessage(messages.title) }}</h1>
				<p class="m-0 text-secondary">{{ formatMessage(messages.subtitle) }}</p>
			</div>
			<Button class="!bg-button-bg" :disabled="loading" @click="loadCatalog">
				{{ formatMessage(messages.refresh) }}
			</Button>
		</header>

		<section
			v-if="isAdmin"
			class="flex flex-col gap-3 rounded-xl border border-solid border-surface-5 bg-surface-2 p-4"
		>
			<button
				type="button"
				class="m-0 flex w-full cursor-pointer items-center justify-between border-none bg-transparent p-0 text-left"
				@click="adminOpen = !adminOpen"
			>
				<span class="text-base font-semibold text-contrast">{{ formatMessage(messages.adminTools) }}</span>
				<span class="text-sm text-secondary">{{ adminOpen ? '▾' : '▸' }}</span>
			</button>

			<template v-if="adminOpen">
				<div class="flex flex-wrap gap-2">
					<button
						type="button"
						class="rounded-lg px-3 py-1.5 text-sm"
						:class="
							adminTab === 'catalog'
								? 'bg-brand/20 text-brand'
								: 'bg-surface-3 text-secondary'
						"
						@click="adminTab = 'catalog'"
					>
						{{ formatMessage(messages.tabCatalog) }}
					</button>
					<button
						type="button"
						class="rounded-lg px-3 py-1.5 text-sm"
						:class="adminTab === 'api' ? 'bg-brand/20 text-brand' : 'bg-surface-3 text-secondary'"
						@click="adminTab = 'api'"
					>
						{{ formatMessage(messages.tabApi) }}
					</button>
				</div>

				<div
					v-if="adminTab === 'catalog'"
					class="flex flex-col gap-3 rounded-lg border border-solid border-surface-5 bg-surface-3 p-3"
				>
					<p class="m-0 text-sm font-medium text-contrast">
						{{ formatMessage(messages.addServerTitle) }}
					</p>
					<div class="grid gap-2 sm:grid-cols-2">
						<label class="flex flex-col gap-1 text-sm">
							<span class="text-secondary">{{ formatMessage(messages.serverName) }}</span>
							<input
								v-model="formName"
								class="rounded-lg border border-solid border-surface-5 bg-surface-2 px-3 py-2 text-primary"
							/>
						</label>
						<label class="flex flex-col gap-1 text-sm">
							<span class="text-secondary">{{ formatMessage(messages.serverAddress) }}</span>
							<input
								v-model="formAddress"
								placeholder="play.example.com"
								class="rounded-lg border border-solid border-surface-5 bg-surface-2 px-3 py-2 text-primary"
							/>
						</label>
						<label class="flex flex-col gap-1 text-sm">
							<span class="text-secondary">{{ formatMessage(messages.serverPort) }}</span>
							<input
								v-model="formPort"
								type="number"
								class="rounded-lg border border-solid border-surface-5 bg-surface-2 px-3 py-2 text-primary"
							/>
						</label>
						<label class="flex flex-col gap-1 text-sm">
							<span class="text-secondary">{{ formatMessage(messages.serverMc) }}</span>
							<Combobox
								v-model="formMc"
								:options="gameVersionOptions"
								searchable
								sync-with-selection
								:placeholder="formatMessage(messages.serverMcPlaceholder)"
								:search-placeholder="formatMessage(messages.serverMcSearch)"
							>
								<template #dropdown-footer>
									<button
										class="flex w-full cursor-pointer items-center justify-center border-0 border-t border-solid border-surface-5 bg-transparent py-2 text-sm font-semibold text-secondary hover:text-contrast"
										@mousedown.prevent
										@click="showSnapshots = !showSnapshots"
									>
										{{
											showSnapshots
												? formatMessage(messages.hideSnapshots)
												: formatMessage(messages.showSnapshots)
										}}
									</button>
								</template>
							</Combobox>
						</label>
						<label class="flex flex-col gap-1 text-sm">
							<span class="text-secondary">{{ formatMessage(messages.serverLoader) }}</span>
							<Combobox
								v-model="formLoader"
								:options="LOADER_OPTIONS"
								searchable
								sync-with-selection
								:placeholder="formatMessage(messages.serverLoaderPlaceholder)"
								:search-placeholder="formatMessage(messages.serverLoaderSearch)"
							/>
						</label>
						<label class="flex flex-col gap-1 text-sm sm:col-span-2">
							<span class="text-secondary">{{ formatMessage(messages.bindInstance) }}</span>
							<Combobox
								v-model="formInstanceId"
								:options="instanceOptions"
								searchable
								sync-with-selection
								:placeholder="formatMessage(messages.bindInstancePlaceholder)"
								:search-placeholder="formatMessage(messages.bindInstanceSearch)"
							/>
						</label>
						<label class="flex flex-col gap-1 text-sm sm:col-span-2">
							<span class="text-secondary">{{ formatMessage(messages.serverDesc) }}</span>
							<input
								v-model="formNotes"
								class="rounded-lg border border-solid border-surface-5 bg-surface-2 px-3 py-2 text-primary"
							/>
						</label>
					</div>
					<div class="flex flex-wrap gap-2">
						<Button
							type="colored"
							color="brand"
							:disabled="busy || !formName || !formAddress || !formMc || !formLoader"
							@click="publishServer"
						>
							{{ busy ? formatMessage(messages.creating) : formatMessage(messages.createServer) }}
						</Button>
						<Button class="!bg-button-bg" @click="openAdminCatalog">
							{{ formatMessage(messages.openAdmin) }}
						</Button>
					</div>
					<p v-if="statusMsg" class="m-0 text-sm text-secondary">{{ statusMsg }}</p>
				</div>

				<div v-else class="flex flex-col gap-3">
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
					<Button type="colored" color="brand" @click="saveSettings">
						{{ formatMessage(messages.saveSettings) }}
					</Button>
					<p v-if="apiError" class="m-0 text-sm text-orange">{{ formatMessage(messages.unreachable) }}</p>
				</div>
			</template>
		</section>

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
						:src="
							server.iconUrl.startsWith('/')
								? `${sanitizeOwyxApiBase(apiBase)}${server.iconUrl}`
								: server.iconUrl
						"
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
								class="rounded bg-brand/10 px-1.5 py-0.5 text-xs uppercase tracking-wide text-brand"
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
				<div class="flex shrink-0 flex-wrap gap-2">
					<Button class="!bg-button-bg" @click="copyAddress(server)">
						{{ copiedId === server.id ? '✓' : formatMessage(messages.copyAddress) }}
					</Button>
					<Button
						v-if="resolveOwyxPackUrl(server.packUrl, sanitizeOwyxApiBase(apiBase))"
						class="!bg-button-bg"
						@click="openPack(server)"
					>
						{{ formatMessage(messages.downloadPack) }}
					</Button>
					<Button
						class="!bg-button-bg"
						:disabled="playingId === server.id"
						@click="openServerSettings(server)"
					>
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
