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
				<h2 class="m-0 text-lg font-bold text-contrast">Installation</h2>

				<!-- DEBUG: Cobblemon modpack card -->
				<div
					v-if="debugProject"
					class="flex items-center gap-3 rounded-2xl border border-solid border-surface-4 bg-surface-2 p-3"
				>
					<Avatar :src="(debugProject as any).icon_url" size="3rem" />
					<div class="flex flex-1 flex-col">
						<div class="flex items-center gap-1.5">
							<span class="line-clamp-1 font-medium text-contrast">
								{{ (debugProject as any).title }}
							</span>
							<span class="text-sm text-secondary">
								by {{ (debugProject as any).team?.name ?? 'Unknown' }}
							</span>
						</div>
						<span class="line-clamp-1 text-sm text-secondary">v1.6.1 for 1.21.1</span>
					</div>
					<div class="flex shrink-0 items-center gap-2">
						<div class="rounded-full bg-bg-orange px-2 py-1 text-xs font-medium text-orange">
							Update available
						</div>
						<ButtonStyled color="brand">
							<button>
								<SettingsIcon class="size-4" />
								Change version
							</button>
						</ButtonStyled>
					</div>
				</div>

				<!-- Summary -->
				<div class="flex flex-col gap-3">
					<!-- Modpack card -->
					<div v-if="data.upstream">
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
							v-else
							class="!bg-bg"
							:title="projectCardData.title ?? ''"
							:icon-url="projectCardData.icon_url"
							:date-updated="projectCardData.date_modified"
							:followers="projectCardData.follows"
							:downloads="projectCardData.downloads"
							layout="list"
							:summary="projectCardData.description"
							:tags="data.project?.categories || []"
						>
							<template #actions>
								<div class="flex items-center gap-2">
									<div
										v-if="updateAvailable"
										class="rounded-full bg-bg-orange px-2 py-1 text-xs font-medium text-orange"
									>
										Update available
									</div>
									<ButtonStyled color="brand">
										<button :disabled="isInstalling" @click="modpackVersionModal?.show()">
											<SettingsIcon class="size-4" />
											Change version
										</button>
									</ButtonStyled>
								</div>
							</template>
						</ProjectCard>
					</div>

					<!-- Platform card -->
					<div
						v-if="data.loader"
						class="flex items-center gap-3 rounded-2xl border border-solid border-surface-4 bg-surface-2 p-3"
					>
						<div
							class="grid size-12 shrink-0 place-content-center rounded-xl border border-solid border-button-border bg-button-bg shadow-sm"
						>
							<LoaderIcon :loader="data.loader as Loaders" class="size-8" />
						</div>
						<div class="flex flex-1 flex-col">
							<span class="font-medium text-contrast">{{ data.loader }}</span>
							<span v-if="data.loader_version" class="text-sm text-secondary">
								{{ data.loader_version }}
								<template v-if="data.mc_version">
									&middot; Minecraft {{ data.mc_version }}
								</template>
							</span>
						</div>
					</div>
				</div>

				<!-- Unlink modpack -->
				<div v-if="data.upstream" class="flex flex-col gap-1">
					<span class="text-lg font-bold text-contrast">Unlink modpack</span>
					<span class="text-sm text-secondary">
						Detach the modpack from this server. Modpack content will remain installed, but will no
						longer be managed.
					</span>
					<ButtonStyled color="orange" type="outlined">
						<button
							v-tooltip="backupInProgress ? formatMessage(backupInProgress.tooltip) : undefined"
							:disabled="isInstalling || !!backupInProgress"
							class="mt-1"
							@click="unlinkModpack"
						>
							<UnlinkIcon class="size-4" />
							Unlink modpack
						</button>
					</ButtonStyled>
				</div>

				<!-- Change installation -->
				<div class="flex flex-col gap-1">
					<span class="text-lg font-bold text-contrast">Change installation</span>
					<span class="text-sm text-secondary">
						Change your server's loader, modpack, or game version.
					</span>
					<ButtonStyled>
						<button
							v-tooltip="backupInProgress ? formatMessage(backupInProgress.tooltip) : undefined"
							:disabled="isInstalling || !!backupInProgress"
							class="mt-1"
							@click="setupModal?.show()"
						>
							<SettingsIcon class="size-4" />
							Change installation
						</button>
					</ButtonStyled>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { SettingsIcon, UnlinkIcon } from '@modrinth/assets'
import {
	Avatar,
	ButtonStyled,
	injectNotificationManager,
	LoaderIcon,
	ProjectCard,
	useVIntl,
} from '@modrinth/ui'
import { type Loaders, ModrinthServersFetchError } from '@modrinth/utils'
import { useTemplateRef } from 'vue'

import PlatformChangeModpackVersionModal from '~/components/ui/servers/PlatformChangeModpackVersionModal.vue'
import ServerSetupModal from '~/components/ui/servers/ServerSetupModal.vue'
import type { ModrinthServer } from '~/composables/servers/modrinth-servers.ts'
import type { BackupInProgressReason } from '~/pages/hosting/manage/[id].vue'

const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()

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

// DEBUG: Fetch Cobblemon modpack for testing
const { data: debugProject } = await useAsyncData('debug-cobblemon', async () => {
	try {
		return await useBaseFetch('project/cobblemon-modpack')
	} catch {
		return null
	}
})

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

async function unlinkModpack() {
	if (!data.value?.loader || !data.value?.mc_version) return

	try {
		await props.server.general.reinstall(
			true,
			data.value.loader,
			data.value.mc_version,
			data.value.loader_version ?? '',
			false,
		)
		emit('reinstall')
	} catch (error) {
		if (error instanceof ModrinthServersFetchError && error.statusCode === 429) {
			addNotification({
				title: 'Cannot unlink modpack',
				text: 'You are being rate limited. Please try again later.',
				type: 'error',
			})
		} else {
			addNotification({
				title: 'Unlink failed',
				text: 'An unexpected error occurred. Please try again later.',
				type: 'error',
			})
		}
	}
}

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
