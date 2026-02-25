<template>
	<ConfirmUnlinkModal ref="unlinkModal" server @unlink="handleUnlinkConfirm" />

	<ServerSetupModal ref="setupModal" @reinstall="emit('reinstall', $event)" />

	<PlatformChangeModpackVersionModal
		ref="modpackVersionModal"
		:project="serverProject"
		:versions="Array.isArray(versions) ? versions : []"
		:current-version="currentVersion"
		:current-version-id="server?.upstream?.version_id"
		:server-status="server?.status"
		@reinstall="emit('reinstall')"
	/>

	<div class="h-full w-full">
		<template v-if="server">
			<!-- Installed modpack -->
			<div v-if="server.upstream" class="card flex flex-col gap-2.5">
				<h2 class="m-0 text-lg font-bold text-contrast">Installed modpack</h2>
				<div class="flex items-center gap-2.5 rounded-[20px] bg-surface-2 p-3">
					<div
						class="size-14 shrink-0 overflow-hidden rounded-2xl border border-solid border-surface-5"
					>
						<img
							v-if="serverProject?.icon_url"
							:src="serverProject.icon_url"
							:alt="serverProject?.title"
							class="size-full object-cover"
						/>
					</div>
					<div class="flex flex-col gap-1">
						<span class="font-semibold text-contrast">
							{{ serverProject?.title ?? 'Unknown modpack' }}
						</span>
					</div>
				</div>
				<div class="flex gap-2">
					<ButtonStyled>
						<button :disabled="isInstalling" @click="modpackVersionModal?.show()">
							<ArrowLeftRightIcon class="size-5" />
							Change version
						</button>
					</ButtonStyled>
					<ButtonStyled>
						<button :disabled="isInstalling" @click="unlinkModal?.show()">
							<UnlinkIcon class="size-5" />
							Unlink
						</button>
					</ButtonStyled>
				</div>
			</div>

			<!-- Installation info -->
			<div class="card flex flex-col gap-2.5">
				<h2 class="m-0 text-lg font-bold text-contrast">Installation info</h2>
				<div class="flex flex-col gap-2.5 overflow-clip rounded-[20px] bg-surface-2 p-4">
					<div class="flex items-center justify-between">
						<span class="text-primary">Platform</span>
						<span class="font-semibold text-contrast">
							{{ server.loader ?? 'Unknown' }}
						</span>
					</div>
					<div class="flex items-center justify-between">
						<span class="text-primary">Game version</span>
						<span class="font-semibold text-contrast">
							{{ server.mc_version ?? 'Unknown' }}
						</span>
					</div>
					<div
						v-if="server.loader && server.loader !== 'Vanilla'"
						class="flex items-center justify-between"
					>
						<span class="text-primary">{{ server.loader }} version</span>
						<span class="font-semibold text-contrast">
							{{ server.loader_version ?? 'Unknown' }}
						</span>
					</div>
				</div>
			</div>

			<!-- Reset server -->
			<div class="card flex flex-col gap-2.5">
				<h2 class="m-0 text-lg font-bold text-contrast">Reset server</h2>
				<div>
					<ButtonStyled color="red">
						<button :disabled="isInstalling" @click="setupModal?.show()">
							<RotateCounterClockwiseIcon class="size-5" />
							Reset server
						</button>
					</ButtonStyled>
				</div>
				<p class="m-0 text-primary">
					Removes all data on your server, including your worlds, mods, and configuration files.
					Backups will remain and can be restored.
				</p>
			</div>
		</template>
	</div>
</template>

<script setup lang="ts">
import { ArrowLeftRightIcon, RotateCounterClockwiseIcon, UnlinkIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	ConfirmUnlinkModal,
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
	ServerSetupModal,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'

import PlatformChangeModpackVersionModal from '~/components/ui/servers/PlatformChangeModpackVersionModal.vue'
import { useServerProject } from '~/composables/servers/use-server-project.ts'

const client = injectModrinthClient()
const { server, serverId, worldId } = injectModrinthServerContext()
const { data: serverProject } = useServerProject(computed(() => server.value?.upstream ?? null))
const { addNotification } = injectNotificationManager()
const queryClient = useQueryClient()

const emit = defineEmits<{
	reinstall: [any?]
}>()

const isInstalling = computed(() => server.value?.status === 'installing')

const unlinkModal = ref<InstanceType<typeof ConfirmUnlinkModal>>()
const setupModal = ref<InstanceType<typeof ServerSetupModal>>()
const modpackVersionModal = ref()

const { data: versions, refresh: refreshVersions } = await useAsyncData(
	`content-loader-versions-${server.value?.upstream?.project_id}`,
	async () => {
		if (!server.value?.upstream?.project_id) return []
		try {
			const result = await useBaseFetch(`project/${server.value.upstream.project_id}/version`)
			return result || []
		} catch (e) {
			console.error('couldnt fetch all versions:', e)
			throw new Error('Failed to load modpack versions.')
		}
	},
	{ default: () => [] },
)

const { data: currentVersion, refresh: refreshCurrentVersion } = await useAsyncData(
	`content-loader-version-${server.value?.upstream?.version_id}`,
	async () => {
		if (!server.value?.upstream?.version_id) return null
		try {
			const result = await useBaseFetch(`version/${server.value.upstream.version_id}`)
			return result || null
		} catch (e) {
			console.error('couldnt fetch version:', e)
			throw new Error('Failed to load modpack version.')
		}
	},
	{ default: () => null },
)

async function handleUnlinkConfirm() {
	try {
		await client.archon.content_v1.unlinkModpack(serverId, worldId.value ?? undefined)
		await queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] })
	} catch (err) {
		addNotification({
			type: 'error',
			text: err instanceof Error ? err.message : 'Failed to unlink modpack',
		})
	}
}

watch(
	() => server.value?.status,
	async (newStatus, oldStatus) => {
		if (oldStatus === 'installing' && newStatus === 'available') {
			await Promise.all([
				refreshVersions(),
				refreshCurrentVersion(),
				queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] }),
			])
		}
	},
)
</script>
