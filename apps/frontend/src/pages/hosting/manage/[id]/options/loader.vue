<template>
	<div class="flex flex-col gap-6 rounded-2xl bg-surface-3 p-6">
		<InstallationSettingsLayout>
			<template #linked-extra>
				<div class="flex flex-col gap-2.5">
					<span class="text-lg font-semibold text-contrast">{{
						formatMessage(messages.resetServerTitle)
					}}</span>
					<span class="text-primary">
						{{ formatMessage(messages.resetServerDescription) }}
					</span>
					<div>
						<ButtonStyled color="red">
							<button
								class="!shadow-none"
								:disabled="isInstalling"
								@click="setupModal?.show()"
							>
								<RotateCounterClockwiseIcon class="size-5" />
								{{ formatMessage(commonMessages.resetServerButton) }}
							</button>
						</ButtonStyled>
					</div>
				</div>
			</template>

			<template #unlinked-extra-buttons>
				<ButtonStyled>
					<button
						class="!shadow-none"
						:disabled="isInstalling"
						@click="setupModal?.show()"
					>
						{{ formatMessage(commonMessages.resetServerButton) }}
						<ChevronRightIcon class="size-5" />
					</button>
				</ButtonStyled>
			</template>

			<template #extra-modals>
				<ServerSetupModal
					ref="setupModal"
					@reinstall="emit('reinstall', $event)"
					@browse-modpacks="onBrowseModpacks"
				/>
			</template>
		</InstallationSettingsLayout>
	</div>
</template>

<script setup lang="ts">
import type { Archon, Labrinth, LauncherMeta } from '@modrinth/api-client'
import { ChevronRightIcon, RotateCounterClockwiseIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	commonMessages,
	defineMessages,
	formatLoaderLabel,
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
	injectTags,
	InstallationSettingsLayout,
	provideInstallationSettings,
	ServerSetupModal,
	useVIntl,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'

const client = injectModrinthClient()
const { server, serverId, worldId } = injectModrinthServerContext()
const { addNotification } = injectNotificationManager()
const queryClient = useQueryClient()
const tags = injectTags()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	resetServerTitle: {
		id: 'hosting.loader.reset-server',
		defaultMessage: 'Reset server',
	},
	resetServerDescription: {
		id: 'hosting.loader.reset-server-description',
		defaultMessage:
			'Removes all data on your server, including your worlds, mods, and configuration files. Backups will remain and can be restored.',
	},
	loaderVersionLabel: {
		id: 'hosting.loader.loader-version',
		defaultMessage: '{loader} version',
	},
	failedToLoadVersions: {
		id: 'hosting.loader.failed-to-load-versions',
		defaultMessage: 'Failed to load versions',
	},
	failedToChangeVersion: {
		id: 'hosting.loader.failed-to-change-version',
		defaultMessage: 'Failed to change modpack version',
	},
	failedToSaveSettings: {
		id: 'hosting.loader.failed-to-save-settings',
		defaultMessage: 'Failed to save installation settings',
	},
	failedToRepair: {
		id: 'hosting.loader.failed-to-repair',
		defaultMessage: 'Failed to repair server',
	},
	failedToReinstall: {
		id: 'hosting.loader.failed-to-reinstall',
		defaultMessage: 'Failed to reinstall modpack',
	},
	failedToUnlink: {
		id: 'hosting.loader.failed-to-unlink',
		defaultMessage: 'Failed to unlink modpack',
	},
})

const emit = defineEmits<{
	reinstall: [any?]
}>()

const isInstalling = computed(() => server.value?.status === 'installing')
const setupModal = ref<InstanceType<typeof ServerSetupModal>>()

const addonsQuery = useQuery({
	queryKey: computed(() => ['content', 'list', 'v1', serverId]),
	queryFn: () =>
		client.archon.content_v1.getAddons(serverId, worldId.value!, { from_modpack: false }),
	enabled: computed(() => worldId.value !== null),
})

const modpack = computed(() => addonsQuery.data.value?.modpack ?? null)

