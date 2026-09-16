<script setup lang="ts">
import { NewspaperIcon, ServerStackIcon, UserIcon } from '@modrinth/assets'
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
	adminBanUser,
	adminCreateNews,
	adminCreatePack,
	adminDeleteNews,
	adminDeletePack,
	adminDeleteServer,
	adminListNews,
	adminListPacks,
	adminListServers,
	adminListUsers,
	adminSetNewsPublished,
	adminSetPackPublished,
	adminSetServerPublished,
	adminSetUserRole,
	type AdminNews,
	type AdminPack,
	type AdminServer,
	type AdminUser,
} from '@/helpers/owyx-admin-api'
import {
	getOwyxClientKey,
	getOwyxDemoFlag,
	getOwyxLocalApiFallback,
	getStoredOwyxApiBase,
	sanitizeOwyxApiBase,
	setOwyxClientKey,
	setOwyxDemoFlag,
	setOwyxLocalApiFallback,
	setStoredOwyxApiBase,
} from '@/helpers/owyx-api'
import { createOwyxCatalogServer, publishLibraryPackToCatalog } from '@/helpers/owyx-friends'
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

type AdminTab = 'publish' | 'catalog' | 'users' | 'news' | 'api'
type CatalogSub = 'servers' | 'packs'
type GameVersionTag = { version: string; version_type: string }

const ROLES = ['user', 'helper', 'moderator', 'admin'] as const

const messages = defineMessages({
	title: { id: 'owyx.admin.title', defaultMessage: 'Admin panel' },
	subtitle: {
		id: 'owyx.admin.subtitle',
		defaultMessage: 'Manage catalog, users, news, and publish packs from your library.',
	},
	tabPublish: { id: 'owyx.admin.tab-publish', defaultMessage: 'Publish' },
	tabCatalog: { id: 'owyx.admin.tab-catalog', defaultMessage: 'Catalog' },
	tabUsers: { id: 'owyx.admin.tab-users', defaultMessage: 'Users' },
	tabNews: { id: 'owyx.admin.tab-news', defaultMessage: 'News' },
	tabApi: { id: 'owyx.admin.tab-api', defaultMessage: 'API' },
	refresh: { id: 'owyx.servers.refresh', defaultMessage: 'Refresh' },
	addServerTitle: {
		id: 'owyx.servers.add-server-title',
		defaultMessage: 'Publish server + pack from library',
	},
	serverName: { id: 'owyx.servers.server-name', defaultMessage: 'Name' },
	serverAddress: { id: 'owyx.servers.server-address', defaultMessage: 'Address / IP' },
	serverPort: { id: 'owyx.servers.server-port', defaultMessage: 'Port' },
	serverMc: { id: 'owyx.servers.server-mc', defaultMessage: 'Minecraft version' },
	serverMcSearch: { id: 'owyx.servers.server-mc-search', defaultMessage: 'Search game version…' },
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
	openSiteAdmin: { id: 'owyx.admin.open-site', defaultMessage: 'Open full site admin' },
	openServersCatalog: {
		id: 'owyx.admin.open-servers',
		defaultMessage: 'Player servers catalog',
	},
	publishExport: { id: 'owyx.admin.publish-export', defaultMessage: 'Exporting pack…' },
	publishUpload: {
		id: 'owyx.admin.publish-upload',
		defaultMessage: 'Uploading pack ({size} MB)…',
	},
	publishCreate: {
		id: 'owyx.admin.publish-create',
		defaultMessage: 'Creating server entry…',
	},
	showSnapshots: { id: 'owyx.servers.show-snapshots', defaultMessage: 'Show all versions' },
	hideSnapshots: { id: 'owyx.servers.hide-snapshots', defaultMessage: 'Hide snapshots' },
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
	servers: { id: 'owyx.admin.servers', defaultMessage: 'Servers' },
	packs: { id: 'owyx.admin.packs', defaultMessage: 'Packs' },
	published: { id: 'owyx.admin.published', defaultMessage: 'Published' },
	draft: { id: 'owyx.admin.draft', defaultMessage: 'Draft' },
	delete: { id: 'owyx.admin.delete', defaultMessage: 'Delete' },
	searchUsers: { id: 'owyx.admin.search-users', defaultMessage: 'Search nickname or email…' },
	newsTitle: { id: 'owyx.admin.news-title', defaultMessage: 'Title' },
	newsSummary: { id: 'owyx.admin.news-summary', defaultMessage: 'Summary' },
	newsTag: { id: 'owyx.admin.news-tag', defaultMessage: 'Tag' },
	createNews: { id: 'owyx.admin.create-news', defaultMessage: 'Create news' },
	createPackUrl: { id: 'owyx.admin.create-pack-url', defaultMessage: 'Add pack by URL' },
	packUrl: { id: 'owyx.admin.pack-url', defaultMessage: 'Archive URL' },
	forbidden: {
		id: 'owyx.admin.forbidden',
		defaultMessage: 'Admin role required.',
	},
})

