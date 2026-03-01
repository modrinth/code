<template>
	<div class="flex flex-col gap-6 rounded-2xl bg-surface-3 p-6">
		<template v-if="server">
			<template v-if="isLinked">
				<div class="flex flex-col gap-2.5">
					<span class="text-lg font-semibold text-contrast">Installation info</span>
					<div class="flex flex-col gap-2.5 rounded-[20px] bg-surface-2 p-4">
						<div
							v-for="row in installationInfo"
							:key="row.label"
							class="flex items-center justify-between"
						>
							<span class="text-primary">{{ row.label }}</span>
							<span class="font-semibold text-contrast">{{ row.value }}</span>
						</div>
					</div>
				</div>

				<div class="flex flex-col gap-2.5">
					<span class="text-lg font-semibold text-contrast">Installed modpack</span>
					<div v-if="modpack" class="flex items-center gap-2.5 rounded-[20px] bg-surface-2 p-3">
						<AutoLink :to="`/project/${modpack.spec.project_id}`" class="shrink-0">
							<div
								class="size-14 shrink-0 overflow-hidden rounded-2xl border border-solid border-surface-5"
							>
								<Avatar
									v-if="modpack.icon_url"
									:src="modpack.icon_url"
									:alt="modpack.title ?? 'Modpack'"
									size="100%"
									no-shadow
								/>
							</div>
						</AutoLink>
						<div class="flex flex-col gap-1">
							<AutoLink
								:to="`/project/${modpack.spec.project_id}`"
								class="font-semibold text-contrast hover:underline"
							>
								{{ modpack.title ?? modpack.spec.project_id }}
							</AutoLink>
							<div
								v-if="modpack.version_number"
								class="flex items-center gap-2 text-sm font-medium text-primary"
							>
								<span>{{ modpack.version_number }}</span>
							</div>
						</div>
					</div>
					<div class="flex flex-wrap gap-2">
						<ButtonStyled>
							<button
								class="!shadow-none"
								:disabled="isInstalling"
								@click="modpackVersionModal?.show()"
							>
								<ArrowLeftRightIcon class="size-5" />
								Change version
							</button>
						</ButtonStyled>
						<ButtonStyled color="orange">
							<button class="!shadow-none" :disabled="isInstalling" @click="repairModal?.show()">
								<HammerIcon class="size-5" />
								Repair
							</button>
						</ButtonStyled>
					</div>
				</div>

				<div class="flex flex-col gap-2.5">
					<span class="text-lg font-semibold text-contrast">Linked instance</span>
					<span class="text-primary">
						Unlinking permanently disconnects this instance from the modpack project, allowing you
						to change the loader and Minecraft version, but you won't receive future updates.
					</span>
					<div>
						<ButtonStyled color="orange">
							<button class="!shadow-none" :disabled="isInstalling" @click="unlinkModal?.show()">
								<UnlinkIcon class="size-5" />
								Unlink modpack
							</button>
						</ButtonStyled>
					</div>
				</div>

				<div class="flex flex-col gap-2.5">
					<span class="text-lg font-semibold text-contrast">Re-install modpack</span>
					<span class="text-primary">
						Re-installing the modpack resets the instance's content to its original state, removing
						any mods or content you have added.
					</span>
					<div>
						<ButtonStyled color="red">
							<button class="!shadow-none" :disabled="isInstalling" @click="reinstallModal?.show()">
								<DownloadIcon class="size-5" />
								Re-install modpack
							</button>
						</ButtonStyled>
					</div>
				</div>

				<div class="flex flex-col gap-2.5">
					<span class="text-lg font-semibold text-contrast">Reset server</span>
					<span class="text-primary">
						Removes all data on your server, including your worlds, mods, and configuration files.
						Backups will remain and can be restored.
					</span>
					<div>
						<ButtonStyled color="red">
							<button class="!shadow-none" :disabled="isInstalling" @click="setupModal?.show()">
								<RotateCounterClockwiseIcon class="size-5" />
								Reset server
							</button>
						</ButtonStyled>
					</div>
				</div>
			</template>

			<template v-else>
				<div class="flex flex-col gap-2.5">
					<span class="text-lg font-semibold text-contrast">Installation info</span>

					<div v-if="!isEditing" class="flex flex-col gap-2.5 rounded-[20px] bg-surface-2 p-4">
						<div
							v-for="row in installationInfo"
							:key="row.label"
							class="flex items-center justify-between"
						>
							<span class="text-primary">{{ row.label }}</span>
							<span class="font-semibold text-contrast">{{ row.value }}</span>
						</div>
					</div>

					<div
						v-else
						class="flex flex-col gap-3 rounded-[20px] border border-solid border-surface-5 p-4"
					>
						<div class="flex flex-col gap-2.5">
							<span class="font-semibold text-contrast">Platform</span>
							<Chips v-model="selectedPlatform" :items="availablePlatforms" />
						</div>

						<div class="flex flex-col gap-2.5">
							<span class="font-semibold text-contrast">Game version</span>
							<Combobox
								v-model="selectedGameVersion"
								:options="gameVersionOptions"
								searchable
								sync-with-selection
								placeholder="Select version"
								search-placeholder="Search game version..."
								:display-value="selectedGameVersion || 'Select version'"
							>
								<template v-if="hasSnapshots" #dropdown-footer>
									<button
										class="flex w-full cursor-pointer items-center justify-center gap-1.5 border-0 border-t border-solid border-surface-5 bg-transparent py-3 text-center text-sm font-semibold text-secondary transition-colors hover:text-contrast"
										@mousedown.prevent
										@click="showSnapshots = !showSnapshots"
									>
										<EyeOffIcon v-if="showSnapshots" class="size-4" />
										<EyeIcon v-else class="size-4" />
										{{ showSnapshots ? 'Hide snapshots' : 'Show all versions' }}
									</button>
								</template>
							</Combobox>
						</div>

						<div v-if="selectedPlatform !== 'vanilla'" class="flex flex-col gap-2.5">
							<span class="font-semibold text-contrast"> {{ formattedLoaderName }} version </span>
							<Combobox
								v-model="selectedLoaderVersion"
								searchable
								sync-with-selection
								:placeholder="loaderVersionDisplayValue"
								search-placeholder="Search version..."
								:options="loaderVersionOptions"
								:display-value="loaderVersionDisplayValue"
							/>
						</div>

						<div class="flex flex-wrap gap-2">
							<ButtonStyled color="brand">
								<button
									class="!shadow-none"
									:disabled="!isValid || !hasChanges || isSaving"
									@click="handleSave"
								>
									<SpinnerIcon v-if="isSaving" class="animate-spin" />
									<SaveIcon v-else />
									{{ isSaving ? 'Saving...' : 'Save' }}
								</button>
							</ButtonStyled>
							<ButtonStyled type="outlined">
								<button class="!border !border-surface-5 !shadow-none" @click="cancelEditing">
									<XIcon />
									Cancel
								</button>
							</ButtonStyled>
						</div>
					</div>
				</div>

				<template v-if="!isEditing">
					<div class="flex items-start gap-2">
						<CircleAlertIcon class="mt-0.5 size-5 shrink-0 text-orange" />
						<span class="text-primary">
							We don't recommend editing your installation settings after installing content. If you
							want to edit them reset your server.
						</span>
					</div>

					<div class="flex flex-wrap gap-2">
						<ButtonStyled color="orange">
							<button class="!shadow-none" :disabled="isInstalling" @click="isEditing = true">
								<PencilIcon class="size-5" />
								Edit
							</button>
						</ButtonStyled>
						<ButtonStyled>
							<button class="!shadow-none" :disabled="isInstalling" @click="setupModal?.show()">
								Reset server
								<ChevronRightIcon class="size-5" />
							</button>
						</ButtonStyled>
					</div>
				</template>

				<div class="flex flex-col gap-2.5">
					<span class="text-lg font-semibold text-contrast">Repair server</span>
					<span class="text-primary">
						Reinstalls the loader and Minecraft dependencies without deleting your content. This may
						resolve issues if your server is not starting correctly.
					</span>
					<div>
						<ButtonStyled color="orange">
							<button class="!shadow-none" :disabled="isInstalling" @click="repairModal?.show()">
								<HammerIcon class="size-5" />
								Repair
							</button>
						</ButtonStyled>
					</div>
				</div>
			</template>
		</template>
	</div>
	<ConfirmUnlinkModal ref="unlinkModal" server @unlink="handleUnlinkConfirm" />
	<ServerSetupModal
		ref="setupModal"
		@reinstall="emit('reinstall', $event)"
		@browse-modpacks="onBrowseModpacks"
	/>
	<PlatformChangeModpackVersionModal
		ref="modpackVersionModal"
		:project="modpackProjectForModal"
		:versions="versions ?? []"
		:current-version="modpackCurrentVersionForModal"
		:current-version-id="modpack?.spec.version_id"
		:server-status="server?.status"
		@reinstall="emit('reinstall')"
	/>
	<ConfirmRepairModal ref="repairModal" server @repair="handleRepair" />
	<ConfirmReinstallModal ref="reinstallModal" @reinstall="handleReinstallConfirm" />
