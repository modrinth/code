<template>
	<div class="max-h-fit rounded-2xl bg-surface-3 p-6">
		<template v-if="server">
			<InstallationSettingsLayout>
				<template #extra>
					<!-- Reset server section -->
					<div class="flex flex-col gap-2.5">
						<span class="text-lg font-semibold text-contrast">Reset server</span>
						<span class="m-0 text-primary">
							Removes all data on your server, including your worlds, mods, and configuration files.
							Backups will remain and can be restored.
						</span>
						<div>
							<ButtonStyled color="red">
								<button :disabled="isInstalling" @click="setupModal?.show()" class="!shadow-none">
									<RotateCounterClockwiseIcon class="size-5" />
									Reset server
								</button>
							</ButtonStyled>
						</div>
					</div>
				</template>

				<template #unlinked-extra>
					<div class="my-2 flex items-center gap-2">
						<Toggle v-model="eraseAllData" small />
						<span class="font-semibold text-contrast">Erase all data</span>
					</div>
				</template>

				<template #save-button>
					<div class="flex flex-wrap gap-2">
						<ButtonStyled :color="eraseAllData ? 'red' : 'brand'">
							<button
								class="max-w-fit !shadow-none"
								:disabled="!isValid || (!hasChanges && !eraseAllData) || isSaving || isInstalling"
								@click="handleSave"
							>
								<SpinnerIcon v-if="isSaving" class="animate-spin" />
								<SaveIcon v-else />
								{{ isSaving ? 'Saving...' : eraseAllData ? 'Erase and save' : 'Save' }}
							</button>
						</ButtonStyled>
						<ButtonStyled>
							<button :disabled="!hasChanges" @click="resetToCurrent" class="!shadow-none">
								<UndoIcon />
								Reset to current
							</button>
						</ButtonStyled>
					</div>
				</template>
			</InstallationSettingsLayout>

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
		</template>
	</div>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { RotateCounterClockwiseIcon, SaveIcon, SpinnerIcon, UndoIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	ConfirmUnlinkModal,
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
	injectTags,
	InstallationSettingsLayout,
	provideInstallationSettings,
	ServerSetupModal,
	Toggle,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'

import PlatformChangeModpackVersionModal from '~/components/ui/servers/PlatformChangeModpackVersionModal.vue'
import { useServerProject } from '~/composables/servers/use-server-project.ts'

const client = injectModrinthClient()
const { server, serverId, worldId } = injectModrinthServerContext()
const { data: serverProject } = useServerProject(computed(() => server.value?.upstream ?? null))
const { addNotification } = injectNotificationManager()
const queryClient = useQueryClient()
const tags = injectTags()

const emit = defineEmits<{
	reinstall: [any?]
}>()

const isInstalling = computed(() => server.value?.status === 'installing')

const unlinkModal = ref<InstanceType<typeof ConfirmUnlinkModal>>()
const setupModal = ref<InstanceType<typeof ServerSetupModal>>()
const modpackVersionModal = ref()

// --- Linked modpack data ---

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

// --- Unlinked state ---

const availablePlatforms = ['vanilla', 'fabric', 'neoforge', 'forge', 'quilt', 'paper', 'purpur']

const selectedPlatform = ref(server.value?.loader?.toLowerCase() ?? 'vanilla')
const selectedGameVersion = ref(server.value?.mc_version ?? '')
const selectedLoaderVersion = ref<number>(0)
const showSnapshots = ref(false)
const eraseAllData = ref(false)
const isSaving = ref(false)

// Loader version manifest cache
interface LoaderVersionEntry {
	id: string
	stable: boolean
}

const loaderVersionsCache = ref<Record<string, { id: string; loaders: LoaderVersionEntry[] }[]>>({})
const paperVersions = ref<Record<string, number[]>>({})
const purpurVersions = ref<Record<string, string[]>>({})

async function fetchLoaderManifest(loader: string) {
	let apiLoader = loader
	if (apiLoader === 'neoforge') apiLoader = 'neo'

	if (loaderVersionsCache.value[apiLoader]) return

	try {
		const res = await fetch(`https://launcher-meta.modrinth.com/${apiLoader}/v0/manifest.json`)
		const data = (await res.json()) as {
			gameVersions: { id: string; loaders: LoaderVersionEntry[] }[]
		}
		loaderVersionsCache.value[apiLoader] = data.gameVersions
	} catch {
		loaderVersionsCache.value[apiLoader] = []
	}
}

async function fetchPaperVersions(mcVersion: string) {
	if (paperVersions.value[mcVersion]) return
	try {
		const res = await fetch(`https://fill.papermc.io/v3/projects/paper/versions/${mcVersion}`)
		const data = (await res.json()) as { builds: number[] }
		paperVersions.value[mcVersion] = data.builds.sort((a, b) => b - a)
	} catch {
		paperVersions.value[mcVersion] = []
	}
}

async function fetchPurpurVersions(mcVersion: string) {
	if (purpurVersions.value[mcVersion]) return
	try {
		const res = await fetch(`https://api.purpurmc.org/v2/purpur/${mcVersion}`)
		const data = (await res.json()) as { builds: { all: string[] } }
		purpurVersions.value[mcVersion] = data.builds.all.sort((a, b) => parseInt(b) - parseInt(a))
	} catch {
		purpurVersions.value[mcVersion] = []
	}
}

function getLoaderVersionsForGameVersion(
	loader: string,
	gameVersion: string,
): LoaderVersionEntry[] {
	let apiLoader = loader
	if (apiLoader === 'neoforge') apiLoader = 'neo'

	if (loader === 'paper') {
		return (paperVersions.value[gameVersion] ?? []).map((b) => ({
			id: String(b),
			stable: true,
		}))
	}
	if (loader === 'purpur') {
		return (purpurVersions.value[gameVersion] ?? []).map((b) => ({
			id: b,
			stable: true,
		}))
	}

	const manifest = loaderVersionsCache.value[apiLoader]
	if (!manifest) return []

	const placeholder = manifest.find((x) => x.id === '${modrinth.gameVersion}')
	if (placeholder) return placeholder.loaders

	const entry = manifest.find((x) => x.id === gameVersion)
	return entry?.loaders ?? []
}

// Game version options filtered by loader support and snapshot toggle
const gameVersionsForLoader = computed(() => {
	const versions = showSnapshots.value
		? tags.gameVersions.value
		: tags.gameVersions.value.filter((v) => v.version_type === 'release')

	if (selectedPlatform.value && selectedPlatform.value !== 'vanilla') {
		let apiLoader = selectedPlatform.value
		if (apiLoader === 'neoforge') apiLoader = 'neo'

		const manifest = loaderVersionsCache.value[apiLoader]
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

// Fetch manifest when loader changes
watch(
	selectedPlatform,
	async (loader) => {
		selectedLoaderVersion.value = 0
		if (!loader || loader === 'vanilla') return
		if (loader === 'paper') {
			if (selectedGameVersion.value) await fetchPaperVersions(selectedGameVersion.value)
		} else if (loader === 'purpur') {
			if (selectedGameVersion.value) await fetchPurpurVersions(selectedGameVersion.value)
		} else {
			await fetchLoaderManifest(loader)
		}
	},
	{ immediate: true },
)

// Fetch paper/purpur versions when game version changes
watch(selectedGameVersion, async (gv) => {
	selectedLoaderVersion.value = 0
	if (!gv) return
	if (selectedPlatform.value === 'paper') {
		await fetchPaperVersions(gv)
	} else if (selectedPlatform.value === 'purpur') {
		await fetchPurpurVersions(gv)
	}
})

/** Map UI loader names to API Modloader values */
function toApiLoader(loader: string): Archon.Content.v1.Modloader {
	if (loader === 'neoforge') return 'neo_forge'
	return loader as Archon.Content.v1.Modloader
}

function resetToCurrent() {
	selectedPlatform.value = server.value?.loader?.toLowerCase() ?? 'vanilla'
	selectedGameVersion.value = server.value?.mc_version ?? ''
	selectedLoaderVersion.value = 0
	eraseAllData.value = false
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
			soft_override: !eraseAllData.value,
		}

		await client.archon.content_v1.installContent(serverId, request, worldId.value ?? undefined)
		server.value.status = 'installing'
		eraseAllData.value = false
	} catch (err) {
		addNotification({
			type: 'error',
			text: err instanceof Error ? err.message : 'Failed to save installation settings',
		})
	} finally {
		isSaving.value = false
	}
}

// --- Context ---

provideInstallationSettings({
	isLinked: computed(() => !!server.value?.upstream),
	modpack: computed(() =>
		serverProject.value
			? {
					title: serverProject.value.title ?? 'Unknown modpack',
					iconUrl: serverProject.value.icon_url ?? undefined,
					projectLink: `/project/${serverProject.value.slug ?? serverProject.value.id}`,
					versionName: (currentVersion.value as any)?.version_number ?? undefined,
					versionLink: currentVersion.value
						? `/project/${serverProject.value.slug ?? serverProject.value.id}/version/${(currentVersion.value as any).id}`
						: undefined,
				}
			: null,
	),
	installationInfo: computed(() => {
		const rows = [
			{ label: 'Platform', value: server.value?.loader ?? 'Unknown' },
			{ label: 'Game version', value: server.value?.mc_version ?? 'Unknown' },
		]
		if (server.value?.loader && server.value.loader !== 'Vanilla') {
			rows.push({
				label: `${server.value.loader} version`,
				value: server.value?.loader_version ?? 'Unknown',
			})
		}
		return rows
	}),
	isBusy: isInstalling,
	changeVersion: () => modpackVersionModal.value?.show(),
	unlink: () => unlinkModal.value?.show(),

	// Unlinked state
	platforms: computed(() => availablePlatforms),
	selectedPlatform,
	gameVersionOptions,
	selectedGameVersion,
	loaderVersionOptions,
	selectedLoaderVersion,
	loaderVersionDisplayValue,
	formattedLoaderName,
	hasChanges,
	isValid,
	isSaving,
	save: handleSave,
	showSnapshots,
	hasSnapshots,
})

// --- Linked state actions ---

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
			// Update unlinked state refs to reflect new server state
			selectedPlatform.value = server.value?.loader?.toLowerCase() ?? 'vanilla'
			selectedGameVersion.value = server.value?.mc_version ?? ''
			selectedLoaderVersion.value = 0

			await Promise.all([
				refreshVersions(),
				refreshCurrentVersion(),
				queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] }),
			])
		}
	},
)
</script>
