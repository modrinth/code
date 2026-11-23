<script setup lang="ts">
import { CompassIcon, PlusIcon, ServerIcon, WorldIcon } from '@modrinth/assets'
import {
	Button,
	defineMessages,
	EmptyState,
	GAME_MODES,
	ReadyTransition,
	useReadyState,
	useVIntl,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed, ref } from 'vue'
import { useRoute } from 'vue-router'

import { showWorldInFolder } from '@/features/worlds/actions'
import { gameVersionsQueryOptions } from '@/features/worlds/queries'
import {
	getWorldIdentifier,
	hasServerQuickPlaySupport,
	hasWorldQuickPlaySupport,
} from '@/features/worlds/world-utils'
import WorldItem from '@/features/worlds/WorldItem.vue'
import { injectInstancePage } from '@/pages/instance/instance-context'
import type { ServerWorld, World } from '@/platform/app-lib/worlds/types'

import AddServerModal from './components/AddServerModal.vue'
import DesyncServerModal from './components/DesyncServerModal.vue'
import EditServerModal from './components/EditServerModal.vue'
import EditWorldModal from './components/EditWorldModal.vue'
import QuickplayToolbar from './components/QuickplayToolbar.vue'
import RemoveWorldModal from './components/RemoveWorldModal.vue'
import { useQuickplayFilters } from './composables/use-quickplay-filters'
import { useQuickplayMutations } from './composables/use-quickplay-mutations'
import { useQuickplayWorlds } from './composables/use-quickplay-worlds'
import { useServerStatuses } from './composables/use-server-statuses'
import { useWorldLaunch } from './composables/use-world-launch'
import type { AddServerValues } from './types'

const { formatMessage } = useVIntl()
const route = useRoute()
const instancePage = injectInstancePage()

const messages = defineMessages({
	addServer: {
		id: 'app.instance.worlds.add-server',
		defaultMessage: 'Add server',
	},
	browseServers: {
		id: 'app.instance.worlds.browse-servers',
		defaultMessage: 'Browse servers',
	},
	noServersHeading: {
		id: 'app.instance.worlds.no-servers-heading',
		defaultMessage: 'No servers added',
	},
	noServersDescription: {
		id: 'app.instance.worlds.no-servers-description',
		defaultMessage: 'Add a server by its address or browse servers to get started.',
	},
	noWorldsHeading: {
		id: 'app.instance.worlds.no-worlds-heading',
		defaultMessage: 'No worlds yet',
	},
	noWorldsDescription: {
		id: 'app.instance.worlds.no-worlds-description',
		defaultMessage: 'Singleplayer worlds you create in this instance will appear here.',
	},
})

const addServerModal = ref<InstanceType<typeof AddServerModal> | null>(null)
const editServerModal = ref<InstanceType<typeof EditServerModal> | null>(null)
const editWorldModal = ref<InstanceType<typeof EditWorldModal> | null>(null)
const removeWorldModal = ref<InstanceType<typeof RemoveWorldModal> | null>(null)
const desyncServerModal = ref<InstanceType<typeof DesyncServerModal> | null>(null)

const instance = computed(() => instancePage.instance.value)
const highlightedWorld = computed(() =>
	typeof route.query.highlight === 'string' ? route.query.highlight : undefined,
)

const { worldsQuery, serverWorlds, singleplayerWorlds, isManagedServerWorld, invalidateWorlds } =
	useQuickplayWorlds()
const readyPending = useReadyState(worldsQuery)

const { protocolVersion, getServerData, getServerAvailability, refreshServer, refreshAllServers } =
	useServerStatuses(serverWorlds)

const {
	activeTab,
	searchQuery,
	selectedFilters,
	filterOptions,
	hasAnyWorlds,
	activeWorlds,
	filteredWorlds,
} = useQuickplayFilters({
	serverWorlds,
	singleplayerWorlds,
	getServerAvailability,
	highlightedWorld,
	loaded: () => worldsQuery.isSuccess.value,
})

const { startingInstance, joinWorld, stop, isPlayingWorld } = useWorldLaunch(isManagedServerWorld)

const {
	addServerMutation,
	editServerMutation,
	editWorldMutation,
	removeWorldMutation,
	desyncServerMutation,
} = useQuickplayMutations(instancePage.instanceId)

const gameVersionsQuery = useQuery(gameVersionsQueryOptions())
const supportsServerQuickPlay = computed(() =>
	hasServerQuickPlaySupport(gameVersionsQuery.data.value ?? [], instance.value.game_version),
)
const supportsWorldQuickPlay = computed(() =>
	hasWorldQuickPlaySupport(gameVersionsQuery.data.value ?? [], instance.value.game_version),
)

