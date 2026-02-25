<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { DownloadIcon, HammerIcon, SpinnerIcon, UndoIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	ConfirmUnlinkModal,
	ContentUpdaterModal,
	defineMessages,
	formatLoader,
	injectNotificationManager,
	InstallationSettingsLayout,
	provideInstallationSettings,
	useVIntl,
} from '@modrinth/ui'
import type { GameVersionTag, PlatformTag } from '@modrinth/utils'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { openUrl } from '@tauri-apps/plugin-opener'
import { computed, type ComputedRef, nextTick, type Ref, ref, shallowRef, watch } from 'vue'

import ConfirmModalWrapper from '@/components/ui/modal/ConfirmModalWrapper.vue'
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

import type {
	InstanceSettingsTabProps,
	Manifest,
	ManifestLoaderVersion,
} from '../../../helpers/types'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()

const repairConfirmModal = ref()
const modalConfirmReinstall = ref()
const confirmUnlinkModal = ref<InstanceType<typeof ConfirmUnlinkModal>>()
const contentUpdaterModal = ref<InstanceType<typeof ContentUpdaterModal> | null>()

const props = defineProps<InstanceSettingsTabProps>()

// --- Data fetching ---

const loader = ref(props.instance.loader)
const gameVersion = ref(props.instance.game_version)
const showSnapshots = ref(false)

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

// --- Linked modpack data ---

const queryClient = useQueryClient()

const { data: modpackInfo, isFetching: fetching } = useQuery({
	queryKey: computed(() => ['linkedModpackInfo', props.instance.path]),
	queryFn: () => get_linked_modpack_info(props.instance.path, 'must_revalidate'),
	enabled: computed(() => !!props.instance.linked_data?.project_id && !props.offline),
})

// --- Modpack version update (ContentUpdaterModal) ---

const updatingModpack = ref(false)
const updatingProjectVersions = ref<Labrinth.Versions.v2.Version[]>([])
const loadingVersions = ref(false)
const loadingChangelog = ref(false)
const isModpackUpdating = ref(false)

async function handleModpackUpdate() {
	if (!props.instance?.linked_data?.project_id) return

	updatingModpack.value = true
	updatingProjectVersions.value = []
	loadingVersions.value = true
	loadingChangelog.value = false

	await nextTick()

	contentUpdaterModal.value?.show(
		modpackInfo.value?.update_version_id ?? props.instance?.linked_data?.version_id ?? undefined,
	)

	const versions = (await get_project_versions(props.instance.linked_data.project_id).catch(
		handleError,
	)) as Labrinth.Versions.v2.Version[] | null

	loadingVersions.value = false

	if (!versions) return

	versions.sort(
		(a, b) => new Date(b.date_published).getTime() - new Date(a.date_published).getTime(),
	)

	updatingProjectVersions.value = versions
}

async function fetchAndSpliceVersion(
	versionId: string,
	cacheBehaviour?: Parameters<typeof get_version>[1],
	onError?: (err: unknown) => void,
) {
	const fullVersion = (await get_version(versionId, cacheBehaviour).catch(
		onError ?? (() => null),
	)) as Labrinth.Versions.v2.Version | null
	if (!fullVersion) return
	const index = updatingProjectVersions.value.findIndex((v) => v.id === versionId)
	if (index !== -1) {
		const newVersions = [...updatingProjectVersions.value]
		newVersions[index] = fullVersion
		updatingProjectVersions.value = newVersions
	}
}

async function handleVersionSelect(version: Labrinth.Versions.v2.Version) {
	if (version.changelog !== undefined) return
	loadingChangelog.value = true
	await fetchAndSpliceVersion(version.id, 'must_revalidate', handleError)
	loadingChangelog.value = false
}

async function handleVersionHover(version: Labrinth.Versions.v2.Version) {
	if (version.changelog !== undefined) return
	await fetchAndSpliceVersion(version.id)
}

function resetUpdateState() {
	updatingModpack.value = false
	updatingProjectVersions.value = []
	loadingVersions.value = false
	loadingChangelog.value = false
}

async function handleModalUpdate(selectedVersion: Labrinth.Versions.v2.Version) {
	if (!props.instance?.path) return

	isModpackUpdating.value = true
	try {
		await update_managed_modrinth_version(props.instance.path, selectedVersion.id)
		await queryClient.invalidateQueries({ queryKey: ['linkedModpackInfo', props.instance.path] })
	} catch (err) {
		handleError(err)
	} finally {
		isModpackUpdating.value = false
		resetUpdateState()
	}
}

