<script setup lang="ts">
import { PlayIcon, ServerStackIcon } from '@modrinth/assets'
import { Button, defineMessages, injectNotificationManager, useVIntl } from '@modrinth/ui'
import { computed, onMounted, ref } from 'vue'

import { useRootBreadcrumb } from '@/providers/breadcrumbs'
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

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()

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
		defaultMessage: 'No servers available right now. Check your API settings or try again later.',
	},
	unreachable: {
		id: 'owyx.servers.unreachable',
		defaultMessage: 'Could not reach the Owyx API. Showing demo entry if enabled.',
	},
	play: {
		id: 'owyx.servers.play',
		defaultMessage: 'Play / Join',
	},
	downloadPack: {
		id: 'owyx.servers.download-pack',
		defaultMessage: 'Download pack',
	},
	copyAddress: {
		id: 'owyx.servers.copy-address',
		defaultMessage: 'Copy address',
	},
	apiBase: {
		id: 'owyx.servers.api-base',
		defaultMessage: 'API base URL',
	},
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
	saveSettings: {
		id: 'owyx.servers.save-settings',
		defaultMessage: 'Save API settings',
	},
	refresh: {
		id: 'owyx.servers.refresh',
		defaultMessage: 'Refresh',
	},
	version: {
		id: 'owyx.servers.version',
		defaultMessage: 'MC {version}',
	},
	demoBadge: {
		id: 'owyx.servers.demo-badge',
		defaultMessage: 'demo',
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
const apiError = ref(false)
const apiBase = ref(getStoredOwyxApiBase())
const clientKey = ref(getOwyxClientKey())
const demoEnabled = ref(getOwyxDemoFlag())
const localFallback = ref(getOwyxLocalApiFallback())
const copiedId = ref<string | null>(null)

const hasServers = computed(() => servers.value.length > 0)

async function loadCatalog() {
	loading.value = true
	apiError.value = false
	try {
		const result = await fetchOwyxCatalog({
			baseUrl: sanitizeOwyxApiBase(apiBase.value),
			clientKey: clientKey.value,
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

function openPack(server: OwyxServerEntry) {
	if (!isSafeExternalHttpsUrl(server.packUrl)) return
	window.open(server.packUrl!, '_blank', 'noopener,noreferrer')
}

onMounted(() => {
	void loadCatalog()
})
</script>

<template>
	<div class="owyx-servers mx-auto flex w-full max-w-4xl flex-col gap-6 px-4 py-6">
		<header class="flex flex-col gap-2">
			<h1 class="m-0 text-2xl font-semibold text-contrast">{{ formatMessage(messages.title) }}</h1>
			<p class="m-0 text-secondary">{{ formatMessage(messages.subtitle) }}</p>
		</header>

		<section class="flex flex-col gap-3 rounded-xl border border-solid border-surface-5 bg-surface-2 p-4">
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
			</div>
			<p v-if="apiError" class="m-0 text-sm text-orange">{{ formatMessage(messages.unreachable) }}</p>
		</section>

		<section v-if="loading" class="text-secondary">…</section>

		<section v-else-if="!hasServers" class="rounded-xl border border-dashed border-surface-5 bg-surface-2 p-8 text-center text-secondary">
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
						:src="server.iconUrl"
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
						{{
							copiedId === server.id
								? '✓'
								: formatMessage(messages.copyAddress)
						}}
					</Button>
					<Button
						v-if="isSafeExternalHttpsUrl(server.packUrl)"
						class="!bg-button-bg"
						@click="openPack(server)"
					>
						{{ formatMessage(messages.downloadPack) }}
					</Button>
					<Button type="colored" color="brand" @click="copyAddress(server)">
						<PlayIcon class="h-4 w-4" />
						{{ formatMessage(messages.play) }}
					</Button>
				</div>
			</li>
		</ul>
	</div>
</template>
