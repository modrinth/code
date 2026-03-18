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

const { instance, offline, isMinecraftServer, onUnlinked } = injectInstanceSettings()

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

const { data: modpackInfo } = useQuery({
	queryKey: computed(() => ['linkedModpackInfo', instance.path]),
	queryFn: () => get_linked_modpack_info(instance.path, 'must_revalidate'),
	enabled: computed(() => !!instance.linked_data?.project_id && !offline),
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
	return map[loader]
}

provideAppBackup({
	async createBackup() {
		const allProfiles = await list()
		const prefix = `${instance.name} - Backup #`
		const existingNums = allProfiles
			.filter((p) => p.name.startsWith(prefix))
			.map((p) => parseInt(p.name.slice(prefix.length), 10))
			.filter((n) => !isNaN(n))
		const nextNum = existingNums.length > 0 ? Math.max(...existingNums) + 1 : 1
		const newPath = await duplicate(instance.path)
		await edit(newPath, { name: `${prefix}${nextNum}` })
	},
})

provideInstallationSettings({
	loading: ref(false),
	installationInfo: computed(() => {
		const rows = [
			{
				label: formatMessage(commonMessages.platformLabel),
				value: formatLoaderLabel(instance.loader),
			},
			{
				label: formatMessage(commonMessages.gameVersionLabel),
				value: instance.game_version,
			},
		]
		if (instance.loader !== 'vanilla' && instance.loader_version) {
			rows.push({
				label: formatMessage(messages.loaderVersion, {
					loader: formatLoaderLabel(instance.loader),
				}),
				value: instance.loader_version,
			})
		}
		return rows
	}),
	isLinked: computed(() => !!instance.linked_data?.locked),
	isBusy: computed(
		() =>
			instance.install_stage !== 'installed' ||
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
	currentPlatform: computed(() => instance.loader),
	currentGameVersion: computed(() => instance.game_version),
	currentLoaderVersion: computed(() => instance.loader_version ?? ''),
	availablePlatforms: loaders?.value?.map((x) => x.name) ?? [],

	resolveGameVersions(loader, showSnapshots) {
		const versions = all_game_versions?.value ?? []
		const filtered = versions.filter((item) => {
			if (loader === 'vanilla') return true
			const manifest = getManifest(loader)
			return !!manifest?.value?.gameVersions?.some((x) => item.version === x.id)
		})
		return (showSnapshots ? filtered : filtered.filter((x) => x.version_type === 'release')).map(
			(x) => ({ value: x.version, label: x.version }),
		)
	},

	resolveLoaderVersions(loader, gameVersion) {
		if (loader === 'vanilla' || !gameVersion) return []
		const manifest = getManifest(loader)
		if (!manifest?.value) return []
		if (loader === 'fabric' || loader === 'quilt') {
			return manifest.value.gameVersions[0]?.loaders ?? []
		}
		return manifest.value.gameVersions?.find((item) => item.id === gameVersion)?.loaders ?? []
	},

	resolveHasSnapshots(loader) {
		const versions = all_game_versions?.value ?? []
		if (loader === 'vanilla') return versions.some((x) => x.version_type !== 'release')
		const manifest = getManifest(loader)
		const supported = versions.filter(
			(item) => !!manifest?.value?.gameVersions?.some((x) => item.version === x.id),
		)
		return supported.some((x) => x.version_type !== 'release')
	},

	async save(platform, gameVersion, loaderVersionId) {
		const editProfile: Record<string, string | undefined> = {
			loader: platform,
			game_version: gameVersion,
		}
		if (platform !== 'vanilla' && loaderVersionId) {
			editProfile.loader_version = loaderVersionId
		}
		await edit(instance.path, editProfile).catch(handleError)
	},

	afterSave: async () => {
		await install(instance.path, false).catch(handleError)
		trackEvent('InstanceRepair', {
			loader: instance.loader,
			game_version: instance.game_version,
		})
	},

	async repair() {
		repairing.value = true
		await install(instance.path, true).catch(handleError)
		repairing.value = false
		trackEvent('InstanceRepair', {
			loader: instance.loader,
			game_version: instance.game_version,
		})
	},

	async reinstallModpack() {
		reinstalling.value = true
		await update_repair_modrinth(instance.path).catch(handleError)
		reinstalling.value = false
		trackEvent('InstanceRepair', {
			loader: instance.loader,
			game_version: instance.game_version,
		})
	},

	async unlinkModpack() {
		await edit(instance.path, {
			linked_data: null as unknown as undefined,
		})
		await queryClient.invalidateQueries({
			queryKey: ['linkedModpackInfo', instance.path],
		})
		onUnlinked()
	},

	getCachedModpackVersions: () => null,
	async fetchModpackVersions() {
		const versions = await get_project_versions(instance.linked_data!.project_id!).catch(
			handleError,
		)
		return (versions ?? []) as Labrinth.Versions.v2.Version[]
	},

	async getVersionChangelog(versionId: string) {
		return (await get_version(versionId, 'must_revalidate').catch(
			() => null,
		)) as Labrinth.Versions.v2.Version | null
	},

	async onModpackVersionConfirm(version) {
		await update_managed_modrinth_version(instance.path, version.id)
		await queryClient.invalidateQueries({
			queryKey: ['linkedModpackInfo', instance.path],
		})
	},

	updaterModalProps: computed(() => ({
		isApp: true,
		currentVersionId:
			modpackInfo.value?.update_version_id ?? instance.linked_data?.version_id ?? '',
		projectIconUrl: modpackInfo.value?.project?.icon_url,
		projectName: modpackInfo.value?.project?.title ?? 'Modpack',
		currentGameVersion: instance.game_version,
		currentLoader: instance.loader,
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
