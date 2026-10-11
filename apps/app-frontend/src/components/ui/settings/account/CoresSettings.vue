<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { LinkIcon, UnlinkIcon } from '@modrinth/assets'
import {
	Button,
	ConfirmModal,
	injectModrinthClient,
	injectNotificationManager,
	useRelativeTime,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import { ref } from 'vue'

import { coresQueryKey, openPairCoreModal, useCores } from '@/composables/use-cores'

const client = injectModrinthClient()
const queryClient = useQueryClient()
const { handleError } = injectNotificationManager()
const formatRelativeTime = useRelativeTime()
const { cores, isLoading, error } = useCores()

const confirmUnpair = ref<InstanceType<typeof ConfirmModal> | null>(null)
const unpairing = ref<Archon.Cores.v1.Core | null>(null)

function askUnpair(core: Archon.Cores.v1.Core) {
	unpairing.value = core
	confirmUnpair.value?.show()
}

async function unpair() {
	const core = unpairing.value
	if (!core) return
	try {
		await client.archon.cores_v1.unpair(core)
	} catch (reason) {
		handleError(reason instanceof Error ? reason : new Error(String(reason)))
	} finally {
		unpairing.value = null
		await Promise.all([
			queryClient.invalidateQueries({ queryKey: coresQueryKey }),
			queryClient.invalidateQueries({ queryKey: ['servers'] }),
		])
	}
}
</script>

<template>
	<ConfirmModal
		ref="confirmUnpair"
		title="Unpair this Core?"
		description="Its servers and files stay on the Core. It will show a new code so it can be paired again."
		proceed-label="Unpair"
		:proceed-icon="UnlinkIcon"
		@proceed="unpair"
	/>
	<div class="flex flex-col gap-4">
		<div class="flex items-center justify-between gap-4">
			<h2 class="m-0 text-lg font-semibold text-contrast">Cores</h2>
			<Button type="colored" color="brand" @click="openPairCoreModal()">
				<LinkIcon aria-hidden="true" /> Pair a Core
			</Button>
		</div>
		<p v-if="error" class="m-0 text-red">{{ error.message }}</p>
		<p v-else-if="isLoading" class="m-0 text-secondary">Loading Cores...</p>
		<p v-else-if="!cores.length" class="m-0 text-secondary">No Core is paired.</p>
		<div v-else class="flex flex-col">
			<div
				v-for="core in cores"
				:key="core.id"
				class="flex items-center justify-between gap-4 border-0 border-t border-solid border-surface-5 py-3 first:border-t-0"
			>
				<div class="flex min-w-0 flex-col gap-1">
					<span class="truncate font-semibold text-contrast">{{ core.name }}</span>
					<span class="truncate text-sm text-secondary">
						{{ core.address }} · Last seen {{ formatRelativeTime(core.last_seen) }}
					</span>
				</div>
				<Button :disabled="unpairing?.id === core.id" @click="askUnpair(core)">
					<UnlinkIcon aria-hidden="true" /> Unpair
				</Button>
			</div>
		</div>
	</div>
</template>
