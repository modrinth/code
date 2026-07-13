<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	commonMessages,
	defineMessages,
	formatLoaderLabel,
	injectFilePicker,
	injectNotificationManager,
	InstallationSettingsLayout,
	provideInstallationSettings,
	useDebugLogger,
	useVIntl,
} from '@modrinth/ui'
import type { GameVersionTag, PlatformTag } from '@modrinth/utils'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import SharedInstanceInstallationSettingsControls from '@/components/ui/shared-instances/SharedInstanceInstallationSettingsControls.vue'
import { useManagedContentPolicy } from '@/composables/instances/use-managed-content-policy'
import { trackEvent } from '@/helpers/analytics'
import { get_project_versions, get_version } from '@/helpers/cache'
import {
	install_existing_instance,
	install_pack_to_existing_instance,
	wait_for_install_job,
} from '@/helpers/install'
import {
	edit,
	get_linked_modpack_info,
	unlink_shared_instance,
	unpublish_shared_instance,
	update_managed_modrinth_version,
	update_repair_modrinth,
} from '@/helpers/instance'
import { get_loader_versions } from '@/helpers/metadata'
import { get_game_versions, get_loaders } from '@/helpers/tags'
import { provideInstanceBackup } from '@/providers/instance-backup'
import { injectInstanceSettings } from '@/providers/instance-settings'
import { useTheming } from '@/store/state'

import type { Manifest } from '../../../helpers/types'

const { handleError } = injectNotificationManager()
const filePicker = injectFilePicker()
const { formatMessage } = useVIntl()
const queryClient = useQueryClient()
const debug = useDebugLogger('AppInstallationSettings')
const themeStore = useTheming()

const { instance, offline, isMinecraftServer, onUnlinked, closeModal } = injectInstanceSettings()
const managedContentPolicy = useManagedContentPolicy(instance)
const skipNonEssentialWarnings = computed(() =>
	themeStore.getFeatureFlag('skip_non_essential_warnings'),
)

debug('metadata load: start', {
	instanceId: instance.value.id,
	loader: instance.value.loader,
	gameVersion: instance.value.game_version,
	installStage: instance.value.install_stage,
})

function getSupportedModpackLoaders() {
	return get_loaders().then((value: PlatformTag[]) =>
		value
			.filter((item) => item.supported_project_types.includes('modpack') || item.name === 'vanilla')
			.sort((a, b) => (a.name === 'vanilla' ? -1 : b.name === 'vanilla' ? 1 : 0)),
	)
}

const fabricVersionsQuery = useQuery({
	queryKey: ['instance-settings', 'loader-versions', 'fabric'],
	queryFn: () => get_loader_versions('fabric') as Promise<Manifest>,
})
const forgeVersionsQuery = useQuery({
	queryKey: ['instance-settings', 'loader-versions', 'forge'],
	queryFn: () => get_loader_versions('forge') as Promise<Manifest>,
})
const quiltVersionsQuery = useQuery({
	queryKey: ['instance-settings', 'loader-versions', 'quilt'],
	queryFn: () => get_loader_versions('quilt') as Promise<Manifest>,
})
const neoforgeVersionsQuery = useQuery({
	queryKey: ['instance-settings', 'loader-versions', 'neo'],
	queryFn: () => get_loader_versions('neo') as Promise<Manifest>,
})
const gameVersionsQuery = useQuery({
	queryKey: ['instance-settings', 'game-versions'],
	queryFn: () => get_game_versions() as Promise<GameVersionTag[]>,
})
const loadersQuery = useQuery({
	queryKey: ['instance-settings', 'loaders', 'modpack'],
	queryFn: getSupportedModpackLoaders,
})

const metadataLoading = computed(() =>
	[
		fabricVersionsQuery,
		forgeVersionsQuery,
		quiltVersionsQuery,
		neoforgeVersionsQuery,
		gameVersionsQuery,
		loadersQuery,
	].some((query) => query.isLoading.value),
)

debug('metadata queries configured', {
	instanceId: instance.value.id,
	loader: instance.value.loader,
	gameVersion: instance.value.game_version,
})

