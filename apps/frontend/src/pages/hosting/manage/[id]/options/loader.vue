<template>
	<ServerSetupModal ref="setupModal" :server="props.server" @reinstall="onReinstall" />

	<PlatformChangeModpackVersionModal
		ref="modpackVersionModal"
		:server="props.server"
		:project="data?.project"
		:versions="Array.isArray(versions) ? versions : []"
		:current-version="currentVersion"
		:current-version-id="data?.upstream?.version_id"
		:server-status="data?.status"
		@reinstall="emit('reinstall')"
	/>

	<div class="flex h-full w-full flex-col">
		<div v-if="data" class="flex w-full flex-col">
			<div class="card flex flex-col gap-4">
				<div class="flex select-none flex-col items-center justify-between gap-2 lg:flex-row">
					<div class="flex flex-row items-center gap-2">
						<h2 class="m-0 text-lg font-bold text-contrast">Modpack</h2>
						<div
							v-if="updateAvailable"
							class="rounded-full bg-bg-orange px-2 py-1 text-xs font-medium text-orange"
						>
							<span>Update available</span>
						</div>
					</div>
					<div v-if="data.upstream" class="flex gap-4">
						<ButtonStyled>
							<template v-if="isInstalling">
								<button :disabled="isInstalling">
									<TransferIcon class="size-4" />
									Switch modpack
								</button>
							</template>
							<nuxt-link v-else :to="`/discover/modpacks?sid=${props.server.serverId}`">
								<TransferIcon class="size-4" />
								Switch modpack
							</nuxt-link>
						</ButtonStyled>
					</div>
				</div>
				<div v-if="data.upstream" class="flex flex-col gap-2">
					<div
						v-if="versionsError || currentVersionError"
						class="rounded-2xl border border-solid border-red p-4 text-contrast"
					>
						<p class="m-0 font-bold">Something went wrong while loading your modpack.</p>
						<p class="m-0 mb-2 mt-1 text-sm">
							{{ versionsError || currentVersionError }}
						</p>
						<ButtonStyled>
							<button :disabled="isInstalling" @click="refreshData">Retry</button>
						</ButtonStyled>
					</div>

					<ProjectCard
						v-if="!versionsError && !currentVersionError"
						class="!bg-bg"
						:title="projectCardData.title"
						:icon-url="projectCardData.icon_url"
						:date-updated="projectCardData.date_modified"
						:followers="projectCardData.follows"
						:downloads="projectCardData.downloads"
						layout="list"
						:summary="projectCardData.description"
						:tags="data.project?.categories || []"
					>
						<template #actions>
							<ButtonStyled color="brand">
								<button :disabled="isInstalling" @click="modpackVersionModal?.show()">
									<SettingsIcon class="size-4" />
									Change version
								</button>
							</ButtonStyled>
						</template>
					</ProjectCard>
				</div>
				<div v-else class="flex w-full flex-col items-center gap-2 sm:w-fit sm:flex-row">
					<ButtonStyled>
						<nuxt-link
							v-tooltip="backupInProgress ? formatMessage(backupInProgress.tooltip) : undefined"
							:class="{ disabled: backupInProgress }"
							class="!w-full sm:!w-auto"
							:to="`/discover/modpacks?sid=${props.server.serverId}`"
						>
							<CompassIcon class="size-4" /> Find a modpack
						</nuxt-link>
					</ButtonStyled>
				</div>
			</div>

			<div class="card flex flex-col gap-4">
				<div class="flex flex-col gap-2">
					<h2 class="m-0 text-lg font-bold text-contrast">Platform</h2>
					<p class="m-0">Your server's platform is the software that runs mods and plugins.</p>
					<div v-if="data.upstream" class="mt-2 flex items-center gap-2">
						<InfoIcon class="hidden sm:block" />
						<span class="text-sm text-secondary">
							The current platform was automatically selected based on your modpack.
						</span>
					</div>
				</div>
				<div
					v-if="data.loader"
					class="flex items-center justify-between rounded-2xl bg-table-alternateRow p-4"
				>
					<div class="flex items-center gap-4">
						<div
							class="grid size-10 place-content-center rounded-xl border border-solid border-button-border bg-button-bg shadow-sm"
						>
							<LoaderIcon :loader="data.loader" class="size-6" />
						</div>
						<div class="flex flex-col gap-0.5">
							<span class="text-lg font-bold text-contrast">{{ data.loader }}</span>
							<span v-if="data.loader_version" class="text-xs text-secondary">
								{{ data.loader_version }}
								<template v-if="data.mc_version">
									&middot; Minecraft {{ data.mc_version }}
								</template>
							</span>
						</div>
					</div>
				</div>
				<ButtonStyled>
					<button
						v-tooltip="backupInProgress ? formatMessage(backupInProgress.tooltip) : undefined"
						:disabled="isInstalling || !!backupInProgress"
						@click="setupModal?.show()"
					>
						<SettingsIcon class="size-4" />
						Change platform
					</button>
				</ButtonStyled>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { CompassIcon, InfoIcon, SettingsIcon, TransferIcon } from '@modrinth/assets'