const modpackVersionsQuery = useQuery({
	queryKey: computed(() => ['labrinth', 'versions', 'v2', modpack.value?.spec.project_id]),
	queryFn: () =>
		client.labrinth.versions_v2.getProjectVersions(modpack.value!.spec.project_id, {
			include_changelog: false,
		}),
	enabled: computed(() => !!modpack.value?.spec.project_id),
})

const editingPlatform = ref(server.value?.loader?.toLowerCase() ?? 'vanilla')
const editingGameVersion = ref(server.value?.mc_version ?? '')

const modLoaders = ['fabric', 'forge', 'quilt', 'neoforge']

function toApiLoaderName(loader: string): string {
	return loader === 'neoforge' ? 'neo' : loader
}

const apiLoaderName = computed(() =>
	modLoaders.includes(editingPlatform.value) ? toApiLoaderName(editingPlatform.value) : null,
)

const manifestQuery = useQuery({
	queryKey: computed(() => ['loader-manifest', apiLoaderName.value] as const),
	queryFn: () => client.launchermeta.manifest_v0.getManifest(apiLoaderName.value!),
	enabled: computed(() => !!apiLoaderName.value),
	staleTime: 5 * 60 * 1000,
})

const paperBuildsQuery = useQuery({
	queryKey: computed(() => ['paper-builds', editingGameVersion.value] as const),
	queryFn: () => client.paper.versions_v3.getBuilds(editingGameVersion.value),
	enabled: computed(() => editingPlatform.value === 'paper' && !!editingGameVersion.value),
	staleTime: 5 * 60 * 1000,
})

const purpurBuildsQuery = useQuery({
	queryKey: computed(() => ['purpur-builds', editingGameVersion.value] as const),
	queryFn: () => client.purpur.versions_v2.getBuilds(editingGameVersion.value),
	enabled: computed(() => editingPlatform.value === 'purpur' && !!editingGameVersion.value),
	staleTime: 5 * 60 * 1000,
})

type LoaderVersionEntry = LauncherMeta.Manifest.v0.LoaderVersion

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

function toApiLoader(loader: string): Archon.Content.v1.Modloader {
	if (loader === 'neoforge') return 'neo_forge'
	return loader as Archon.Content.v1.Modloader
}