const isModrinthLinkedModpack = computed(
	() =>
		instance.value.link?.type === 'modrinth_modpack' ||
		instance.value.link?.type === 'server_project_modpack' ||
		(instance.value.link?.type === 'shared_instance' &&
			!!instance.value.link.modpack_project_id &&
			!!instance.value.link.modpack_version_id),
)
const isImportedModpack = computed(() => instance.value.link?.type === 'imported_modpack')
const isSharedInstanceManagedModpack = managedContentPolicy.isManagedModpack
const canUnpublishSharedInstance = managedContentPolicy.canUnpublish
const canUnlinkSharedInstance = managedContentPolicy.canUnlink

const modpackInfoQuery = useQuery({
	queryKey: computed(() => ['linkedModpackInfo', instance.value.id]),
	queryFn: () => get_linked_modpack_info(instance.value.id, 'must_revalidate'),
	enabled: computed(() => isModrinthLinkedModpack.value && !offline),
})
const modpackInfo = modpackInfoQuery.data

const repairing = ref(false)
const reinstalling = ref(false)
const unpublishingSharedInstance = ref(false)
const unlinkingSharedInstance = ref(false)

async function unpublishSharedInstance() {
	unpublishingSharedInstance.value = true
	try {
		await unpublish_shared_instance(instance.value.id)
		await queryClient.invalidateQueries({ queryKey: ['sharedInstanceUsers', instance.value.id] })
		await queryClient.invalidateQueries({ queryKey: ['linkedModpackInfo', instance.value.id] })
		onUnlinked()
	} catch (error) {
		handleError(error)
	} finally {
		unpublishingSharedInstance.value = false
	}
}

async function unlinkSharedInstance() {
	unlinkingSharedInstance.value = true
	try {
		await unlink_shared_instance(instance.value.id)
		await queryClient.invalidateQueries({ queryKey: ['sharedInstanceUsers', instance.value.id] })
		await queryClient.invalidateQueries({ queryKey: ['linkedModpackInfo', instance.value.id] })
		onUnlinked()
	} catch (error) {
		handleError(error)
	} finally {
		unlinkingSharedInstance.value = false
	}
}

const messages = defineMessages({
	loaderVersion: {
		id: 'instance.settings.tabs.installation.loader-version',
		defaultMessage: '{loader} version',
	},
})

function getManifest(loader: string) {
	const map: Record<string, Manifest | undefined> = {
		fabric: fabricVersionsQuery.data.value,
		forge: forgeVersionsQuery.data.value,
		quilt: quiltVersionsQuery.data.value,
		neoforge: neoforgeVersionsQuery.data.value,
	}
	const manifest = map[loader]
	debug('getManifest:', {
		loader,
		hasManifest: !!manifest,
		gameVersions: manifest?.gameVersions?.length ?? 0,
	})
	return manifest
}

async function installLocalModpackFromPicker() {
	const picked = await filePicker.pickModpackFile({ readFile: false })
	if (!picked?.path) return false

	const job = await install_pack_to_existing_instance(instance.value.id, {
		type: 'fromFile',
		path: picked.path,
	}).catch(handleError)
	if (!job) return false

	const completed = await wait_for_install_job(job.job_id).catch(handleError)
	return !!completed
}

provideInstanceBackup(instance)

