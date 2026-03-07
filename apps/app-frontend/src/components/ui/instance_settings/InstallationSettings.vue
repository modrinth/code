<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	commonMessages,
	defineMessages,
	formatLoaderLabel,
	injectNotificationManager,
	InstallationSettingsLayout,
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
	edit,
	get_linked_modpack_info,
	install,
	update_managed_modrinth_version,
	update_repair_modrinth,
} from '@/helpers/profile'
import { get_game_versions, get_loaders } from '@/helpers/tags'

import type { InstanceSettingsTabProps, Manifest } from '../../../helpers/types'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const queryClient = useQueryClient()

const props = defineProps<InstanceSettingsTabProps>()
const emit = defineEmits<{
	unlinked: []
}>()

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
	queryKey: computed(() => ['linkedModpackInfo', props.instance.path]),
	queryFn: () => get_linked_modpack_info(props.instance.path, 'must_revalidate'),
	enabled: computed(() => !!props.instance.linked_data?.project_id && !props.offline),
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

provideInstallationSettings({
	loading: ref(false),
	installationInfo: computed(() => {
		const rows = [
			{
				label: formatMessage(commonMessages.platformLabel),
				value: formatLoaderLabel(props.instance.loader),
			},
			{
				label: formatMessage(commonMessages.gameVersionLabel),
				value: props.instance.game_version,
			},
		]
		if (props.instance.loader !== 'vanilla' && props.instance.loader_version) {
			rows.push({
				label: formatMessage(messages.loaderVersion, {
					loader: formatLoaderLabel(props.instance.loader),
				}),
				value: props.instance.loader_version,
			})
		}
		return rows
	}),
	isLinked: computed(() => !!props.instance.linked_data?.locked),
	isBusy: computed(
		() =>
			props.instance.install_stage !== 'installed' ||
			repairing.value ||
			reinstalling.value ||
			!!props.offline,
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
	currentPlatform: computed(() => props.instance.loader),
	currentGameVersion: computed(() => props.instance.game_version),
	currentLoaderVersion: computed(() => props.instance.loader_version ?? ''),
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
		await edit(props.instance.path, editProfile).catch(handleError)
	},

	afterSave: async () => {
		await install(props.instance.path, false).catch(handleError)
		trackEvent('InstanceRepair', {
			loader: props.instance.loader,
			game_version: props.instance.game_version,
		})
	},

	async repair() {
		repairing.value = true
		await install(props.instance.path, true).catch(handleError)
		repairing.value = false
		trackEvent('InstanceRepair', {
			loader: props.instance.loader,
			game_version: props.instance.game_version,
		})
	},

	async reinstallModpack() {
		reinstalling.value = true
		await update_repair_modrinth(props.instance.path).catch(handleError)
		reinstalling.value = false
		trackEvent('InstanceRepair', {
			loader: props.instance.loader,
			game_version: props.instance.game_version,
		})
	},

	async unlinkModpack() {
		await edit(props.instance.path, {
			linked_data: null as unknown as undefined,
		})
	},

	getCachedModpackVersions: () => null,
	async fetchModpackVersions() {
		const versions = await get_project_versions(props.instance.linked_data!.project_id!).catch(
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
		await update_managed_modrinth_version(props.instance.path, version.id)
		await queryClient.invalidateQueries({
			queryKey: ['linkedModpackInfo', props.instance.path],
		})
	},

	updaterModalProps: computed(() => ({
		isApp: true,
		currentVersionId:
			modpackInfo.value?.update_version_id ?? props.instance.linked_data?.version_id ?? '',
		projectIconUrl: modpackInfo.value?.project?.icon_url,
		projectName: modpackInfo.value?.project?.title ?? 'Modpack',
		currentGameVersion: props.instance.game_version,
		currentLoader: props.instance.loader,
	})),

	isServer: false,
	isApp: true,
	showModpackVersionActions: !props.isMinecraftServer,
	repairing,
	reinstalling,
})
</script>

<template>
	<InstallationSettingsLayout />
</template>
