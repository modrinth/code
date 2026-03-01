<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	ArrowLeftRightIcon,
	CircleAlertIcon,
	DownloadIcon,
	EyeIcon,
	EyeOffIcon,
	HammerIcon,
	PencilIcon,
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
	ContentUpdaterModal,
	defineMessages,
	formatLoader,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import type { GameVersionTag, PlatformTag } from '@modrinth/utils'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type ComputedRef, nextTick, type Ref, ref, shallowRef, watch } from 'vue'

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

const repairConfirmModal = ref<InstanceType<typeof ConfirmRepairModal>>()
const modalConfirmReinstall = ref<InstanceType<typeof ConfirmReinstallModal>>()
const confirmUnlinkModal = ref<InstanceType<typeof ConfirmUnlinkModal>>()
const contentUpdaterModal = ref<InstanceType<typeof ContentUpdaterModal> | null>()

const props = defineProps<InstanceSettingsTabProps>()

const isLinked = computed(() => !!props.instance.linked_data?.locked)
const isEditing = ref(false)

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

const queryClient = useQueryClient()

const { data: modpackInfo, isFetching: fetching } = useQuery({
	queryKey: computed(() => ['linkedModpackInfo', props.instance.path]),
	queryFn: () => get_linked_modpack_info(props.instance.path, 'must_revalidate'),
	enabled: computed(() => !!props.instance.linked_data?.project_id && !props.offline),
})

const installationInfo = computed(() => {
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
})

const platforms = computed(() => loaders?.value?.map((x) => x.name) ?? [])
const formattedLoaderName = computed(() => formatLoader(formatMessage, loader.value))

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
	isEditing.value = false
}

const installing = computed(() => props.instance.install_stage !== 'installed')
const repairing = ref(false)
const reinstalling = ref(false)

const isBusy = computed(
	() =>
		installing.value ||
		repairing.value ||
		reinstalling.value ||
		isModpackUpdating.value ||
		!!props.offline,
)

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

function cancelEditing() {
	loader.value = props.instance.loader
	gameVersion.value = props.instance.game_version
	resetLoaderVersionIndex()
	isEditing.value = false
}

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
	installationInfoTitle: {
		id: 'instance.settings.tabs.installation.info.title',
		defaultMessage: 'Installation info',
	},
	installedModpackTitle: {
		id: 'instance.settings.tabs.installation.installed-modpack.title',
		defaultMessage: 'Installed modpack',
	},
	changeVersionButton: {
		id: 'instance.settings.tabs.installation.change-version.button',
		defaultMessage: 'Change version',
	},
	linkedInstanceTitle: {
		id: 'instance.settings.tabs.installation.linked-instance.title',
		defaultMessage: 'Linked instance',
	},
	unlinkDescription: {
		id: 'instance.settings.tabs.installation.unlink.description',
		defaultMessage:
			"Unlinking permanently disconnects this instance from the modpack project, allowing you to change the loader and Minecraft version, but you won't receive future updates.",
	},
	unlinkButton: {
		id: 'instance.settings.tabs.installation.unlink.button',
		defaultMessage: 'Unlink modpack',
	},
	reinstallModpackTitle: {
		id: 'instance.settings.tabs.installation.reinstall.title',
		defaultMessage: 'Re-install modpack',
	},
	reinstallModpackDescription: {
		id: 'instance.settings.tabs.installation.reinstall.description',
		defaultMessage:
			'Re-installing the modpack resets the instance\'s content to its original state, removing any mods or content you have added.',
	},
	editWarning: {
		id: 'instance.settings.tabs.installation.edit.warning',
		defaultMessage:
			"We don't recommend editing your installation settings after installing content. If you want to edit them, be cautious as it may cause issues.",
	},
	editButton: {
		id: 'instance.settings.tabs.installation.edit.button',
		defaultMessage: 'Edit',
	},
	cancelButton: {
		id: 'instance.settings.tabs.installation.cancel.button',
		defaultMessage: 'Cancel',
	},
	saveButton: {
		id: 'instance.settings.tabs.installation.save.button',
		defaultMessage: 'Save',
	},
	savingButton: {
		id: 'instance.settings.tabs.installation.saving.button',
		defaultMessage: 'Saving...',
	},
	showAllVersions: {
		id: 'instance.settings.tabs.installation.show-all-versions',
		defaultMessage: 'Show all versions',
	},
	hideSnapshots: {
		id: 'instance.settings.tabs.installation.hide-snapshots',
		defaultMessage: 'Hide snapshots',
	},
})
</script>