provideInstallationSettings({
	loading: computed(() => !server.value || addonsQuery.isLoading.value),
	installationInfo: computed(() => {
		const addons = addonsQuery.data.value
		const unknownStr = formatMessage(commonMessages.unknownLabel)
		const rawLoader = addons?.modloader ?? server.value?.loader ?? unknownStr
		const loader = formatLoaderLabel(rawLoader)
		const gameVersion = addons?.game_version ?? server.value?.mc_version ?? unknownStr
		const loaderVersion =
			addons?.modloader_version ?? server.value?.loader_version ?? unknownStr

		const rows = [
			{ label: formatMessage(commonMessages.platformLabel), value: loader },
			{ label: formatMessage(commonMessages.gameVersionLabel), value: gameVersion },
		]
		if (loader && loader !== 'Vanilla') {
			rows.push({
				label: formatMessage(messages.loaderVersionLabel, { loader }),
				value: loaderVersion,
			})
		}
		return rows
	}),
	isLinked: computed(() => !!modpack.value),
	isBusy: isInstalling,
	modpack: computed(() => {
		if (!modpack.value) return null
		return {
			iconUrl: modpack.value.icon_url,
			title: modpack.value.title ?? modpack.value.spec.project_id,
			link: `/project/${modpack.value.spec.project_id}`,
			versionNumber: modpack.value.version_number,
			owner: modpack.value.owner
				? {
						id: modpack.value.owner.id,
						name: modpack.value.owner.name,
						iconUrl: modpack.value.owner.icon_url,
						type: modpack.value.owner.type as 'user' | 'organization',
					}
				: undefined,
		}
	}),
	currentPlatform: computed(() => server.value?.loader?.toLowerCase() ?? 'vanilla'),
	currentGameVersion: computed(() => server.value?.mc_version ?? ''),
	currentLoaderVersion: computed(() => server.value?.loader_version ?? ''),
	availablePlatforms: ['vanilla', 'fabric', 'neoforge', 'forge', 'quilt', 'paper', 'purpur'],

	editingPlatformRef: editingPlatform,
	editingGameVersionRef: editingGameVersion,

	resolveGameVersions(loader, showSnapshots) {
		const versions = showSnapshots
			? tags.gameVersions.value
			: tags.gameVersions.value.filter((v) => v.version_type === 'release')

		if (loader && loader !== 'vanilla' && !['paper', 'purpur'].includes(loader)) {
			const manifest = manifestQuery.data.value?.gameVersions
			if (manifest) {
				const hasPlaceholder = manifest.some(
					(x) => x.id === '${modrinth.gameVersion}',
				)
				if (!hasPlaceholder) {
					const supportedVersions = new Set(
						manifest.filter((x) => x.loaders.length > 0).map((x) => x.id),
					)
					return versions
						.filter((v) => supportedVersions.has(v.version))
						.map((v) => ({ value: v.version, label: v.version }))
				}
			}
		}

		return versions.map((v) => ({ value: v.version, label: v.version }))
	},

	resolveLoaderVersions(loader, gameVersion) {
		if (loader === 'vanilla' || !gameVersion) return []
		return getLoaderVersionsForGameVersion(loader, gameVersion)
	},

	resolveHasSnapshots(loader) {
		if (loader === 'vanilla' || ['paper', 'purpur'].includes(loader)) {
			return tags.gameVersions.value.some((v) => v.version_type !== 'release')
		}
		const manifest = manifestQuery.data.value?.gameVersions
		if (!manifest) return false
		const hasPlaceholder = manifest.some((x) => x.id === '${modrinth.gameVersion}')
		if (hasPlaceholder) {
			return tags.gameVersions.value.some((v) => v.version_type !== 'release')
		}
		const supportedVersions = new Set(
			manifest.filter((x) => x.loaders.length > 0).map((x) => x.id),
		)
		const supported = tags.gameVersions.value.filter((v) =>
			supportedVersions.has(v.version),
		)
		return supported.some((v) => v.version_type !== 'release')
	},

	async save(platform, gameVersion, loaderVersionId) {
		try {
			const request: Archon.Content.v1.InstallWorldContent = {
				content_variant: 'bare',
				loader: toApiLoader(platform),
				version: loaderVersionId ?? '',
				game_version: gameVersion || undefined,
				soft_override: true,
			}

			await client.archon.content_v1.installContent(serverId, worldId.value!, request)
			await queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] })
		} catch (err) {
			addNotification({
				type: 'error',
				text:
					err instanceof Error
						? err.message
						: formatMessage(messages.failedToSaveSettings),
			})
			throw err
		}
	},

	async repair() {
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
				await queryClient.invalidateQueries({
					queryKey: ['servers', 'detail', serverId],
				})
			} catch (err) {
				addNotification({
					type: 'error',
					text:
						err instanceof Error
							? err.message
							: formatMessage(messages.failedToRepair),
				})
			}
		} else {
			const addons = addonsQuery.data.value
			const currentLoader = addons?.modloader ?? server.value?.loader
			const currentGameVersion = addons?.game_version ?? server.value?.mc_version
			const currentLoaderVersion =
				addons?.modloader_version ?? server.value?.loader_version
			if (!currentLoader || !currentGameVersion) return
			try {
				await client.archon.content_v1.installContent(serverId, worldId.value!, {
					content_variant: 'bare',
					loader: toApiLoader(currentLoader),
					version: currentLoaderVersion ?? '',
					game_version: currentGameVersion,
					soft_override: true,
				})
				await queryClient.invalidateQueries({
					queryKey: ['servers', 'detail', serverId],
				})
			} catch (err) {
				addNotification({
					type: 'error',
					text:
						err instanceof Error
							? err.message
							: formatMessage(messages.failedToRepair),
				})
			}
		}
	},

	async reinstallModpack() {
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
			await queryClient.invalidateQueries({
				queryKey: ['servers', 'detail', serverId],
			})
			emit('reinstall')
		} catch (err) {
			addNotification({
				type: 'error',
				text:
					err instanceof Error
						? err.message
						: formatMessage(messages.failedToReinstall),
			})
		}
	},

	async unlinkModpack() {
		const previousData = addonsQuery.data.value
		if (previousData) {
			queryClient.setQueryData(['content', 'list', 'v1', serverId], {
				...previousData,
				modpack: null,
			})
		}

		try {
			await client.archon.content_v1.unlinkModpack(serverId, worldId.value!)
		} catch (err) {
			if (previousData) {
				queryClient.setQueryData(
					['content', 'list', 'v1', serverId],
					previousData,
				)
			}
			addNotification({
				type: 'error',
				text:
					err instanceof Error
						? err.message
						: formatMessage(messages.failedToUnlink),
			})
		} finally {
			await Promise.all([
				queryClient.invalidateQueries({
					queryKey: ['servers', 'detail', serverId],
				}),
				queryClient.invalidateQueries({
					queryKey: ['content', 'list', 'v1', serverId],
				}),
			])
		}
	},

	getCachedModpackVersions: () => modpackVersionsQuery.data.value ?? null,

	async fetchModpackVersions() {
		try {
			return await client.labrinth.versions_v2.getProjectVersions(
				modpack.value!.spec.project_id,
				{ include_changelog: false },
			)
		} catch (err) {
			addNotification({
				type: 'error',
				text:
					err instanceof Error
						? err.message
						: formatMessage(messages.failedToLoadVersions),
			})
			throw err
		}
	},

	async getVersionChangelog(versionId) {
		try {
			return await client.labrinth.versions_v2.getVersion(versionId)
		} catch {
			return null
		}
	},

	async onModpackVersionConfirm(version) {
		if (!modpack.value) return
		try {
			await client.archon.content_v1.installContent(serverId, worldId.value!, {
				content_variant: 'modpack',
				spec: {
					platform: 'modrinth',
					project_id: modpack.value.spec.project_id,
					version_id: version.id,
				},
				soft_override: true,
			})
			await queryClient.invalidateQueries({
				queryKey: ['servers', 'detail', serverId],
			})
			emit('reinstall')
		} catch (err) {
			addNotification({
				type: 'error',
				text:
					err instanceof Error
						? err.message
						: formatMessage(messages.failedToChangeVersion),
			})
		}
	},

	updaterModalProps: computed(() => ({
		isApp: false,
		currentVersionId: modpack.value?.spec.version_id ?? '',
		projectIconUrl: modpack.value?.icon_url ?? undefined,
		projectName:
			modpack.value?.title ??
			modpack.value?.spec.project_id ??
			formatMessage(commonMessages.modpackLabel),
		currentGameVersion:
			addonsQuery.data.value?.game_version ?? server.value?.mc_version ?? '',
		currentLoader: addonsQuery.data.value?.modloader ?? server.value?.loader ?? '',
	})),

	isServer: true,
	isApp: false,
})

watch(
	() => server.value?.status,
	async (newStatus, oldStatus) => {
		if (oldStatus === 'installing' && newStatus === 'available') {
			editingPlatform.value = server.value?.loader?.toLowerCase() ?? 'vanilla'
			editingGameVersion.value = server.value?.mc_version ?? ''

			await Promise.all([
				queryClient.invalidateQueries({
					queryKey: ['content', 'list', 'v1', serverId],
				}),
				queryClient.invalidateQueries({
					queryKey: ['content', 'loader', 'versions'],
				}),
				queryClient.invalidateQueries({
					queryKey: ['servers', 'detail', serverId],
				}),
			])
		}
	},
)

function onBrowseModpacks() {
	navigateTo({
		path: '/discover/modpacks',
		query: { sid: serverId, from: 'reset-server', wid: worldId.value },
	})
}
</script>
