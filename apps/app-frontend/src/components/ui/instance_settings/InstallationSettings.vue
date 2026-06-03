<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	commonMessages,
	defineMessages,
	formatLoaderLabel,
	injectNotificationManager,
	InstallationSettingsLayout,
	provideAppBackup,
	provideInstallationSettings,
	useDebugLogger,
	useVIntl,
} from '@modrinth/ui'
import type { GameVersionTag, PlatformTag } from '@modrinth/utils'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref, shallowRef } from 'vue'

import { trackEvent } from '@/helpers/analytics'
import { get_project_versions, get_version } from '@/helpers/cache'
import { get_loader_versions } from '@/helpers/metadata'
import {
	duplicate,
	edit,
	get_linked_modpack_info,
	install,
	list,
	update_managed_modrinth_version,
	update_repair_modrinth,
} from '@/helpers/profile'
import { get_game_versions, get_loaders } from '@/helpers/tags'
import { injectInstanceSettings } from '@/providers/instance-settings'

import type { Manifest } from '../../../helpers/types'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const queryClient = useQueryClient()
const debug = useDebugLogger('AppInstallationSettings')

const { instance, offline, isMinecraftServer, onUnlinked, closeModal } = injectInstanceSettings()

debug('metadata load: start', {
	instancePath: instance.value.path,
	loader: instance.value.loader,
	gameVersion: instance.value.game_version,
	installStage: instance.value.install_stage,
})

const [
	fabric_versions,
	forge_versions,
	quilt_versions,
	neoforge_versions,
	all_game_versions,
	loaders,
] = await Promise.all([
	get_loader_versions('fabric')
		.then((manifest: Manifest) => shallowRef(manifest))
		.catch(handleError),
	get_loader_versions('forge')
		.then((manifest: Manifest) => shallowRef(manifest))
		.catch(handleError),
	get_loader_versions('quilt')
		.then((manifest: Manifest) => shallowRef(manifest))
		.catch(handleError),
	get_loader_versions('neo')
		.then((manifest: Manifest) => shallowRef(manifest))
		.catch(handleError),
	get_game_versions()
		.then((gameVersions: GameVersionTag[]) => shallowRef(gameVersions))
		.catch(handleError),
	get_loaders()
		.then((value: PlatformTag[]) =>
			value
				.filter(
					(item) => item.supported_project_types.includes('modpack') || item.name === 'vanilla',
				)
				.sort((a, b) => (a.name === 'vanilla' ? -1 : b.name === 'vanilla' ? 1 : 0)),
		)
		.then((loader: PlatformTag[]) => ref(loader))
		.catch(handleError),
])

debug('metadata load: done', {
	hasFabricManifest: !!fabric_versions?.value,
	hasForgeManifest: !!forge_versions?.value,
	hasQuiltManifest: !!quilt_versions?.value,
	hasNeoforgeManifest: !!neoforge_versions?.value,
	gameVersions: all_game_versions?.value?.length ?? 0,
	availablePlatforms: loaders?.value?.map((loader) => loader.name) ?? [],
})

const { data: modpackInfo } = useQuery({
	queryKey: computed(() => ['linkedModpackInfo', instance.value.path]),
	queryFn: () => get_linked_modpack_info(instance.value.path, 'must_revalidate'),
	enabled: computed(() => !!instance.value.linked_data?.project_id && !offline),
})

const repairing = ref(false)
const reinstalling = ref(false)

const messages = defineMessages({
	loaderVersion: {
		id: 'instance.settings.tabs.installation.loader-version',
		defaultMessage: '{loader} version',
	},
})

function getManifest(loader: string) {
	const map: Record<string, typeof fabric_versions> = {
		fabric: fabric_versions,
		forge: forge_versions,
		quilt: quilt_versions,
		neoforge: neoforge_versions,
	}
	const manifest = map[loader]
	debug('getManifest:', {
		loader,
		hasManifest: !!manifest?.value,
		gameVersions: manifest?.value?.gameVersions?.length ?? 0,
	})
	return manifest
}

provideAppBackup({
	async createBackup() {
		debug('createBackup: start', {
			instancePath: instance.value.path,
			instanceName: instance.value.name,
		})
		const allProfiles = await list()
		const prefix = `${instance.value.name} - Backup #`
		const existingNums = allProfiles
			.filter((p) => p.name.startsWith(prefix))
			.map((p) => parseInt(p.name.slice(prefix.length), 10))
			.filter((n) => !isNaN(n))
		const nextNum = existingNums.length > 0 ? Math.max(...existingNums) + 1 : 1
		const newPath = await duplicate(instance.value.path)
		await edit(newPath, { name: `${prefix}${nextNum}` })
		debug('createBackup: done', { newPath, backupName: `${prefix}${nextNum}` })
	},
})