// --- Version selection logic ---

const gameVersionsForLoader = computed(() => {
	return all_game_versions?.value.filter((item) => {
		if (loader.value === 'fabric') {
			return !!fabric_versions?.value.gameVersions.some((x) => item.version === x.id)
		} else if (loader.value === 'forge') {
			return !!forge_versions?.value.gameVersions.some((x) => item.version === x.id)
		} else if (loader.value === 'quilt') {
			return !!quilt_versions?.value.gameVersions.some((x) => item.version === x.id)
		} else if (loader.value === 'neoforge') {
			return !!neoforge_versions?.value.gameVersions.some((x) => item.version === x.id)
		}

		return []
	})
})

const hasSnapshots = computed(() =>
	gameVersionsForLoader.value?.some((x) => x.version_type !== 'release'),
)

const selectableGameVersionNumbers = computed(() => {
	return gameVersionsForLoader.value
		?.filter((x) => x.version_type === 'release' || showSnapshots.value)
		.map((x) => x.version)
})

const gameVersionOptions = computed(() =>
	(selectableGameVersionNumbers.value ?? []).map((v) => ({ value: v, label: v })),
)

const loaderVersionOptions = computed(() =>
	(selectableLoaderVersions.value ?? []).map((opt, index) => ({ value: index, label: opt.id })),
)

const loaderVersionLabel = computed(() => {
	const idx = loaderVersionIndex.value
	return idx >= 0 && selectableLoaderVersions.value
		? selectableLoaderVersions.value[idx]?.id
		: 'Select version'
})

const selectableLoaderVersions: ComputedRef<ManifestLoaderVersion[] | undefined> = computed(() => {
	if (gameVersion.value) {
		if (loader.value === 'fabric') {
			return fabric_versions?.value.gameVersions[0].loaders
		} else if (loader.value === 'forge') {
			return forge_versions?.value?.gameVersions?.find((item) => item.id === gameVersion.value)
				?.loaders
		} else if (loader.value === 'quilt') {
			return quilt_versions?.value.gameVersions[0].loaders
		} else if (loader.value === 'neoforge') {
			return neoforge_versions?.value?.gameVersions?.find((item) => item.id === gameVersion.value)
				?.loaders
		}
	}
	return []
})
const loaderVersionIndex: Ref<number> = ref(-1)

resetLoaderVersionIndex()

function resetLoaderVersionIndex() {
	loaderVersionIndex.value =
		selectableLoaderVersions.value?.findIndex((x) => x.id === props.instance.loader_version) ?? -1
}

const isValid = computed(() => {
	return (
		selectableGameVersionNumbers.value?.includes(gameVersion.value) &&
		((loaderVersionIndex.value !== undefined && loaderVersionIndex.value >= 0) ||
			loader.value === 'vanilla')
	)
})

const isChanged = computed(() => {
	return (
		loader.value !== props.instance.loader ||
		gameVersion.value !== props.instance.game_version ||
		(loader.value !== 'vanilla' &&
			loaderVersionIndex.value !== undefined &&
			loaderVersionIndex.value >= 0 &&
			selectableLoaderVersions.value?.[loaderVersionIndex.value].id !==
				props.instance.loader_version)
	)
})

watch(loader, () => {
	loaderVersionIndex.value = 0
})

// --- Actions ---

const editing = ref(false)

async function saveGvLoaderEdits() {
	editing.value = true

	const editProfile: { loader?: string; game_version?: string; loader_version?: string } = {}
	editProfile.loader = loader.value
	editProfile.game_version = gameVersion.value

	if (loader.value !== 'vanilla' && loaderVersionIndex.value !== undefined) {
		editProfile.loader_version = selectableLoaderVersions.value?.[loaderVersionIndex.value].id
	} else {
		loaderVersionIndex.value = -1
	}

	await edit(props.instance.path, editProfile).catch(handleError)
	await repairProfile(false)

	editing.value = false
}

const installing = computed(() => props.instance.install_stage !== 'installed')
const repairing = ref(false)
const reinstalling = ref(false)

async function repairProfile(force: boolean) {
	if (force) {
		repairing.value = true
	}
	await install(props.instance.path, force).catch(handleError)
	if (force) {
		repairing.value = false
	}

	trackEvent('InstanceRepair', {
		loader: props.instance.loader,
		game_version: props.instance.game_version,
	})
}

