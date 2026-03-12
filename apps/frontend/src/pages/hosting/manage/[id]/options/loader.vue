<template>
	<div class="flex flex-col gap-6 rounded-2xl bg-surface-3 p-6">
		<InstallationSettingsLayout ref="installationSettingsLayout">
			<template #extra>
				<div class="flex flex-col gap-2.5">
					<span class="text-lg font-semibold text-contrast">{{
						formatMessage(messages.resetServerTitle)
					}}</span>
					<span class="text-primary">
						{{ formatMessage(messages.resetServerDescription) }}
					</span>
					<div>
						<ButtonStyled color="red">
							<button class="!shadow-none" :disabled="isInstalling" @click="setupModal?.show()">
								<RotateCounterClockwiseIcon class="size-5" />
								{{ formatMessage(commonMessages.resetServerButton) }}
							</button>
						</ButtonStyled>
					</div>
				</div>
			</template>

			<template #extra-modals>
				<ServerSetupModal
					ref="setupModal"
					@reinstall="
						installationSettingsLayout?.cancelEditing();
						emit('reinstall', $event)
					"
					@browse-modpacks="onBrowseModpacks"
				/>
			</template>
		</InstallationSettingsLayout>
	</div>
</template>

<script setup lang="ts">
import type { Archon, LauncherMeta } from '@modrinth/api-client'
import { RotateCounterClockwiseIcon } from '@modrinth/assets'
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
	useDebugLogger,
	useVIntl,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'

const debug = useDebugLogger('LoaderPage')
const client = injectModrinthClient()
const { server, serverId, worldId, isSyncingContent, busyReasons } = injectModrinthServerContext()
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
		defaultMessage: '{loader, select, null {Loader} other {{loader}}} version',
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
	repairStartedTitle: {
		id: 'hosting.loader.repair-started-title',
		defaultMessage: 'Repair completed',
	},
	repairStartedText: {
		id: 'hosting.loader.repair-started-text',
		defaultMessage: 'Your server installation has been repaired.',
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
	'reinstall-failed': []
}>()

const isInstalling = computed(() => {
	const val = server.value?.status === 'installing' || isSyncingContent.value || busyReasons.value.length > 0
	debug(
		'isInstalling:',
		val,
		'server.status:',
		server.value?.status,
		'isSyncingContent:',
		isSyncingContent.value,
	)
	return val
})
const installationSettingsLayout = ref<InstanceType<typeof InstallationSettingsLayout>>()
const setupModal = ref<InstanceType<typeof ServerSetupModal>>()