useRootBreadcrumb({
	slot: 'root',
	id: 'owyx-admin',
	label: formatMessage(messages.title),
	to: '/owyx-admin',
	visual: { type: 'icon', component: ServerStackIcon },
})

const isAdmin = computed(() => owyx.isAdmin.value)
const adminTab = ref<AdminTab>('publish')
const catalogSub = ref<CatalogSub>('servers')
const busy = ref(false)
const statusMsg = ref('')
const loading = ref(false)

const servers = ref<AdminServer[]>([])
const packs = ref<AdminPack[]>([])
const users = ref<AdminUser[]>([])
const news = ref<AdminNews[]>([])
const userSearch = ref('')
const instances = ref<GameInstance[]>([])
const gameVersions = ref<GameVersionTag[]>([])
const showSnapshots = ref(false)

const formName = ref('')
const formAddress = ref('')
const formPort = ref('25565')
const formMc = ref<string | null>('1.21.1')
const formLoader = ref<string | null>('vanilla')
const formNotes = ref('')
const formInstanceId = ref<string | null>(null)

const packName = ref('')
const packMc = ref('1.21.1')
const packLoader = ref('neoforge')
const packUrl = ref('')

const newsTitle = ref('')
const newsTag = ref('News')
const newsSummary = ref('')

const apiBase = ref(getStoredOwyxApiBase())
const clientKey = ref(getOwyxClientKey())
const demoEnabled = ref(getOwyxDemoFlag())
const localFallback = ref(getOwyxLocalApiFallback())

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

async function loadCatalogAdmin() {
	loading.value = true
	try {
		const [s, p] = await Promise.all([adminListServers(), adminListPacks()])
		servers.value = s
		packs.value = p
	} catch (e) {
		handleError(e)
	} finally {
		loading.value = false
	}
}

async function loadUsers() {
	loading.value = true
	try {
		users.value = await adminListUsers(userSearch.value)
	} catch (e) {
		handleError(e)
	} finally {
		loading.value = false
	}
}