const refreshing = ref(false)

async function refresh() {
	refreshing.value = true
	try {
		await Promise.all([invalidateWorlds(), refreshAllServers()])
	} finally {
		refreshing.value = false
	}
}

function addServer(values: AddServerValues) {
	addServerMutation.mutate(values, {
		onSuccess: (index) => {
			if (!values.play) return
			const server: ServerWorld = {
				type: 'server',
				index,
				name: values.name,
				address: values.address,
				pack_status: values.packStatus,
				display_status: 'normal',
			}
			void joinWorld(server)
		},
	})
}

function editWorld(world: World) {
	if (world.type === 'singleplayer') {
		editWorldModal.value?.show(world)
	} else if (!isManagedServerWorld(world)) {
		editServerModal.value?.show(world)
	}
}

function removeWorld(world: World) {
	if (!isManagedServerWorld(world)) removeWorldModal.value?.show(world)
}

function desyncServer(world: World) {
	if (world.type === 'server') desyncServerModal.value?.show(world)
}

function serverData(world: World) {
	return world.type === 'server' ? getServerData(world) : undefined
}

function worldKey(world: World) {
	return world.type === 'singleplayer'
		? `world-singleplayer-${world.path}`
		: `world-server-${world.address}-${world.index}`
}
</script>

<template>
	<AddServerModal ref="addServerModal" @submit="addServer" />
	<EditServerModal ref="editServerModal" @submit="editServerMutation.mutate" />
	<EditWorldModal ref="editWorldModal" @submit="editWorldMutation.mutate" />
	<RemoveWorldModal ref="removeWorldModal" @confirm="removeWorldMutation.mutate" />
	<DesyncServerModal
		ref="desyncServerModal"
		@confirm="(server, mode) => desyncServerMutation.mutate({ server, mode })"
	/>
	<ReadyTransition :pending="readyPending">
		<div class="flex flex-col gap-4">
			<QuickplayToolbar
				v-if="hasAnyWorlds"
				v-model:tab="activeTab"
				v-model:search="searchQuery"
				v-model:filters="selectedFilters"
				:world-count="activeWorlds.length"
				:filter-options="filterOptions"
				:show-server-actions="serverWorlds.length > 0"
				:refreshing="refreshing"
				@add-server="addServerModal?.show($event)"
				@browse-servers="instancePage.browseServers"
				@refresh="refresh"
			/>
			<div v-if="activeWorlds.length > 0" class="flex w-full flex-col gap-2">
				<WorldItem
					v-for="world in filteredWorlds"
					:key="worldKey(world)"
					:world="world"
					:managed="isManagedServerWorld(world)"
					:highlighted="highlightedWorld === getWorldIdentifier(world)"
					:supports-server-quick-play="supportsServerQuickPlay"
					:supports-world-quick-play="supportsWorldQuickPlay"
					:launch-blocked="instance.launch_blocked"
					:current-protocol="protocolVersion"
					:playing-instance="instancePage.playing.value"
					:playing-world="isPlayingWorld(world)"
					:starting-instance="startingInstance"
					:refreshing="serverData(world)?.refreshing"
					:server-status="serverData(world)?.status"
					:rendered-motd="serverData(world)?.renderedMotd"
					:game-mode="world.type === 'singleplayer' ? GAME_MODES[world.game_mode] : undefined"
					:shortcut-instance-id="instance.id"
					@play="joinWorld(world)"
					@stop="stop"
					@refresh="world.type === 'server' && refreshServer(world.address)"
					@edit="editWorld(world)"
					@delete="removeWorld(world)"
					@desync="desyncServer(world)"
					@open-folder="
						(singleplayerWorld) => showWorldInFolder(instance.id, singleplayerWorld.path)
					"
				/>
			</div>
			<EmptyState
				v-else-if="activeTab === 'servers'"
				:icon="ServerIcon"
				:heading="formatMessage(messages.noServersHeading)"
				:description="formatMessage(messages.noServersDescription)"
			>
				<template #actions>
					<Button type="outlined" size="lg" @click="addServerModal?.show($event)">
						<PlusIcon aria-hidden="true" />
						{{ formatMessage(messages.addServer) }}
					</Button>
					<Button type="colored" color="brand" size="lg" @click="instancePage.browseServers">
						<CompassIcon aria-hidden="true" />
						{{ formatMessage(messages.browseServers) }}
					</Button>
				</template>
			</EmptyState>
			<EmptyState
				v-else
				:icon="WorldIcon"
				:heading="formatMessage(messages.noWorldsHeading)"
				:description="formatMessage(messages.noWorldsDescription)"
			/>
		</div>
	</ReadyTransition>
</template>
