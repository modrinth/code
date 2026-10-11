<script setup lang="ts">
import { type Archon, ModrinthApiError } from '@modrinth/api-client'
import { PlusIcon, SearchIcon, SettingsIcon, UpdatedIcon } from '@modrinth/assets'
import {
	Button,
	type CreationFlowContextValue,
	CreationFlowModal,
	EmptyState,
	IconButton,
	injectInstanceServerSource,
	injectModrinthClient,
	injectNotificationManager,
	Input,
	ServerListing,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, inject, ref } from 'vue'
import { useRouter } from 'vue-router'

import { appSettingsModalOpenCoresKey } from '@/providers/app-settings-modal'

const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const instanceSource = injectInstanceServerSource(null)
const openCoreSettings = inject(appSettingsModalOpenCoresKey, () => {})
const router = useRouter()
const queryClient = useQueryClient()
const searchInput = ref('')
const setupModal = ref<InstanceType<typeof CreationFlowModal>>()
const { data, error, isLoading, isFetching, refetch } = useQuery({
	queryKey: ['servers'],
	queryFn: () => client.archon.servers_v0.list({ limit: 100 }),
	staleTime: 30_000,
})
const allServers = computed(() => data.value?.servers ?? [])
const servers = computed(() =>
	allServers.value.filter((server) =>
		server.name.toLowerCase().includes(searchInput.value.toLowerCase()),
	),
)

// Never show the raw error. A Core answers 403 to anyone but its owner; anything else means
// it could not be reached.
const problem = computed(() => {
	if (!error.value) return null
	if (error.value instanceof ModrinthApiError && error.value.statusCode === 403) {
		return {
			type: 'error' as const,
			heading: 'This Core belongs to someone else',
			description: 'Only the owner of a Core can see and manage its servers.',
			retry: false,
		}
	}
	return {
		type: 'no-connection' as const,
		heading: 'Your Core is offline',
		description: 'Start your Core, or check its connection, then try again.',
		retry: true,
	}
})

async function searchModpacks(query: string, limit: number = 10) {
	return client.labrinth.projects_v2.search({
		query: query || undefined,
		facets: [['project_type:modpack'], ['client_side:required'], ['server_side:required']],
		limit,
	})
}

async function getProjectVersions(projectId: string) {
	const versions = await client.labrinth.versions_v3.getProjectVersions(projectId)
	return versions.map((version) => ({ id: version.id }))
}

function serverName(config: CreationFlowContextValue) {
	return (
		config.instanceSelection.value?.name ??
		config.modpackSelection.value?.name ??
		config.modpackFile.value?.name.replace(/\.mrpack$/i, '') ??
		'New server'
	)
}

/** Sets up a server that already exists, from what was chosen in the setup modal. */
async function install(serverId: string, config: CreationFlowContextValue) {
	// A Core server has one world, with the server's own id.
	const worldId = serverId
	const properties = config.buildProperties()
	if (config.instanceSelection.value && instanceSource) {
		await instanceSource.pushInstance({
			serverId,
			worldId,
			instanceId: config.instanceSelection.value.id,
			properties,
		})
	} else if (config.setupType.value === 'modpack' && config.modpackFile.value) {
		await client.archon.content_v1.uploadInstance(
			serverId,
			worldId,
			config.modpackFile.value,
			null,
			{ properties },
		).promise
	} else if (config.setupType.value === 'modpack' && config.modpackSelection.value) {
		await client.archon.content_v1.installContent(serverId, worldId, {
			content_variant: 'modpack',
			spec: {
				platform: 'modrinth',
				project_id: config.modpackSelection.value.projectId,
				version_id: config.modpackSelection.value.versionId,
			},
			soft_override: false,
			properties,
		})
	} else {
		const loader = config.selectedLoader.value ?? 'vanilla'
		await client.archon.content_v1.installContent(serverId, worldId, {
			content_variant: 'bare',
			loader: (loader === 'neoforge' ? 'neo_forge' : loader) as Archon.Content.v1.Modloader,
			version: config.selectedLoaderVersion.value ?? '',
			game_version: config.selectedGameVersion.value ?? undefined,
			soft_override: false,
			properties,
		})
	}
}

/**
 * The server is only made once setup is confirmed, so cancelling leaves nothing behind. Once
 * made it stays: if its content then fails to arrive, the server's own page offers setup again,
 * and a server from an instance shows a banner on that instance.
 */