async function loadNews() {
	loading.value = true
	try {
		news.value = await adminListNews()
	} catch (e) {
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

async function refreshCurrent() {
	if (adminTab.value === 'catalog' || adminTab.value === 'publish') await loadCatalogAdmin()
	if (adminTab.value === 'users') await loadUsers()
	if (adminTab.value === 'news') await loadNews()
}

watch(adminTab, (tab) => {
	void refreshCurrent()
	if (tab === 'publish') {
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
			statusMsg.value = formatMessage(messages.publishExport)
			const { blob, fileName } = await exportInstancePack(inst)
			const sizeMb = Math.max(1, Math.round(blob.size / (1024 * 1024)))
			statusMsg.value = formatMessage(messages.publishUpload, { size: sizeMb })
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
		statusMsg.value = formatMessage(messages.publishCreate)
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
		formInstanceId.value = null
		await loadCatalogAdmin()
	} catch (e) {
		handleError(e)
		statusMsg.value = e instanceof Error ? e.message : String(e)
	} finally {
		busy.value = false
	}
}

async function createPackFromUrl() {
	if (!packName.value.trim() || !packUrl.value.trim()) return
	busy.value = true
	try {
		await adminCreatePack({
			name: packName.value.trim(),
			minecraft: packMc.value.trim() || '1.21.1',
			loader: packLoader.value || 'vanilla',
			sourceType: 'http_zip',
			sourceConfig: { url: packUrl.value.trim() },
			published: true,
		})
		packName.value = ''
		packUrl.value = ''
		await loadCatalogAdmin()
	} catch (e) {
		handleError(e)
	} finally {
		busy.value = false
	}
}

async function createNewsItem() {
	if (!newsTitle.value.trim()) return
	busy.value = true
	try {
		await adminCreateNews({
			title: newsTitle.value.trim(),
			tag: newsTag.value.trim() || 'News',
			summary: newsSummary.value.trim(),
			published: true,
		})
		newsTitle.value = ''
		newsSummary.value = ''
		await loadNews()
	} catch (e) {
		handleError(e)
	} finally {
		busy.value = false
	}
}

function saveSettings() {
	setStoredOwyxApiBase(sanitizeOwyxApiBase(apiBase.value))
	apiBase.value = getStoredOwyxApiBase()
	setOwyxClientKey(clientKey.value.trim())
	setOwyxDemoFlag(demoEnabled.value)
	setOwyxLocalApiFallback(localFallback.value)
	statusMsg.value = 'Saved'
}

function openSiteAdmin() {
	window.open('https://owyx.site/admin', '_blank', 'noopener,noreferrer')
}

onMounted(() => {
	if (!isAdmin.value) {
		void router.replace('/owyx-servers')
		return
	}
	void loadInstances()
	void loadGameVersions()
	void loadCatalogAdmin()
})
</script>

<template>
	<div class="owyx-admin mx-auto flex w-full max-w-5xl flex-col gap-5 px-4 py-6">
		<header class="flex flex-col gap-2 sm:flex-row sm:items-end sm:justify-between">
			<div>
				<h1 class="m-0 text-2xl font-semibold text-contrast">{{ formatMessage(messages.title) }}</h1>
				<p class="m-0 text-secondary">{{ formatMessage(messages.subtitle) }}</p>
			</div>
			<div class="flex flex-wrap gap-2">
				<Button class="!bg-button-bg" :disabled="loading" @click="refreshCurrent">
					{{ formatMessage(messages.refresh) }}
				</Button>
				<Button class="!bg-button-bg" @click="router.push('/owyx-servers')">
					{{ formatMessage(messages.openServersCatalog) }}
				</Button>
				<Button type="colored" color="brand" @click="openSiteAdmin">
					{{ formatMessage(messages.openSiteAdmin) }}
				</Button>
			</div>
		</header>

		<p v-if="!isAdmin" class="m-0 text-orange">{{ formatMessage(messages.forbidden) }}</p>

		<template v-else>
			<nav class="flex flex-wrap gap-2">
				<button
					v-for="tab in [
						{ id: 'publish' as const, label: messages.tabPublish },
						{ id: 'catalog' as const, label: messages.tabCatalog },
						{ id: 'users' as const, label: messages.tabUsers },
						{ id: 'news' as const, label: messages.tabNews },
						{ id: 'api' as const, label: messages.tabApi },
					]"
					:key="tab.id"
					type="button"
					class="rounded-lg px-3 py-1.5 text-sm"
					:class="adminTab === tab.id ? 'bg-brand/20 text-brand' : 'bg-surface-3 text-secondary'"
					@click="adminTab = tab.id"
				>
					{{ formatMessage(tab.label) }}
				</button>
			</nav>

			<!-- Publish -->
			<section
				v-if="adminTab === 'publish'"
				class="flex flex-col gap-3 rounded-xl border border-solid border-surface-5 bg-surface-2 p-4"
			>
				<p class="m-0 text-sm font-medium text-contrast">
					{{ formatMessage(messages.addServerTitle) }}
				</p>
				<div class="grid gap-2 sm:grid-cols-2">
					<label class="flex flex-col gap-1 text-sm">
						<span class="text-secondary">{{ formatMessage(messages.serverName) }}</span>
						<input
							v-model="formName"
							class="rounded-lg border border-solid border-surface-5 bg-surface-3 px-3 py-2 text-primary"
						/>
					</label>
					<label class="flex flex-col gap-1 text-sm">
						<span class="text-secondary">{{ formatMessage(messages.serverAddress) }}</span>
						<input
							v-model="formAddress"
							placeholder="play.example.com"
							class="rounded-lg border border-solid border-surface-5 bg-surface-3 px-3 py-2 text-primary"
						/>
					</label>
					<label class="flex flex-col gap-1 text-sm">
						<span class="text-secondary">{{ formatMessage(messages.serverPort) }}</span>
						<input
							v-model="formPort"
							type="number"
							class="rounded-lg border border-solid border-surface-5 bg-surface-3 px-3 py-2 text-primary"
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
							class="rounded-lg border border-solid border-surface-5 bg-surface-3 px-3 py-2 text-primary"
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
						{{ busy ? statusMsg || formatMessage(messages.creating) : formatMessage(messages.createServer) }}
					</Button>
				</div>
				<p v-if="statusMsg" class="m-0 text-sm text-secondary">{{ statusMsg }}</p>
			</section>

			<!-- Catalog -->
			<section
				v-else-if="adminTab === 'catalog'"
				class="flex flex-col gap-4 rounded-xl border border-solid border-surface-5 bg-surface-2 p-4"
			>
				<div class="flex flex-wrap gap-2">
					<button
						type="button"
						class="rounded-lg px-3 py-1.5 text-sm"
						:class="
							catalogSub === 'servers' ? 'bg-brand/20 text-brand' : 'bg-surface-3 text-secondary'
						"
						@click="catalogSub = 'servers'"
					>
						{{ formatMessage(messages.servers) }} ({{ servers.length }})
					</button>
					<button
						type="button"
						class="rounded-lg px-3 py-1.5 text-sm"
						:class="catalogSub === 'packs' ? 'bg-brand/20 text-brand' : 'bg-surface-3 text-secondary'"
						@click="catalogSub = 'packs'"
					>
						{{ formatMessage(messages.packs) }} ({{ packs.length }})
					</button>
				</div>

				<ul v-if="catalogSub === 'servers'" class="m-0 flex list-none flex-col gap-2 p-0">
					<li
						v-for="s in servers"
						:key="s.id"
						class="flex flex-wrap items-center justify-between gap-2 rounded-lg border border-solid border-surface-5 bg-surface-3 px-3 py-2"
					>
						<div class="min-w-0">
							<p class="m-0 truncate font-medium text-contrast">{{ s.name }}</p>
							<p class="m-0 font-mono text-xs text-secondary">
								{{ s.address }}:{{ s.port }} · {{ s.minecraft }} / {{ s.loader }}
							</p>
						</div>
						<div class="flex flex-wrap gap-2">
							<Button
								class="!bg-button-bg"
								@click="
									adminSetServerPublished(s.id, !s.published)
										.then(loadCatalogAdmin)
										.catch(handleError)
								"
							>
								{{ s.published ? formatMessage(messages.published) : formatMessage(messages.draft) }}
							</Button>
							<Button
								class="!bg-button-bg !text-red"
								@click="adminDeleteServer(s.id).then(loadCatalogAdmin).catch(handleError)"
							>
								{{ formatMessage(messages.delete) }}
							</Button>
						</div>
					</li>
					<li v-if="servers.length === 0" class="text-sm text-secondary">No servers yet.</li>
				</ul>

				<template v-else>
					<div
						class="grid gap-2 rounded-lg border border-solid border-surface-5 bg-surface-3 p-3 sm:grid-cols-2"
					>
						<p class="m-0 text-sm font-medium text-contrast sm:col-span-2">
							{{ formatMessage(messages.createPackUrl) }}
						</p>
						<input
							v-model="packName"
							:placeholder="formatMessage(messages.serverName)"
							class="rounded-lg border border-solid border-surface-5 bg-surface-2 px-3 py-2 text-sm text-primary"
						/>
						<input
							v-model="packUrl"
							:placeholder="formatMessage(messages.packUrl)"
							class="rounded-lg border border-solid border-surface-5 bg-surface-2 px-3 py-2 text-sm text-primary"
						/>
						<input
							v-model="packMc"
							placeholder="1.21.1"
							class="rounded-lg border border-solid border-surface-5 bg-surface-2 px-3 py-2 text-sm text-primary"
						/>
						<select
							v-model="packLoader"
							class="rounded-lg border border-solid border-surface-5 bg-surface-2 px-3 py-2 text-sm text-primary"
						>
							<option v-for="l in LOADER_OPTIONS" :key="l.value" :value="l.value">
								{{ l.label }}
							</option>
						</select>
						<Button
							type="colored"
							color="brand"
							class="sm:col-span-2"
							:disabled="busy || !packName || !packUrl"
							@click="createPackFromUrl"
						>
							{{ formatMessage(messages.createPackUrl) }}
						</Button>
					</div>
					<ul class="m-0 flex list-none flex-col gap-2 p-0">
						<li
							v-for="p in packs"
							:key="p.id"
							class="flex flex-wrap items-center justify-between gap-2 rounded-lg border border-solid border-surface-5 bg-surface-3 px-3 py-2"
						>
							<div class="min-w-0">
								<p class="m-0 truncate font-medium text-contrast">{{ p.name }}</p>
								<p class="m-0 text-xs text-secondary">
									{{ p.minecraft }} / {{ p.loader }} · {{ p.sourceType || '—' }}
								</p>
							</div>
							<div class="flex flex-wrap gap-2">
								<Button
									class="!bg-button-bg"
									@click="
										adminSetPackPublished(p.id, !p.published)
											.then(loadCatalogAdmin)
											.catch(handleError)
									"
								>
									{{
										p.published ? formatMessage(messages.published) : formatMessage(messages.draft)
									}}
								</Button>
								<Button
									class="!bg-button-bg !text-red"
									@click="adminDeletePack(p.id).then(loadCatalogAdmin).catch(handleError)"
								>
									{{ formatMessage(messages.delete) }}
								</Button>
							</div>
						</li>
						<li v-if="packs.length === 0" class="text-sm text-secondary">No packs yet.</li>
					</ul>
				</template>
			</section>

			<!-- Users -->
			<section
				v-else-if="adminTab === 'users'"
				class="flex flex-col gap-3 rounded-xl border border-solid border-surface-5 bg-surface-2 p-4"
			>
				<div class="flex flex-wrap gap-2">
					<input
						v-model="userSearch"
						:placeholder="formatMessage(messages.searchUsers)"
						class="min-w-[12rem] flex-1 rounded-lg border border-solid border-surface-5 bg-surface-3 px-3 py-2 text-sm text-primary"
						@keyup.enter="loadUsers"
					/>
					<Button type="colored" color="brand" @click="loadUsers">
						{{ formatMessage(messages.refresh) }}
					</Button>
				</div>
				<ul class="m-0 flex list-none flex-col gap-2 p-0">
					<li
						v-for="u in users"
						:key="u.id"
						class="flex flex-wrap items-center justify-between gap-2 rounded-lg border border-solid border-surface-5 bg-surface-3 px-3 py-2"
					>
						<div class="flex min-w-0 items-center gap-2">
							<UserIcon class="h-4 w-4 shrink-0 text-secondary" />
							<div class="min-w-0">
								<p class="m-0 truncate font-medium text-contrast">
									{{ u.nickname || '—' }}
									<span
										v-if="u.status === 'banned' || u.is_banned"
										class="ml-2 rounded bg-red/20 px-1.5 py-0.5 text-xs text-red"
										>banned</span
									>
								</p>
								<p class="m-0 truncate text-xs text-secondary">{{ u.email }}</p>
							</div>
						</div>
						<div class="flex flex-wrap items-center gap-2">
							<select
								class="rounded-lg border border-solid border-surface-5 bg-surface-2 px-2 py-1.5 text-sm text-primary"
								:value="u.role"
								@change="
									adminSetUserRole(u.id, ($event.target as HTMLSelectElement).value)
										.then(loadUsers)
										.catch(handleError)
								"
							>
								<option v-for="r in ROLES" :key="r" :value="r">{{ r }}</option>
							</select>
							<Button
								class="!bg-button-bg"
								@click="
									adminBanUser(u.id, !(u.status === 'banned' || u.is_banned))
										.then(loadUsers)
										.catch(handleError)
								"
							>
								{{ u.status === 'banned' || u.is_banned ? 'Unban' : 'Ban' }}
							</Button>
						</div>
					</li>
					<li v-if="!loading && users.length === 0" class="text-sm text-secondary">No users.</li>
				</ul>
			</section>

			<!-- News -->
			<section
				v-else-if="adminTab === 'news'"
				class="flex flex-col gap-3 rounded-xl border border-solid border-surface-5 bg-surface-2 p-4"
			>
				<div class="grid gap-2 rounded-lg border border-solid border-surface-5 bg-surface-3 p-3 sm:grid-cols-2">
					<label class="flex flex-col gap-1 text-sm sm:col-span-2">
						<span class="text-secondary">{{ formatMessage(messages.newsTitle) }}</span>
						<input
							v-model="newsTitle"
							class="rounded-lg border border-solid border-surface-5 bg-surface-2 px-3 py-2 text-primary"
						/>
					</label>
					<label class="flex flex-col gap-1 text-sm">
						<span class="text-secondary">{{ formatMessage(messages.newsTag) }}</span>
						<input
							v-model="newsTag"
							class="rounded-lg border border-solid border-surface-5 bg-surface-2 px-3 py-2 text-primary"
						/>
					</label>
					<label class="flex flex-col gap-1 text-sm sm:col-span-2">
						<span class="text-secondary">{{ formatMessage(messages.newsSummary) }}</span>
						<textarea
							v-model="newsSummary"
							rows="3"
							class="rounded-lg border border-solid border-surface-5 bg-surface-2 px-3 py-2 text-primary"
						/>
					</label>
					<Button
						type="colored"
						color="brand"
						class="sm:col-span-2"
						:disabled="busy || !newsTitle"
						@click="createNewsItem"
					>
						<NewspaperIcon class="h-4 w-4" />
						{{ formatMessage(messages.createNews) }}
					</Button>
				</div>
				<ul class="m-0 flex list-none flex-col gap-2 p-0">
					<li
						v-for="n in news"
						:key="String(n.id)"
						class="flex flex-wrap items-center justify-between gap-2 rounded-lg border border-solid border-surface-5 bg-surface-3 px-3 py-2"
					>
						<div class="min-w-0">
							<p class="m-0 truncate font-medium text-contrast">{{ n.title }}</p>
							<p class="m-0 text-xs text-secondary">{{ n.tag }} · {{ n.summary }}</p>
						</div>
						<div class="flex flex-wrap gap-2">
							<Button
								class="!bg-button-bg"
								@click="
									adminSetNewsPublished(n.id, !n.published).then(loadNews).catch(handleError)
								"
							>
								{{ n.published ? formatMessage(messages.published) : formatMessage(messages.draft) }}
							</Button>
							<Button
								class="!bg-button-bg !text-red"
								@click="adminDeleteNews(n.id).then(loadNews).catch(handleError)"
							>
								{{ formatMessage(messages.delete) }}
							</Button>
						</div>
					</li>
					<li v-if="!loading && news.length === 0" class="text-sm text-secondary">No news.</li>
				</ul>
			</section>

			<!-- API -->
			<section
				v-else
				class="flex flex-col gap-3 rounded-xl border border-solid border-surface-5 bg-surface-2 p-4"
			>
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
				<p v-if="statusMsg" class="m-0 text-sm text-secondary">{{ statusMsg }}</p>
			</section>
		</template>
	</div>
</template>