provideInstallationSettings({
	closeSettings: closeModal,
	loading: ref(false),
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
	isLinked: computed(() => !!instance.value.linked_data?.locked),
	isBusy: computed(
		() =>
			instance.value.install_stage !== 'installed' ||
			repairing.value ||
			reinstalling.value ||
			!!offline,
	),
	modpack: computed(() => {
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
	availablePlatforms: loaders?.value?.map((x) => x.name) ?? [],

	resolveGameVersions(loader, showSnapshots) {
		const versions = all_game_versions?.value ?? []
		const filtered = versions.filter((item) => {
			if (loader === 'vanilla') return true
			const manifest = getManifest(loader)
			return !!manifest?.value?.gameVersions?.some((x) => item.version === x.id)
		})
		const result = (showSnapshots
			? filtered
			: filtered.filter((x) => x.version_type === 'release')
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
		if (!manifest?.value) {
			debug('resolveLoaderVersions: no manifest', { loader, gameVersion })
			return []
		}
		if (loader === 'fabric' || loader === 'quilt') {
			const result = manifest.value.gameVersions[0]?.loaders ?? []
			debug('resolveLoaderVersions: fabric/quilt result', {
				loader,
				gameVersion,
				count: result.length,
			})
			return result
		}
		const result = manifest.value.gameVersions?.find((item) => item.id === gameVersion)?.loaders ?? []
		debug('resolveLoaderVersions: result', { loader, gameVersion, count: result.length })
		return result
	},

	resolveHasSnapshots(loader) {
		const versions = all_game_versions?.value ?? []
		if (loader === 'vanilla') {
			const result = versions.some((x) => x.version_type !== 'release')
			debug('resolveHasSnapshots: vanilla', { loader, result })
			return result
		}
		const manifest = getManifest(loader)
		const supported = versions.filter(
			(item) => !!manifest?.value?.gameVersions?.some((x) => item.version === x.id),
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
			instancePath: instance.value.path,
			platform,
			gameVersion,
			loaderVersionId,
		})
		const editProfile: Record<string, string | undefined> = {
			loader: platform,
			game_version: gameVersion,
		}
		if (platform !== 'vanilla' && loaderVersionId) {
			editProfile.loader_version = loaderVersionId
		}
		await edit(instance.value.path, editProfile).catch(handleError)
		debug('save: edit complete', { editProfile })
	},

	afterSave: async () => {
		debug('afterSave: installing', { instancePath: instance.value.path })
		await install(instance.value.path, false).catch(handleError)
		trackEvent('InstanceRepair', {
			loader: instance.value.loader,
			game_version: instance.value.game_version,
		})
		debug('afterSave: done')
	},

	async repair() {
		debug('repair: called', { instancePath: instance.value.path })
		repairing.value = true
		await install(instance.value.path, true).catch(handleError)
		repairing.value = false
		trackEvent('InstanceRepair', {
			loader: instance.value.loader,
			game_version: instance.value.game_version,
		})
		debug('repair: done')
	},

	async reinstallModpack() {
		debug('reinstallModpack: called', { instancePath: instance.value.path })
		reinstalling.value = true
		await update_repair_modrinth(instance.value.path).catch(handleError)
		reinstalling.value = false
		trackEvent('InstanceRepair', {
			loader: instance.value.loader,
			game_version: instance.value.game_version,
		})
		debug('reinstallModpack: done')
	},

	async unlinkModpack() {
		debug('unlinkModpack: called', { instancePath: instance.value.path })
		await edit(instance.value.path, {
			linked_data: null as unknown as undefined,
		})
		await queryClient.invalidateQueries({
			queryKey: ['linkedModpackInfo', instance.value.path],
		})
		onUnlinked()
		debug('unlinkModpack: done')
	},

	getCachedModpackVersions: () => null,
	async fetchModpackVersions() {
		debug('fetchModpackVersions: called', {
			projectId: instance.value.linked_data?.project_id,
		})
		const versions = await get_project_versions(instance.value.linked_data!.project_id!).catch(
			handleError,
		)
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
			instancePath: instance.value.path,
		})
		await update_managed_modrinth_version(instance.value.path, version.id)
		await queryClient.invalidateQueries({
			queryKey: ['linkedModpackInfo', instance.value.path],
		})
		debug('onModpackVersionConfirm: done')
	},

	updaterModalProps: computed(() => ({
		isApp: true,
		currentVersionId:
			modpackInfo.value?.update_version_id ?? instance.value.linked_data?.version_id ?? '',
		projectIconUrl: modpackInfo.value?.project?.icon_url,
		projectName: modpackInfo.value?.project?.title ?? 'Modpack',
		currentGameVersion: instance.value.game_version,
		currentLoader: instance.value.loader,
	})),

	isServer: false,
	isApp: true,
	showModpackVersionActions: !isMinecraftServer.value,
	repairing,
	reinstalling,
})
</script>

<template>
	<InstallationSettingsLayout />
</template>
