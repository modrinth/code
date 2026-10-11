<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { LinkIcon, PlusIcon, SearchIcon } from '@modrinth/assets'
import { Button, injectModrinthClient, Input, NewModal, ServerListing } from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'

import { ensureCorePaired, openPairCoreModal, useCores } from '@/composables/use-cores'

const client = injectModrinthClient()
const router = useRouter()
const queryClient = useQueryClient()
const searchInput = ref('')
const creating = ref(false)
const creationError = ref('')
const chooseCore = ref<InstanceType<typeof NewModal> | null>(null)
const { cores, needsPairing } = useCores()
const { data, error, isLoading, refetch } = useQuery({
	queryKey: ['servers'],
	queryFn: () => client.archon.servers_v0.list({ limit: 100 }),
	staleTime: 30_000,
})
const servers = computed(() =>
	(data.value?.servers ?? []).filter((server) =>
		server.name.toLowerCase().includes(searchInput.value.toLowerCase()),
	),
)

// Pairs a Core first when there is none, and asks which Core only when there are several.
async function newServer() {
	if (creating.value || !(await ensureCorePaired())) return
	const paired = await client.archon.cores_v1.list()
	if (paired.length > 1) chooseCore.value?.show()
	else await createServer(paired[0])
}

async function createServer(core: Archon.Cores.v1.Core) {
	chooseCore.value?.hide()
	if (creating.value) return
	creating.value = true
	creationError.value = ''
	try {
		const server = await client.archon.cores_v1.createServer('New server', core)
		await queryClient.invalidateQueries({ queryKey: ['servers'] })
		await router.push(`/hosting/manage/${encodeURIComponent(server.id)}`)
	} catch (error) {
		creationError.value = error instanceof Error ? error.message : String(error)
	} finally {
		creating.value = false
	}
}
</script>

<template>
	<div data-pyro-server-list-root class="relative mx-auto flex w-full flex-col p-6">
		<NewModal ref="chooseCore" header="Choose a Core" max-width="460px" width="100%">
			<div class="flex flex-col gap-2">
				<Button v-for="core in cores" :key="core.id" class="w-full" @click="createServer(core)">
					{{ core.name }}
				</Button>
			</div>
		</NewModal>
		<div class="flex w-full flex-row items-center justify-between gap-2 mb-4">
			<h1 class="text-2xl m-0 font-extrabold text-contrast">Servers</h1>
			<div class="flex items-center gap-2">
				<Input v-model="searchInput" :icon="SearchIcon" placeholder="Search servers" />
				<Button type="colored" color="brand" :disabled="creating" @click="newServer">
					<PlusIcon /> New server
				</Button>
			</div>
		</div>
		<div v-if="needsPairing" class="flex flex-col items-start gap-3">
			<p class="m-0">Pair a Core to host servers.</p>
			<Button type="colored" color="brand" @click="openPairCoreModal()">
				<LinkIcon /> Pair a Core
			</Button>
		</div>
		<div v-else-if="error" class="flex flex-col gap-3">
			<p class="text-red">{{ error.message }}</p>
			<Button @click="refetch()">Retry</Button>
		</div>
		<p v-else-if="isLoading">Loading servers...</p>
		<div v-else class="flex flex-col gap-3">
			<ServerListing v-for="server in servers" :key="server.server_id" v-bind="server" />
			<p v-if="!servers.length">No servers found.</p>
		</div>
		<p v-if="creationError" class="text-red">{{ creationError }}</p>
	</div>
</template>