</template>

<script setup lang="ts">
import type { Archon, LauncherMeta } from '@modrinth/api-client'
import {
	ArrowLeftRightIcon,
	ChevronRightIcon,
	CircleAlertIcon,
	DownloadIcon,
	EyeIcon,
	EyeOffIcon,
	HammerIcon,
	PencilIcon,
	RotateCounterClockwiseIcon,
	SaveIcon,
	SpinnerIcon,
	UnlinkIcon,
	XIcon,
} from '@modrinth/assets'
import {
	AutoLink,
	Avatar,
	ButtonStyled,
	Chips,
	Combobox,
	ConfirmReinstallModal,
	ConfirmRepairModal,
	ConfirmUnlinkModal,
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
	injectTags,
	ServerSetupModal,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'

import PlatformChangeModpackVersionModal from '~/components/ui/servers/PlatformChangeModpackVersionModal.vue'

const client = injectModrinthClient()
const { server, serverId, worldId } = injectModrinthServerContext()
const { addNotification } = injectNotificationManager()
const queryClient = useQueryClient()
const tags = injectTags()

const emit = defineEmits<{
	reinstall: [any?]
}>()

const isInstalling = computed(() => server.value?.status === 'installing')
const isEditing = ref(false)

const addonsQuery = useQuery({
	queryKey: computed(() => ['content', 'list', 'v1', serverId]),
	queryFn: () => client.archon.content_v1.getAddons(serverId, worldId.value!),
	enabled: computed(() => worldId.value !== null),
})

const modpack = computed(() => addonsQuery.data.value?.modpack ?? null)
const isLinked = computed(() => !!modpack.value)

function capitalize(str: string): string {
	return str.charAt(0).toUpperCase() + str.slice(1)
}

const installationInfo = computed(() => {
	const addons = addonsQuery.data.value
	const rawLoader = addons?.modloader ?? server.value?.loader ?? 'Unknown'
	const loader = capitalize(rawLoader)
	const gameVersion = addons?.game_version ?? server.value?.mc_version ?? 'Unknown'
	const loaderVersion = addons?.modloader_version ?? server.value?.loader_version ?? 'Unknown'

	const rows = [
		{ label: 'Platform', value: loader },
		{ label: 'Game version', value: gameVersion },
	]
	if (loader && loader !== 'Vanilla') {
		rows.push({ label: `${loader} version`, value: loaderVersion })
	}
	return rows
})

const versionsQuery = useQuery({
	queryKey: computed(() => [
		'content',
		'loader',
		'versions',
		modpack.value?.spec.project_id ?? null,
	]),
	queryFn: () => client.labrinth.versions_v2.getProjectVersions(modpack.value!.spec.project_id),
	enabled: computed(() => !!modpack.value?.spec.project_id),
})

const versions = computed(() => versionsQuery.data.value ?? [])

const modpackProjectForModal = computed(() => {
	if (!modpack.value) return null
	return {
		id: modpack.value.spec.project_id,
		title: modpack.value.title ?? modpack.value.spec.project_id,
	}
})

const modpackCurrentVersionForModal = computed(() => {
	if (!modpack.value) return null
	return { version_number: modpack.value.version_number ?? '' }
})

function onBrowseModpacks() {
	navigateTo({
		path: '/discover/modpacks',
		query: { sid: serverId, from: 'content', wid: worldId.value },
	})
}

const unlinkModal = ref<InstanceType<typeof ConfirmUnlinkModal>>()
const setupModal = ref<InstanceType<typeof ServerSetupModal>>()
const modpackVersionModal = ref()
const reinstallModal = ref<InstanceType<typeof ConfirmReinstallModal>>()
const repairModal = ref<InstanceType<typeof ConfirmRepairModal>>()

const availablePlatforms = ['vanilla', 'fabric', 'neoforge', 'forge', 'quilt', 'paper', 'purpur']

const selectedPlatform = ref(server.value?.loader?.toLowerCase() ?? 'vanilla')
const selectedGameVersion = ref(server.value?.mc_version ?? '')
const selectedLoaderVersion = ref<number>(0)
const showSnapshots = ref(false)
const isSaving = ref(false)

type LoaderVersionEntry = LauncherMeta.Manifest.v0.LoaderVersion

const modLoaders = ['fabric', 'forge', 'quilt', 'neoforge']

function toApiLoaderName(loader: string): string {
	return loader === 'neoforge' ? 'neo' : loader
}

const apiLoaderName = computed(() =>
	modLoaders.includes(selectedPlatform.value) ? toApiLoaderName(selectedPlatform.value) : null,
)

const manifestQuery = useQuery({
	queryKey: computed(() => ['loader-manifest', apiLoaderName.value] as const),
	queryFn: () => client.launchermeta.manifest_v0.getManifest(apiLoaderName.value!),
	enabled: computed(() => !!apiLoaderName.value),
	staleTime: 5 * 60 * 1000,
})

const paperBuildsQuery = useQuery({
	queryKey: computed(() => ['paper-builds', selectedGameVersion.value] as const),
	queryFn: () => client.paper.versions_v3.getBuilds(selectedGameVersion.value),
	enabled: computed(() => selectedPlatform.value === 'paper' && !!selectedGameVersion.value),
	staleTime: 5 * 60 * 1000,
})

const purpurBuildsQuery = useQuery({
	queryKey: computed(() => ['purpur-builds', selectedGameVersion.value] as const),
	queryFn: () => client.purpur.versions_v2.getBuilds(selectedGameVersion.value),
	enabled: computed(() => selectedPlatform.value === 'purpur' && !!selectedGameVersion.value),
	staleTime: 5 * 60 * 1000,
})

function getLoaderVersionsForGameVersion(
	loader: string,
	gameVersion: string,
): LoaderVersionEntry[] {
	if (loader === 'paper') {
		return (paperBuildsQuery.data.value?.builds ?? [])
			.toSorted((a, b) => b - a)
			.map((b) => ({ id: String(b), stable: true }))
	}
	if (loader === 'purpur') {
		return (purpurBuildsQuery.data.value?.builds.all ?? [])
			.toSorted((a, b) => parseInt(b) - parseInt(a))
			.map((b) => ({ id: b, stable: true }))
	}

	const manifest = manifestQuery.data.value?.gameVersions
	if (!manifest) return []

	const placeholder = manifest.find((x) => x.id === '${modrinth.gameVersion}')
	if (placeholder) return placeholder.loaders

	const entry = manifest.find((x) => x.id === gameVersion)
	return entry?.loaders ?? []
}

const gameVersionsForLoader = computed(() => {
	const versions = showSnapshots.value
		? tags.gameVersions.value
		: tags.gameVersions.value.filter((v) => v.version_type === 'release')

	if (selectedPlatform.value && selectedPlatform.value !== 'vanilla') {
		const manifest = manifestQuery.data.value?.gameVersions
		if (manifest) {
			const hasPlaceholder = manifest.some((x) => x.id === '${modrinth.gameVersion}')
			if (!hasPlaceholder) {
				const supportedVersions = new Set(
					manifest.filter((x) => x.loaders.length > 0).map((x) => x.id),
				)
				return versions.filter((v) => supportedVersions.has(v.version))
			}
		}
	}

	return versions
})

const hasSnapshots = computed(() =>
	gameVersionsForLoader.value.some((v) => v.version_type !== 'release'),
)

const gameVersionOptions = computed(() =>
	gameVersionsForLoader.value.map((v) => ({ value: v.version, label: v.version })),
)

const loaderVersionEntries = computed(() =>
	getLoaderVersionsForGameVersion(selectedPlatform.value, selectedGameVersion.value),
)

const loaderVersionOptions = computed(() =>
	loaderVersionEntries.value.map((v, index) => ({ value: index, label: v.id })),
)

const loaderVersionDisplayValue = computed(() => {
	const idx = selectedLoaderVersion.value
	return idx >= 0 && loaderVersionEntries.value[idx]
		? loaderVersionEntries.value[idx].id
		: 'Select version'
})

const formattedLoaderName = computed(() => {
	const name = selectedPlatform.value
	return name.charAt(0).toUpperCase() + name.slice(1)
})

const hasChanges = computed(() => {
	const currentLoader = server.value?.loader?.toLowerCase() ?? ''
	const currentGameVersion = server.value?.mc_version ?? ''
	const currentLoaderVersion = server.value?.loader_version ?? ''

	if (selectedPlatform.value !== currentLoader) return true
	if (selectedGameVersion.value !== currentGameVersion) return true
	if (
		selectedPlatform.value !== 'vanilla' &&
		loaderVersionEntries.value[selectedLoaderVersion.value]?.id !== currentLoaderVersion
	) {
		return true
	}
	return false
})

const isValid = computed(() => {
	if (!selectedGameVersion.value) return false
	if (selectedPlatform.value !== 'vanilla') {
		return selectedLoaderVersion.value >= 0 && loaderVersionEntries.value.length > 0
	}
	return true
})

watch(selectedPlatform, () => {
	selectedLoaderVersion.value = 0
})

watch(selectedGameVersion, () => {
	selectedLoaderVersion.value = 0
})

function toApiLoader(loader: string): Archon.Content.v1.Modloader {
	if (loader === 'neoforge') return 'neo_forge'
	return loader as Archon.Content.v1.Modloader
}

function cancelEditing() {
	selectedPlatform.value = server.value?.loader?.toLowerCase() ?? 'vanilla'
	selectedGameVersion.value = server.value?.mc_version ?? ''
	selectedLoaderVersion.value = 0
	isEditing.value = false
}

async function handleSave() {
	isSaving.value = true
	try {
		const loaderVersion = loaderVersionEntries.value[selectedLoaderVersion.value]?.id ?? ''

		const request: Archon.Content.v1.InstallWorldContent = {
			content_variant: 'bare',
			loader: toApiLoader(selectedPlatform.value),
			version: loaderVersion,
			game_version: selectedGameVersion.value || undefined,
			soft_override: true,
		}

		await client.archon.content_v1.installContent(serverId, worldId.value!, request)
		await queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] })
		isEditing.value = false
	} catch (err) {
		addNotification({
			type: 'error',
			text: err instanceof Error ? err.message : 'Failed to save installation settings',
		})
	} finally {
		isSaving.value = false
	}
}