provideInstallationSettings({
	closeSettings: closeModal,
	loading: computed(() => metadataLoading.value || modpackInfoQuery.isLoading.value),
	installationInfo: computed(() => {
		const rows = [
			{
				label: formatMessage(commonMessages.platformLabel),
				value: formatLoaderLabel(instance.value.loader),
			},
			{
				label: formatMessage(commonMessages.gameVersionLabel),
				value: instance.value.game_version,
			},
		]
		if (instance.value.loader !== 'vanilla' && instance.value.loader_version) {
			rows.push({
				label: formatMessage(messages.loaderVersion, {
					loader: formatLoaderLabel(instance.value.loader),
				}),
				value: instance.value.loader_version,
			})
		}
		return rows
	}),
	isLinked: computed(
		() =>
			isModrinthLinkedModpack.value ||
			isImportedModpack.value ||
			isSharedInstanceManagedModpack.value,
	),
	isBusy: computed(
		() =>
			instance.value.install_stage !== 'installed' ||
			repairing.value ||
			reinstalling.value ||
			unlinkingSharedInstance.value ||
			unpublishingSharedInstance.value ||
			!!offline,
	),
	skipNonEssentialWarnings,
	modpack: computed(() => {
		if (isImportedModpack.value && instance.value.link?.type === 'imported_modpack') {
			return {
				iconUrl: instance.value.icon_path,
				title: instance.value.link.name ?? instance.value.name,
				versionNumber: instance.value.link.version_number ?? undefined,
				filename: instance.value.link.filename ?? undefined,
			}
		}
		if (!modpackInfo.value) return null
		return {
			iconUrl: modpackInfo.value.project.icon_url,
			title: modpackInfo.value.project.title,
			link: `/project/${modpackInfo.value.project.slug ?? modpackInfo.value.project.id}`,
			versionNumber: modpackInfo.value.version?.version_number,
		}
	}),
	currentPlatform: computed(() => instance.value.loader),
	currentGameVersion: computed(() => instance.value.game_version),
	currentLoaderVersion: computed(() => instance.value.loader_version ?? ''),
	availablePlatforms: computed(() => loadersQuery.data.value?.map((x) => x.name) ?? []),

	resolveGameVersions(loader, showSnapshots) {
		const versions = gameVersionsQuery.data.value ?? []
		const filtered = versions.filter((item) => {
			if (loader === 'vanilla') return true
			const manifest = getManifest(loader)
			return !!manifest?.gameVersions?.some((x) => item.version === x.id)
		})
		const result = (
			showSnapshots ? filtered : filtered.filter((x) => x.version_type === 'release')
		).map((x) => ({ value: x.version, label: x.version }))
		debug('resolveGameVersions:', {
			loader,
			showSnapshots,
			totalVersions: versions.length,
			filteredVersions: filtered.length,
			resultVersions: result.length,
		})
		return result
	},

	resolveLoaderVersions(loader, gameVersion) {
		if (loader === 'vanilla' || !gameVersion) {
			debug('resolveLoaderVersions: skipped', { loader, gameVersion })
			return []
		}
		const manifest = getManifest(loader)
		if (!manifest) {
			debug('resolveLoaderVersions: no manifest', { loader, gameVersion })
			return []
		}
		const entry = manifest.gameVersions?.find((item) => item.id === gameVersion)
		if (entry?.versionGroup) {
			const result =
				manifest.versionGroups?.find((group) => group.id === entry.versionGroup)?.loaders ?? []
			debug('resolveLoaderVersions: version group result', {
				loader,
				gameVersion,
				versionGroup: entry.versionGroup,
				count: result.length,
			})
			return result
		}
		const placeholder = manifest.gameVersions?.find((item) => item.id === '${modrinth.gameVersion}')
		if (placeholder) {
			const result = manifest.gameVersions?.some((item) => item.id === gameVersion)
				? placeholder.loaders
				: []
			debug('resolveLoaderVersions: placeholder result', {
				loader,
				gameVersion,
				count: result.length,
			})
			return result
		}
		const result = entry?.loaders ?? []
		debug('resolveLoaderVersions: result', { loader, gameVersion, count: result.length })
		return result
	},

	resolveHasSnapshots(loader) {
		const versions = gameVersionsQuery.data.value ?? []
		if (loader === 'vanilla') {
			const result = versions.some((x) => x.version_type !== 'release')
			debug('resolveHasSnapshots: vanilla', { loader, result })
			return result
		}
		const manifest = getManifest(loader)
		const supported = versions.filter(
			(item) => !!manifest?.gameVersions?.some((x) => item.version === x.id),
		)
		const result = supported.some((x) => x.version_type !== 'release')
		debug('resolveHasSnapshots:', {
			loader,
			totalVersions: versions.length,
			supportedVersions: supported.length,
			result,
		})
		return result
	},

	async save(platform, gameVersion, loaderVersionId) {
		debug('save: called', {
			instanceId: instance.value.id,
			platform,
			gameVersion,
			loaderVersionId,
		})
		const editInstancePatch: Record<string, string | undefined> = {
			loader: platform,
			game_version: gameVersion,
		}
		if (platform !== 'vanilla' && loaderVersionId) {
			editInstancePatch.loader_version = loaderVersionId
		}
		await edit(instance.value.id, editInstancePatch).catch(handleError)
		debug('save: edit complete', { editInstancePatch })
	},

	afterSave: async () => {
		debug('afterSave: installing', { instanceId: instance.value.id })
		await install_existing_instance(instance.value.id, false).catch(handleError)
		trackEvent('InstanceRepair', {
			loader: instance.value.loader,
			game_version: instance.value.game_version,
		})
		debug('afterSave: done')
	},

	async repair() {
		debug('repair: called', { instanceId: instance.value.id })
		repairing.value = true
		await install_existing_instance(instance.value.id, true).catch(handleError)
		repairing.value = false
		trackEvent('InstanceRepair', {
			loader: instance.value.loader,
			game_version: instance.value.game_version,
		})
		debug('repair: done')
	},

	async reinstallModpack() {
		debug('reinstallModpack: called', { instanceId: instance.value.id })
		reinstalling.value = true
		let shouldTrack = false
		try {
			if (isImportedModpack.value) {
				shouldTrack = await installLocalModpackFromPicker()
			} else {
				await update_repair_modrinth(instance.value.id).catch(handleError)
				shouldTrack = true
			}
		} finally {
			reinstalling.value = false
		}
		if (shouldTrack) {
			trackEvent('InstanceRepair', {
				loader: instance.value.loader,
				game_version: instance.value.game_version,
			})
		}
		debug('reinstallModpack: done')
	},

	async swapModpack() {
		debug('swapModpack: called', { instanceId: instance.value.id })
		reinstalling.value = true
		try {
			const installed = await installLocalModpackFromPicker()
			if (installed) {
				trackEvent('InstanceRepair', {
					loader: instance.value.loader,
					game_version: instance.value.game_version,
				})
			}
		} finally {
			reinstalling.value = false
		}
		debug('swapModpack: done')
	},

	async unlinkModpack() {
		debug('unlinkModpack: called', { instanceId: instance.value.id })
		await edit(instance.value.id, {
			link: null as unknown as undefined,
		})
		await queryClient.invalidateQueries({
			queryKey: ['linkedModpackInfo', instance.value.id],
		})
		onUnlinked()
		debug('unlinkModpack: done')
	},

	getCachedModpackVersions: () => null,
	async fetchModpackVersions() {
		debug('fetchModpackVersions: called', {
			projectId: instance.value.link?.project_id,
		})
		const versions = await get_project_versions(instance.value.link!.project_id!).catch(handleError)
		debug('fetchModpackVersions: done', { count: versions?.length ?? 0 })
		return (versions ?? []) as Labrinth.Versions.v2.Version[]
	},

	async getVersionChangelog(versionId: string) {
		debug('getVersionChangelog: called', { versionId })
		return (await get_version(versionId, 'must_revalidate').catch(
			() => null,
		)) as Labrinth.Versions.v2.Version | null
	},

	async onModpackVersionConfirm(version) {
		debug('onModpackVersionConfirm: called', {
			versionId: version.id,
			instanceId: instance.value.id,
		})
		await update_managed_modrinth_version(instance.value.id, version.id)
		await queryClient.invalidateQueries({
			queryKey: ['linkedModpackInfo', instance.value.id],
		})
		debug('onModpackVersionConfirm: done')
	},

	updaterModalProps: computed(() => ({
		isApp: true,
		currentVersionId: modpackInfo.value?.update_version_id ?? instance.value.link?.version_id ?? '',
		projectIconUrl: modpackInfo.value?.project?.icon_url,
		projectName: modpackInfo.value?.project?.title ?? 'Modpack',
		currentGameVersion: instance.value.game_version,
		currentLoader: instance.value.loader,
	})),

	isServer: false,
	isApp: true,
	showModpackVersionActions: computed(
		() =>
			isModrinthLinkedModpack.value &&
			!isMinecraftServer.value &&
			!isSharedInstanceManagedModpack.value,
	),
	isLocalFile: isImportedModpack,
	isManagedModpack: isSharedInstanceManagedModpack,
	managedModpackWarning: managedContentPolicy.managedModpackWarning,
	repairing,
	reinstalling,
})
</script>

<template>
	<InstallationSettingsLayout>
		<template #extra>
			<SharedInstanceInstallationSettingsControls
				:can-unpublish="canUnpublishSharedInstance"
				:can-unlink="canUnlinkSharedInstance"
				:busy="
					instance.install_stage !== 'installed' ||
					repairing ||
					reinstalling ||
					unpublishingSharedInstance ||
					unlinkingSharedInstance ||
					!!offline
				"
				:unpublishing="unpublishingSharedInstance"
				:unlinking="unlinkingSharedInstance"
				:unpublish="unpublishSharedInstance"
				:unlink="unlinkSharedInstance"
			/>
		</template>
	</InstallationSettingsLayout>
</template>