async function repairModpack() {
	reinstalling.value = true
	await update_repair_modrinth(props.instance.path).catch(handleError)
	reinstalling.value = false

	trackEvent('InstanceRepair', {
		loader: props.instance.loader,
		game_version: props.instance.game_version,
	})
}

async function unlinkModpack() {
	await edit(props.instance.path, {
		linked_data: null as unknown as undefined,
	})
}

// --- Context ---

const isBusy = computed(
	() =>
		installing.value ||
		repairing.value ||
		reinstalling.value ||
		isModpackUpdating.value ||
		!!props.offline,
)

provideInstallationSettings({
	isLinked: computed(() => !!props.instance.linked_data?.locked),
	modpack: computed(() =>
		modpackInfo.value
			? {
					title: modpackInfo.value.project.title,
					iconUrl: modpackInfo.value.project.icon_url ?? undefined,
					projectLink: `/project/${modpackInfo.value.project.slug ?? modpackInfo.value.project.id}`,
					versionName: modpackInfo.value.version?.version_number,
					versionLink: modpackInfo.value.version
						? `/project/${modpackInfo.value.project.slug ?? modpackInfo.value.project.id}/version/${modpackInfo.value.version.id}`
						: undefined,
					owner: modpackInfo.value.owner
						? {
								name: modpackInfo.value.owner.name,
								avatarUrl: modpackInfo.value.owner.avatar_url,
								type: modpackInfo.value.owner.type,
								link: () =>
									openUrl(
										`https://modrinth.com/${modpackInfo.value!.owner!.type}/${modpackInfo.value!.owner!.id}`,
									),
							}
						: undefined,
				}
			: null,
	),
	installationInfo: computed(() => {
		const rows = [
			{
				label: formatMessage(messages.platform),
				value: formatLoader(formatMessage, props.instance.loader),
			},
			{ label: formatMessage(messages.gameVersion), value: props.instance.game_version },
		]
		if (props.instance.loader !== 'vanilla' && props.instance.loader_version) {
			rows.push({
				label: formatMessage(messages.loaderVersion, {
					loader: formatLoader(formatMessage, props.instance.loader),
				}),
				value: props.instance.loader_version,
			})
		}
		return rows
	}),
	isBusy,
	changeVersion: () => handleModpackUpdate(),
	unlink: () => confirmUnlinkModal.value?.show(),
	extraLinkedActions: computed(() => [
		{
			label: formatMessage(messages.reinstallModpackButton),
			icon: DownloadIcon,
			color: 'red' as const,
			disabled:
				repairing.value ||
				installing.value ||
				!!props.offline ||
				fetching.value ||
				!modpackInfo.value,
			loading: reinstalling.value,
			loadingLabel: formatMessage(messages.reinstallingModpackButton),
			tooltip: reinstalling.value
				? formatMessage(messages.reinstallingModpackButton)
				: repairing.value
					? formatMessage(messages.cannotWhileRepairing, {
							action: formatMessage(messages.reinstallAction),
						})
					: installing.value
						? formatMessage(messages.cannotWhileInstalling, {
								action: formatMessage(messages.reinstallAction),
							})
						: props.offline
							? formatMessage(messages.cannotWhileOffline, {
									action: formatMessage(messages.reinstallAction),
								})
							: null,
			handler: () => modalConfirmReinstall.value?.show(),
		},
	]),

	// Unlinked state
	platforms: computed(() => loaders?.value?.map((x) => x.name) ?? []),
	selectedPlatform: loader,
	gameVersionOptions,
	selectedGameVersion: gameVersion,
	loaderVersionOptions,
	selectedLoaderVersion: loaderVersionIndex,
	loaderVersionDisplayValue: loaderVersionLabel,
	formattedLoaderName: computed(() => formatLoader(formatMessage, loader.value)),
	hasChanges: isChanged,
	isValid,
	isSaving: editing,
	save: saveGvLoaderEdits,
	showSnapshots,
	hasSnapshots,
})

// --- Messages ---