async function invalidateServerState() {
	debug('invalidateServerState: starting')
	await Promise.all([
		queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] }),
		queryClient.invalidateQueries({ queryKey: ['content', 'list', 'v1', serverId] }),
	])
	debug('invalidateServerState: complete')
}

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
		const rawLoader = addons?.modloader ?? server.value?.loader ?? null
		const loader = rawLoader ? formatLoaderLabel(rawLoader) : null
		const gameVersion = addons?.game_version ?? server.value?.mc_version ?? null
		const loaderVersion = addons?.modloader_version ?? server.value?.loader_version ?? null

		debug('installationInfo computed:', {
			'addons?.modloader': addons?.modloader,
			'server.loader': server.value?.loader,
			rawLoader,
			loader,
			'addons?.game_version': addons?.game_version,
			'server.mc_version': server.value?.mc_version,
			gameVersion,
			'addons?.modloader_version': addons?.modloader_version,
			'server.loader_version': server.value?.loader_version,
			loaderVersion,
			'addonsQuery.isLoading': addonsQuery.isLoading.value,
			'addonsQuery.isFetching': addonsQuery.isFetching.value,
		})

		const rows = [
			{ label: formatMessage(commonMessages.platformLabel), value: loader },
			{ label: formatMessage(commonMessages.gameVersionLabel), value: gameVersion },
		]
		if (loader !== 'Vanilla') {
			rows.push({
				label: formatMessage(messages.loaderVersionLabel, { loader: loader ?? 'null' }),
				value: loaderVersion,
			})
		}
		return rows
	}),
	isLinked: computed(() => {
		const val = !!modpack.value
		debug('isLinked:', val, 'modpack:', modpack.value?.spec?.project_id)
		return val
	}),
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
				const hasPlaceholder = manifest.some((x) => x.id === '${modrinth.gameVersion}')
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
		const supportedVersions = new Set(manifest.filter((x) => x.loaders.length > 0).map((x) => x.id))
		const supported = tags.gameVersions.value.filter((v) => supportedVersions.has(v.version))
		return supported.some((v) => v.version_type !== 'release')
	},

	async save(platform, gameVersion, loaderVersionId) {
		debug('save: called with', { platform, gameVersion, loaderVersionId })
		const currentPlatform = server.value?.loader?.toLowerCase() ?? 'vanilla'
		const platformChanged = platform !== currentPlatform

		debug('save: emitting reinstall before API call')
		emit('reinstall', platformChanged
			? { loader: platform, lVersion: loaderVersionId, mVersion: gameVersion }
			: { mVersion: gameVersion })
		try {
			if (platformChanged) {
				const request: Archon.Content.v1.InstallWorldContent = {
					content_variant: 'bare',
					loader: toApiLoader(platform),
					version: loaderVersionId ?? '',
					game_version: gameVersion || undefined,
					soft_override: true,
				}
				debug('save: platform changed, calling installContent', request)
				await client.archon.content_v1.installContent(serverId, worldId.value!, request)
			} else {
				debug('save: game version only, calling applyGameVersionUpdate', gameVersion)
				await client.archon.content_v1.applyGameVersionUpdate(serverId, worldId.value!, gameVersion)
			}
			debug('save: succeeded, invalidating')
			invalidateServerState()
		} catch (err) {
			debug('save: failed, emitting reinstall-failed', err)
			emit('reinstall-failed')
			addNotification({
				type: 'error',
				text: err instanceof Error ? err.message : formatMessage(messages.failedToSaveSettings),
			})
			throw err
		}
	},

	async repair() {
		debug('repair: called')
		try {
			await client.archon.content_v1.repair(serverId, worldId.value!)
			debug('repair: API succeeded, invalidating')
			await invalidateServerState()
			addNotification({
				type: 'success',
				title: formatMessage(messages.repairStartedTitle),
				text: formatMessage(messages.repairStartedText),
			})
		} catch (err) {
			debug('repair: failed', err)
			addNotification({
				type: 'error',
				text: err instanceof Error ? err.message : formatMessage(messages.failedToRepair),
			})
		}
	},

	async reinstallModpack() {
		if (!modpack.value) return
		debug(
			'reinstallModpack: called, project:',
			modpack.value.spec.project_id,
			'version:',
			modpack.value.spec.version_id,
		)
		debug('reinstallModpack: emitting reinstall before API call')
		emit('reinstall')
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
			debug('reinstallModpack: installContent succeeded, invalidating')
			invalidateServerState()
		} catch (err) {
			debug('reinstallModpack: failed, emitting reinstall-failed', err)
			emit('reinstall-failed')
			addNotification({
				type: 'error',
				text: err instanceof Error ? err.message : formatMessage(messages.failedToReinstall),
			})
		}
	},

	async unlinkModpack() {
		debug('unlinkModpack: called')
		const previousData = addonsQuery.data.value
		if (previousData) {
			debug('unlinkModpack: optimistically removing modpack from cache')
			queryClient.setQueryData(['content', 'list', 'v1', serverId], {
				...previousData,
				modpack: null,
			})
		}

		try {
			await client.archon.content_v1.unlinkModpack(serverId, worldId.value!)
			debug('unlinkModpack: API succeeded')
		} catch (err) {
			debug('unlinkModpack: failed, reverting cache', err)
			if (previousData) {
				queryClient.setQueryData(['content', 'list', 'v1', serverId], previousData)
			}
			addNotification({
				type: 'error',
				text: err instanceof Error ? err.message : formatMessage(messages.failedToUnlink),
			})
		} finally {
			debug('unlinkModpack: invalidating queries')
			await Promise.all([
				queryClient.invalidateQueries({
					queryKey: ['servers', 'detail', serverId],
				}),
				queryClient.invalidateQueries({
					queryKey: ['content', 'list', 'v1', serverId],
				}),
			])
			debug('unlinkModpack: invalidation complete')
		}
	},

	getCachedModpackVersions: () => modpackVersionsQuery.data.value ?? null,

	async fetchModpackVersions() {
		debug('fetchModpackVersions: called, project:', modpack.value?.spec.project_id)
		try {
			const versions = await client.labrinth.versions_v2.getProjectVersions(
				modpack.value!.spec.project_id,
				{
					include_changelog: false,
				},
			)
			debug('fetchModpackVersions: got', versions.length, 'versions')
			return versions
		} catch (err) {
			debug('fetchModpackVersions: failed', err)
			addNotification({
				type: 'error',
				text: err instanceof Error ? err.message : formatMessage(messages.failedToLoadVersions),
			})
			throw err
		}
	},

	async getVersionChangelog(versionId) {
		debug('getVersionChangelog: called, versionId:', versionId)
		try {
			return await client.labrinth.versions_v2.getVersion(versionId)
		} catch {
			debug('getVersionChangelog: failed for', versionId)
			return null
		}
	},

	async onModpackVersionConfirm(version) {
		if (!modpack.value) return
		debug('onModpackVersionConfirm: called, version:', version.id)
		debug('onModpackVersionConfirm: emitting reinstall before API call')
		emit('reinstall')
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
			debug('onModpackVersionConfirm: installContent succeeded, invalidating')
			invalidateServerState()
		} catch (err) {
			debug('onModpackVersionConfirm: failed, emitting reinstall-failed', err)
			emit('reinstall-failed')
			addNotification({
				type: 'error',
				text: err instanceof Error ? err.message : formatMessage(messages.failedToChangeVersion),
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
		currentGameVersion: addonsQuery.data.value?.game_version ?? server.value?.mc_version ?? '',
		currentLoader: addonsQuery.data.value?.modloader ?? server.value?.loader ?? '',
	})),

	isServer: true,
	isApp: false,

	lockPlatform: true,
	hideLoaderVersion: true,

	async previewSave(_platform, gameVersion, _loaderVersionId, signal) {
		const result = await client.archon.content_v1.getUpdateGameVersionPreview(
			serverId,
			worldId.value!,
			gameVersion,
			signal,
		)
		if (result.addon_changes.length === 0 && !result.has_unknown_content) return null
		return {
			diffs: result.addon_changes.map((diff) => ({
				type: diff.type,
				projectName: diff.project?.title ?? undefined,
				fileName: diff.file_name ?? undefined,
				currentVersionName: diff.current_version?.version_number ?? undefined,
				newVersionName: diff.new_version?.version_number ?? undefined,
			})),
			newGameVersion: result.new_game_version,
			newLoaderVersion: result.new_loader_version,
			hasUnknownContent: result.has_unknown_content,
		}
	},
})

watch(
	() => server.value?.status,
	(newStatus, oldStatus) => {
		debug('status watcher:', oldStatus, '->', newStatus, {
			'server.loader': server.value?.loader,
			'server.mc_version': server.value?.mc_version,
			'server.loader_version': server.value?.loader_version,
		})
		if (oldStatus === 'installing' && newStatus === 'available') {
			debug('status installing->available, resetting editing refs')
			editingPlatform.value = server.value?.loader?.toLowerCase() ?? 'vanilla'
			editingGameVersion.value = server.value?.mc_version ?? ''
		}
	},
)

function onBrowseModpacks() {
	debug('onBrowseModpacks: navigating to modpack discovery')
	navigateTo({
		path: '/discover/modpacks',
		query: { sid: serverId, from: 'reset-server', wid: worldId.value },
	})
}
</script>