async function createServer(config: CreationFlowContextValue) {
	const instance = config.instanceSelection.value
	try {
		const { id } = await client.archon.servers_v1.createLocal(
			serverName(config),
			instance ? { id: instance.id, name: instance.name } : undefined,
		)
		try {
			await install(id, config)
		} catch (cause) {
			console.error('Failed to set up the new server', cause)
			addNotification({
				type: 'error',
				title: 'Server created, but not set up',
				text: instance
					? 'The instance could not be sent to it. Push an update from the instance to try again.'
					: 'Its content could not be installed. Set it up again from the server.',
			})
		}
		setupModal.value?.hide()
		await queryClient.invalidateQueries({ queryKey: ['servers'] })
		await router.push(`/hosting/manage/${encodeURIComponent(id)}`)
	} catch (cause) {
		console.error('Failed to create server', cause)
		addNotification({
			type: 'error',
			title: 'Could not create the server',
			text: 'Check that your Core is running, then try again.',
		})
	} finally {
		config.loading.value = false
	}
}

/** Browsing modpacks installs onto a server, so this one path makes the server first. */
async function browseModpacks() {
	try {
		const { id } = await client.archon.servers_v1.createLocal('New server')
		setupModal.value?.hide()
		await queryClient.invalidateQueries({ queryKey: ['servers'] })
		await router.push({ path: '/browse/modpack', query: { sid: id, wid: id, from: 'onboarding' } })
	} catch (cause) {
		console.error('Failed to create server', cause)
		addNotification({
			type: 'error',
			title: 'Could not create the server',
			text: 'Check that your Core is running, then try again.',
		})
	}
}
</script>

<template>
	<div data-pyro-server-list-root class="relative mx-auto flex h-full w-full flex-col p-6">
		<div class="mb-4 flex w-full flex-row items-center justify-between gap-2">
			<h1 class="m-0 text-2xl font-extrabold text-contrast">Servers</h1>
			<div class="flex items-center gap-2">
				<Input
					v-if="allServers.length > 0"
					v-model="searchInput"
					:icon="SearchIcon"
					placeholder="Search servers"
				/>
				<IconButton v-tooltip="'Core settings'" label="Core settings" @click="openCoreSettings">
					<SettingsIcon />
				</IconButton>
				<Button
					v-if="allServers.length > 0"
					type="colored"
					color="brand"
					@click="setupModal?.show()"
				>
					<PlusIcon /> New server
				</Button>
			</div>
		</div>

		<EmptyState
			v-if="problem"
			class="my-auto"
			:type="problem.type"
			:heading="problem.heading"
			:description="problem.description"
		>
			<template #actions>
				<Button v-if="problem.retry" :disabled="isFetching" @click="refetch()">
					<UpdatedIcon /> Try again
				</Button>
				<Button @click="openCoreSettings"><SettingsIcon /> Core settings</Button>
			</template>
		</EmptyState>
		<!-- Ghost rows while the list loads. Static on purpose: no shimmer. -->
		<div v-else-if="isLoading" class="flex flex-col gap-3" aria-busy="true">
			<div
				v-for="row in 3"
				:key="row"
				class="flex items-center gap-4 rounded-2xl border border-solid border-surface-4 bg-bg-raised p-4"
			>
				<div class="size-16 shrink-0 rounded-xl bg-surface-4" />
				<div class="flex flex-col gap-2.5">
					<div class="h-5 w-48 rounded-md bg-surface-4" />
					<div class="h-4 w-72 rounded-md bg-surface-3" />
				</div>
			</div>
		</div>
		<EmptyState
			v-else-if="allServers.length === 0"
			class="my-auto"
			type="empty"
			heading="No servers yet"
			description="Create a server on your Core and play with your friends."
		>
			<template #actions>
				<Button type="colored" color="brand" size="lg" @click="setupModal?.show()">
					<PlusIcon /> New server
				</Button>
			</template>
		</EmptyState>
		<div v-else class="flex flex-col gap-3">
			<ServerListing v-for="server in servers" :key="server.server_id" v-bind="server" />
			<EmptyState
				v-if="servers.length === 0"
				type="no-search-result"
				heading="No servers match your search"
			/>
		</div>

		<CreationFlowModal
			ref="setupModal"
			type="server-onboarding"
			:available-loaders="['vanilla', 'fabric', 'neoforge', 'forge', 'quilt', 'paper', 'purpur']"
			:show-snapshot-toggle="true"
			:search-projects="searchModpacks"
			:get-project-versions="getProjectVersions"
			@browse-modpacks="browseModpacks"
			@create="createServer"
		/>
	</div>
</template>