const messages = defineMessages({
	cannotWhileInstalling: {
		id: 'instance.settings.tabs.installation.tooltip.cannot-while-installing',
		defaultMessage: 'Cannot {action} while installing',
	},
	cannotWhileOffline: {
		id: 'instance.settings.tabs.installation.tooltip.cannot-while-offline',
		defaultMessage: 'Cannot {action} while offline',
	},
	cannotWhileRepairing: {
		id: 'instance.settings.tabs.installation.tooltip.cannot-while-repairing',
		defaultMessage: 'Cannot {action} while repairing',
	},
	repairInstanceTitle: {
		id: 'instance.settings.tabs.installation.repair.title',
		defaultMessage: 'Repair instance',
	},
	repairInstanceDescription: {
		id: 'instance.settings.tabs.installation.repair.description',
		defaultMessage:
			'Reinstalls Minecraft dependencies and checks for corruption. This may resolve issues if your game is not launching due to launcher-related errors.',
	},
	platform: {
		id: 'instance.settings.tabs.installation.platform',
		defaultMessage: 'Platform',
	},
	gameVersion: {
		id: 'instance.settings.tabs.installation.game-version',
		defaultMessage: 'Game version',
	},
	loaderVersion: {
		id: 'instance.settings.tabs.installation.loader-version',
		defaultMessage: '{loader} version',
	},
	repairConfirmTitle: {
		id: 'instance.settings.tabs.installation.repair.confirm.title',
		defaultMessage: 'Repair instance?',
	},
	repairConfirmDescription: {
		id: 'instance.settings.tabs.installation.repair.confirm.description',
		defaultMessage:
			'Repairing reinstalls Minecraft dependencies and checks for corruption. This may resolve issues if your game is not launching due to launcher-related errors, but will not resolve issues or crashes related to installed mods.',
	},
	repairButton: {
		id: 'instance.settings.tabs.installation.repair.button',
		defaultMessage: 'Repair',
	},
	repairingButton: {
		id: 'instance.settings.tabs.installation.repair.button.repairing',
		defaultMessage: 'Repairing',
	},
	repairInProgress: {
		id: 'instance.settings.tabs.installation.repair.in-progress',
		defaultMessage: 'Repair in progress',
	},
	repairAction: {
		id: 'instance.settings.tabs.installation.tooltip.action.repair',
		defaultMessage: 'repair',
	},
	installingButton: {
		id: 'instance.settings.tabs.installation.change-version.button.installing',
		defaultMessage: 'Installing',
	},
	installInProgress: {
		id: 'instance.settings.tabs.installation.install.in-progress',
		defaultMessage: 'Installation in progress',
	},
	installButton: {
		id: 'instance.settings.tabs.installation.change-version.button.install',
		defaultMessage: 'Install',
	},
	alreadyInstalledVanilla: {
		id: 'instance.settings.tabs.installation.change-version.already-installed.vanilla',
		defaultMessage: 'Vanilla {game_version} already installed',
	},
	alreadyInstalledModded: {
		id: 'instance.settings.tabs.installation.change-version.already-installed.modded',
		defaultMessage: '{platform} {version} for Minecraft {game_version} already installed',
	},
	installAction: {
		id: 'instance.settings.tabs.installation.tooltip.action.install',
		defaultMessage: 'install',
	},
	noLoaderVersions: {
		id: 'instance.settings.tabs.installation.no-loader-versions',
		defaultMessage: '{loader} is not available for Minecraft {version}. Try another mod loader.',
	},
	reinstallModpackConfirmTitle: {
		id: 'instance.settings.tabs.installation.reinstall.confirm.title',
		defaultMessage: 'Are you sure you want to reinstall this instance?',
	},
	reinstallModpackConfirmDescription: {
		id: 'instance.settings.tabs.installation.reinstall.confirm.description',
		defaultMessage: `Reinstalling will reset all installed or modified content to what is provided by the modpack, removing any mods or content you have added on top of the original installation. This may fix unexpected behavior if changes have been made to the instance, but if your worlds now depend on additional installed content, it may break existing worlds.`,
	},
	reinstallModpackButton: {
		id: 'instance.settings.tabs.installation.reinstall.button',
		defaultMessage: 'Reinstall modpack',
	},
	reinstallingModpackButton: {
		id: 'instance.settings.tabs.installation.reinstall.button.reinstalling',
		defaultMessage: 'Reinstalling modpack',
	},
	reinstallAction: {
		id: 'instance.settings.tabs.installation.tooltip.action.reinstall',
		defaultMessage: 'reinstall',
	},
	resetSelections: {
		id: 'instance.settings.tabs.installation.reset-selections',
		defaultMessage: 'Reset to current',
	},
})
</script>