async function handleRepair() {
	if (modpack.value) {
		try {
			await client.archon.content_v1.installContent(serverId, worldId.value!, {
				content_variant: 'modpack',
				spec: {
					platform: 'modrinth',
					project_id: modpack.value.spec.project_id,
					version_id: modpack.value.spec.version_id,
				},
				soft_override: true,
			})
			await queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] })
		} catch (err) {
			addNotification({
				type: 'error',
				text: err instanceof Error ? err.message : 'Failed to repair server',
			})
		}
	} else {
		const addons = addonsQuery.data.value
		const currentLoader = addons?.modloader ?? server.value?.loader
		const currentGameVersion = addons?.game_version ?? server.value?.mc_version
		const currentLoaderVersion = addons?.modloader_version ?? server.value?.loader_version
		if (!currentLoader || !currentGameVersion) return
		try {
			await client.archon.content_v1.installContent(serverId, worldId.value!, {
				content_variant: 'bare',
				loader: toApiLoader(currentLoader),
				version: currentLoaderVersion ?? '',
				game_version: currentGameVersion,
				soft_override: true,
			})
			await queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] })
		} catch (err) {
			addNotification({
				type: 'error',
				text: err instanceof Error ? err.message : 'Failed to repair server',
			})
		}
	}
}

async function handleReinstallConfirm() {
	if (!modpack.value) return
	try {
		await client.archon.content_v1.installContent(serverId, worldId.value!, {
			content_variant: 'modpack',
			spec: {
				platform: 'modrinth',
				project_id: modpack.value.spec.project_id,
				version_id: modpack.value.spec.version_id,
			},
			soft_override: false,
		})
		await queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] })
		emit('reinstall')
	} catch (err) {
		addNotification({
			type: 'error',
			text: err instanceof Error ? err.message : 'Failed to reinstall modpack',
		})
	}
}

async function handleUnlinkConfirm() {
	try {
		await client.archon.content_v1.unlinkModpack(serverId, worldId.value!)
		await Promise.all([
			queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] }),
			queryClient.invalidateQueries({ queryKey: ['content', 'list', 'v1', serverId] }),
		])
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
			selectedPlatform.value = server.value?.loader?.toLowerCase() ?? 'vanilla'
			selectedGameVersion.value = server.value?.mc_version ?? ''
			selectedLoaderVersion.value = 0
			isEditing.value = false

			await Promise.all([
				queryClient.invalidateQueries({ queryKey: ['content', 'list', 'v1', serverId] }),
				queryClient.invalidateQueries({ queryKey: ['content', 'loader', 'versions'] }),
				queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] }),
			])
		}
	},
)
</script>