import { ButtonStyled, LoaderIcon, ProjectCard, useVIntl } from '@modrinth/ui'
import { useTemplateRef } from 'vue'

import PlatformChangeModpackVersionModal from '~/components/ui/servers/PlatformChangeModpackVersionModal.vue'
import ServerSetupModal from '~/components/ui/servers/ServerSetupModal.vue'
import type { ModrinthServer } from '~/composables/servers/modrinth-servers.ts'
import type { BackupInProgressReason } from '~/pages/hosting/manage/[id].vue'

const { formatMessage } = useVIntl()

const props = defineProps<{
	server: ModrinthServer
	backupInProgress?: BackupInProgressReason
}>()

const emit = defineEmits<{
	reinstall: [any?]
}>()

const setupModal = useTemplateRef<InstanceType<typeof ServerSetupModal>>('setupModal')
const modpackVersionModal =
	useTemplateRef<InstanceType<typeof PlatformChangeModpackVersionModal>>('modpackVersionModal')

const isInstalling = computed(() => props.server.general?.status === 'installing')
const data = computed(() => props.server.general)

const {
	data: versions,
	error: versionsError,
	refresh: refreshVersions,
} = await useAsyncData(
	`content-loader-versions-${data.value?.upstream?.project_id}`,
	async () => {
		if (!data.value?.upstream?.project_id) return []
		try {
			const result = await useBaseFetch(`project/${data.value.upstream.project_id}/version`)
			return result || []
		} catch (e) {
			console.error('couldnt fetch all versions:', e)
			throw new Error('Failed to load modpack versions.')
		}
	},
	{ default: () => [] },
)

const {
	data: currentVersion,
	error: currentVersionError,
	refresh: refreshCurrentVersion,
} = await useAsyncData(
	`content-loader-version-${data.value?.upstream?.version_id}`,
	async () => {
		if (!data.value?.upstream?.version_id) return null
		try {
			const result = await useBaseFetch(`version/${data.value.upstream.version_id}`)
			return result || null
		} catch (e) {
			console.error('couldnt fetch version:', e)
			throw new Error('Failed to load modpack version.')
		}
	},
	{ default: () => null },
)

const projectCardData = computed(() => ({
	icon_url: data.value?.project?.icon_url,
	title: data.value?.project?.title,
	description: data.value?.project?.description,
	downloads: data.value?.project?.downloads,
	follows: data.value?.project?.followers,
	// @ts-ignore
	date_modified: currentVersion.value?.date_published || data.value?.project?.updated,
}))

const refreshData = async () => {
	await Promise.all([refreshVersions(), refreshCurrentVersion()])
}

const updateAvailable = computed(() => {
	// @ts-ignore
	if (!data.value?.upstream || !versions.value?.length || !currentVersion.value) {
		return false
	}

	// @ts-ignore
	const latestVersion = versions.value[0]
	// @ts-ignore
	return latestVersion.id !== currentVersion.value.id
})

const onReinstall = (args: any) => {
	emit('reinstall', args)
}

watch(
	() => props.server.general?.status,
	async (newStatus, oldStatus) => {
		if (oldStatus === 'installing' && newStatus === 'available') {
			await Promise.all([
				refreshVersions(),
				refreshCurrentVersion(),
				props.server.refresh(['general']),
			])
		}
	},
)
</script>