<template>
	<InstallationSettingsLayout>
		<template #linked-extra>
			<!-- Repair instance section -->
			<div class="gap-2 flex flex-col">
				<span class="font-semibold text-contrast block">
					{{ formatMessage(messages.repairInstanceTitle) }}
				</span>
				<span class="">
					{{ formatMessage(messages.repairInstanceDescription) }}
				</span>
				<ButtonStyled color="orange" type="outlined">
					<button
						v-tooltip="
							repairing
								? formatMessage(messages.repairInProgress)
								: installing || reinstalling
									? formatMessage(messages.cannotWhileInstalling, {
											action: formatMessage(messages.repairAction),
										})
									: offline
										? formatMessage(messages.cannotWhileOffline, {
												action: formatMessage(messages.repairAction),
											})
										: null
						"
						class="mt-2 !border max-w-fit"
						:disabled="installing || repairing || reinstalling || offline"
						@click="repairConfirmModal.show()"
					>
						<SpinnerIcon v-if="repairing" class="animate-spin" />
						<HammerIcon v-else />
						{{
							repairing
								? formatMessage(messages.repairingButton)
								: formatMessage(messages.repairButton)
						}}
					</button>
				</ButtonStyled>
			</div>
		</template>

		<template #save-button>
			<div class="flex flex-wrap gap-2">
				<ButtonStyled color="brand">
					<button
						v-tooltip="
							installing || reinstalling
								? formatMessage(messages.installInProgress)
								: !isChanged
									? formatMessage(
											loader === 'vanilla'
												? messages.alreadyInstalledVanilla
												: messages.alreadyInstalledModded,
											{
												platform: formatLoader(formatMessage, loader),
												version: instance.loader_version,
												game_version: gameVersion,
											},
										)
									: repairing
										? formatMessage(messages.cannotWhileRepairing, {
												action: formatMessage(messages.installAction),
											})
										: offline
											? formatMessage(messages.cannotWhileOffline, {
													action: formatMessage(messages.installAction),
												})
											: null
						"
						:disabled="!isValid || !isChanged || editing || offline || repairing"
						@click="saveGvLoaderEdits()"
					>
						<SpinnerIcon v-if="editing" class="animate-spin" />
						<DownloadIcon v-else />
						{{
							editing
								? formatMessage(messages.installingButton)
								: formatMessage(messages.installButton)
						}}
					</button>
				</ButtonStyled>
				<ButtonStyled>
					<button
						:disabled="!isChanged"
						@click="
							() => {
								loader = instance.loader
								gameVersion = instance.game_version
								resetLoaderVersionIndex()
							}
						"
					>
						<UndoIcon />
						{{ formatMessage(messages.resetSelections) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</InstallationSettingsLayout>

	<ConfirmModalWrapper
		ref="repairConfirmModal"
		:title="formatMessage(messages.repairConfirmTitle)"
		:description="formatMessage(messages.repairConfirmDescription)"
		:proceed-icon="HammerIcon"
		:proceed-label="formatMessage(messages.repairButton)"
		:danger="false"
		:show-ad-on-close="false"
		@proceed="() => repairProfile(true)"
	/>
	<ConfirmModalWrapper
		ref="modalConfirmReinstall"
		:title="formatMessage(messages.reinstallModpackConfirmTitle)"
		:description="formatMessage(messages.reinstallModpackConfirmDescription)"
		:proceed-icon="DownloadIcon"
		:proceed-label="formatMessage(messages.reinstallModpackButton)"
		:show-ad-on-close="false"
		@proceed="() => repairModpack()"
	/>
	<ConfirmUnlinkModal ref="confirmUnlinkModal" @unlink="unlinkModpack" />
	<Teleport to="body">
		<ContentUpdaterModal
			v-if="updatingModpack"
			ref="contentUpdaterModal"
			:versions="updatingProjectVersions"
			:current-game-version="instance.game_version"
			:current-loader="instance.loader"
			:current-version-id="instance.linked_data?.version_id ?? ''"
			:is-app="true"
			:is-modpack="true"
			:project-icon-url="modpackInfo?.project?.icon_url"
			:project-name="modpackInfo?.project?.title ?? 'Modpack'"
			:loading="loadingVersions"
			:loading-changelog="loadingChangelog"
			@update="handleModalUpdate"
			@cancel="resetUpdateState"
			@version-select="handleVersionSelect"
			@version-hover="handleVersionHover"
		/>
	</Teleport>
</template>