<template>
	<div class="flex flex-col gap-6">
		<template v-if="isLinked">
			<div class="flex flex-col gap-2.5">
				<span class="text-lg font-semibold text-contrast">
					{{ formatMessage(messages.installationInfoTitle) }}
				</span>
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
				<span class="text-lg font-semibold text-contrast">
					{{ formatMessage(messages.installedModpackTitle) }}
				</span>
				<div
					v-if="modpackInfo"
					class="flex items-center gap-2.5 rounded-[20px] bg-surface-2 p-3"
				>
					<AutoLink
						:to="`/project/${modpackInfo.project.slug ?? modpackInfo.project.id}`"
						class="shrink-0"
					>
						<div
							class="size-14 shrink-0 overflow-hidden rounded-2xl border border-solid border-surface-5"
						>
							<Avatar
								v-if="modpackInfo.project.icon_url"
								:src="modpackInfo.project.icon_url"
								:alt="modpackInfo.project.title"
								size="100%"
								no-shadow
							/>
						</div>
					</AutoLink>
					<div class="flex flex-col gap-1">
						<AutoLink
							:to="`/project/${modpackInfo.project.slug ?? modpackInfo.project.id}`"
							class="font-semibold text-contrast hover:underline"
						>
							{{ modpackInfo.project.title }}
						</AutoLink>
						<div
							v-if="modpackInfo.version?.version_number"
							class="flex items-center gap-2 text-sm font-medium text-primary"
						>
							<span>{{ modpackInfo.version.version_number }}</span>
						</div>
					</div>
				</div>
				<div class="flex flex-wrap gap-2">
					<ButtonStyled>
						<button class="!shadow-none" :disabled="isBusy" @click="handleModpackUpdate">
							<ArrowLeftRightIcon class="size-5" />
							{{ formatMessage(messages.changeVersionButton) }}
						</button>
					</ButtonStyled>
					<ButtonStyled color="orange">
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
							class="!shadow-none"
							:disabled="isBusy"
							@click="repairConfirmModal.show()"
						>
							<SpinnerIcon v-if="repairing" class="animate-spin" />
							<HammerIcon v-else class="size-5" />
							{{
								repairing
									? formatMessage(messages.repairingButton)
									: formatMessage(messages.repairButton)
							}}
						</button>
					</ButtonStyled>
				</div>
			</div>

			<div class="flex flex-col gap-2.5">
				<span class="text-lg font-semibold text-contrast">
					{{ formatMessage(messages.linkedInstanceTitle) }}
				</span>
				<span class="text-primary">
					{{ formatMessage(messages.unlinkDescription) }}
				</span>
				<div>
					<ButtonStyled color="orange">
						<button class="!shadow-none" :disabled="isBusy" @click="confirmUnlinkModal?.show()">
							<UnlinkIcon class="size-5" />
							{{ formatMessage(messages.unlinkButton) }}
						</button>
					</ButtonStyled>
				</div>
			</div>

			<div class="flex flex-col gap-2.5">
				<span class="text-lg font-semibold text-contrast">
					{{ formatMessage(messages.reinstallModpackTitle) }}
				</span>
				<span class="text-primary">
					{{ formatMessage(messages.reinstallModpackDescription) }}
				</span>
				<div>
					<ButtonStyled color="red">
						<button
							v-tooltip="
								reinstalling
									? formatMessage(messages.reinstallingModpackButton)
									: repairing
										? formatMessage(messages.cannotWhileRepairing, {
												action: formatMessage(messages.reinstallAction),
											})
										: installing
											? formatMessage(messages.cannotWhileInstalling, {
													action: formatMessage(messages.reinstallAction),
												})
											: offline
												? formatMessage(messages.cannotWhileOffline, {
														action: formatMessage(messages.reinstallAction),
													})
												: null
							"
							class="!shadow-none"
							:disabled="isBusy || !modpackInfo"
							@click="modalConfirmReinstall?.show()"
						>
							<SpinnerIcon v-if="reinstalling" class="animate-spin" />
							<DownloadIcon v-else class="size-5" />
							{{
								reinstalling
									? formatMessage(messages.reinstallingModpackButton)
									: formatMessage(messages.reinstallModpackButton)
							}}
						</button>
					</ButtonStyled>
				</div>
			</div>
		</template>

		<template v-else>
			<div class="flex flex-col gap-2.5">
				<span class="text-lg font-semibold text-contrast">
					{{ formatMessage(messages.installationInfoTitle) }}
				</span>

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
						<span class="font-semibold text-contrast">
							{{ formatMessage(messages.platform) }}
						</span>
						<Chips v-model="loader" :items="platforms" />
					</div>

					<div class="flex flex-col gap-2.5">
						<span class="font-semibold text-contrast">
							{{ formatMessage(messages.gameVersion) }}
						</span>
						<Combobox
							v-model="gameVersion"
							:options="gameVersionOptions"
							searchable
							sync-with-selection
							placeholder="Select version"
							search-placeholder="Search game version..."
							:display-value="gameVersion || 'Select version'"
						>
							<template v-if="hasSnapshots" #dropdown-footer>
								<button
									class="flex w-full cursor-pointer items-center justify-center gap-1.5 border-0 border-t border-solid border-surface-5 bg-transparent py-3 text-center text-sm font-semibold text-secondary transition-colors hover:text-contrast"
									@mousedown.prevent
									@click="showSnapshots = !showSnapshots"
								>
									<EyeOffIcon v-if="showSnapshots" class="size-4" />
									<EyeIcon v-else class="size-4" />
									{{
										showSnapshots
											? formatMessage(messages.hideSnapshots)
											: formatMessage(messages.showAllVersions)
									}}
								</button>
							</template>
						</Combobox>
					</div>

					<div v-if="loader !== 'vanilla'" class="flex flex-col gap-2.5">
						<span class="font-semibold text-contrast">
							{{ formatMessage(messages.loaderVersion, { loader: formattedLoaderName }) }}
						</span>
						<Combobox
							v-model="loaderVersionIndex"
							searchable
							sync-with-selection
							:placeholder="loaderVersionLabel"
							search-placeholder="Search version..."
							:options="loaderVersionOptions"
							:display-value="loaderVersionLabel"
						/>
					</div>

					<div class="flex flex-wrap gap-2">
						<ButtonStyled color="brand">
							<button
								class="!shadow-none"
								:disabled="!isValid || !isChanged || editing || !!offline || repairing"
								@click="saveGvLoaderEdits()"
							>
								<SpinnerIcon v-if="editing" class="animate-spin" />
								<SaveIcon v-else />
								{{
									editing
										? formatMessage(messages.savingButton)
										: formatMessage(messages.saveButton)
								}}
							</button>
						</ButtonStyled>
						<ButtonStyled type="outlined">
							<button class="!border !border-surface-5 !shadow-none" @click="cancelEditing">
								<XIcon />
								{{ formatMessage(messages.cancelButton) }}
							</button>
						</ButtonStyled>
					</div>
				</div>
			</div>

			<template v-if="!isEditing">
				<div class="flex items-start gap-2">
					<CircleAlertIcon class="mt-0.5 size-5 shrink-0 text-orange" />
					<span class="text-primary">
						{{ formatMessage(messages.editWarning) }}
					</span>
				</div>
				<div class="flex flex-wrap gap-2">
					<ButtonStyled color="orange">
						<button class="!shadow-none" :disabled="isBusy" @click="isEditing = true">
							<PencilIcon class="size-5" />
							{{ formatMessage(messages.editButton) }}
						</button>
					</ButtonStyled>
				</div>
			</template>

			<div class="flex flex-col gap-2.5">
				<span class="text-lg font-semibold text-contrast">
					{{ formatMessage(messages.repairInstanceTitle) }}
				</span>
				<span class="text-primary">
					{{ formatMessage(messages.repairInstanceDescription) }}
				</span>
				<div>
					<ButtonStyled color="orange">
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
							class="!shadow-none"
							:disabled="isBusy"
							@click="repairConfirmModal.show()"
						>
							<SpinnerIcon v-if="repairing" class="animate-spin" />
							<HammerIcon v-else class="size-5" />
							{{
								repairing
									? formatMessage(messages.repairingButton)
									: formatMessage(messages.repairButton)
							}}
						</button>
					</ButtonStyled>
				</div>
			</div>
		</template>
	</div>

	<ConfirmRepairModal ref="repairConfirmModal" @repair="() => repairProfile(true)" />
	<ConfirmReinstallModal ref="modalConfirmReinstall" @reinstall="repairModpack" />
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
