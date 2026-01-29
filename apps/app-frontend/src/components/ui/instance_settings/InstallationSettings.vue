<script setup lang="ts">
import { DownloadIcon, HammerIcon, IssuesIcon, SpinnerIcon, UndoIcon } from '@modrinth/assets'
import {
	Admonition,
	ButtonStyled,
	Checkbox,
	Chips,
	Combobox,
	defineMessages,
	getTagMessageOrDefault,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import type { GameVersionTag, PlatformTag, Project, Version } from '@modrinth/utils'
import dayjs from 'dayjs'
import { computed, type ComputedRef, type Ref, ref, shallowRef, watch } from 'vue'

import ConfirmModalWrapper from '@/components/ui/modal/ConfirmModalWrapper.vue'
import { trackEvent } from '@/helpers/analytics'
import { get_project, get_version_many } from '@/helpers/cache'
import { get_loader_versions } from '@/helpers/metadata'
import { edit, install, update_repair_modrinth } from '@/helpers/profile'
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

const props = defineProps<InstanceSettingsTabProps>()

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

const modpackProject: Ref<Project | null> = ref(null)
const modpackVersions: Ref<Version[] | null> = ref(null)
const fetching = ref(true)

if (props.instance.linked_data && props.instance.linked_data.project_id && !props.offline) {
	get_project(props.instance.linked_data.project_id, 'must_revalidate')
		.then((project) => {
			modpackProject.value = project

			if (project && project.versions) {
				get_version_many(project.versions, 'must_revalidate')
					.then((versions: Version[]) => {
						modpackVersions.value = versions.sort((a, b) =>
							dayjs(b.date_published).diff(dayjs(a.date_published)),
						)
					})
					.catch(handleError)
					.finally(() => {
						fetching.value = false
					})
			}
		})
		.catch((err) => {
			handleError(err)
			fetching.value = false
		})
} else {
	fetching.value = false
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
	console.log('Editing:')
	console.log(loader.value)

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
	unlinkMovedNotice: {
		id: 'instance.settings.tabs.installation.unlink-moved-notice',
		defaultMessage:
			"Looking to unlink this instance from it's modpack? This option has moved to the Content tab.",
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
	showAllVersions: {
		id: 'instance.settings.tabs.installation.show-all-versions',
		defaultMessage: 'Show all versions',
	},
	install: {
		id: 'instance.settings.tabs.installation.install',
		defaultMessage: 'Install',
	},
	resetSelections: {
		id: 'instance.settings.tabs.installation.reset-selections',
		defaultMessage: 'Reset to current',
	},
	unknownVersion: {
		id: 'instance.settings.tabs.installation.unknown-version',
		defaultMessage: '(unknown version)',
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
	reinstallModpackTitle: {
		id: 'instance.settings.tabs.installation.reinstall.title',
		defaultMessage: 'Reinstall modpack',
	},
	reinstallModpackDescription: {
		id: 'instance.settings.tabs.installation.reinstall.description',
		defaultMessage: `Resets the instance's content to its original state, removing any mods or content you have added on top of the original modpack.`,
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
})
</script>

<template>
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
	<div>
		<Admonition
			v-if="instance.linked_data && instance.linked_data.locked"
			type="info"
			:body="formatMessage(messages.unlinkMovedNotice)"
			class="mb-4"
		/>
		<h2 class="m-0 mb-1 text-lg font-extrabold text-contrast block">
			{{ formatMessage(messages.repairInstanceTitle) }}
		</h2>
		<p class="m-0">
			{{ formatMessage(messages.repairInstanceDescription) }}
		</p>
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
				class="mt-2"
				:disabled="installing || repairing || reinstalling || offline"
				@click="repairConfirmModal.show()"
			>
				<SpinnerIcon v-if="repairing" class="animate-spin" />
				<HammerIcon v-else />
				{{
					repairing ? formatMessage(messages.repairingButton) : formatMessage(messages.repairButton)
				}}
			</button>
		</ButtonStyled>
		<template v-if="!instance.linked_data || !instance.linked_data.locked">
			<h2 class="m-0 mt-4 text-lg font-extrabold text-contrast block">
				{{ formatMessage(messages.platform) }}
			</h2>
			<Chips v-if="loaders" v-model="loader" :items="loaders.map((x) => x.name)" class="mt-2" />
			<h2 class="m-0 mt-4 text-lg font-extrabold text-contrast block">
				{{ formatMessage(messages.gameVersion) }}
			</h2>
			<div class="flex flex-wrap mt-2 gap-2">
				<Combobox
					v-if="selectableGameVersionNumbers !== undefined"
					v-model="gameVersion"
					:options="gameVersionOptions"
					:display-value="gameVersion || formatMessage(messages.unknownVersion)"
				/>
				<Checkbox
					v-if="hasSnapshots"
					v-model="showSnapshots"
					:label="formatMessage(messages.showAllVersions)"
				/>
			</div>
			<template v-if="loader !== 'vanilla'">
				<h2 class="m-0 mt-4 text-lg font-extrabold text-contrast block">
					{{
						formatMessage(messages.loaderVersion, {
							loader: (() => {
								const message = getTagMessageOrDefault(loader, 'loader')
								return typeof message === 'string' ? message : formatMessage(message)
							})(),
						})
					}}
				</h2>
				<Combobox
					v-if="selectableLoaderVersions"
					v-model="loaderVersionIndex"
					:options="loaderVersionOptions"
					:display-value="loaderVersionLabel"
					name="Version selector"
					class="mt-2"
				/>
				<div v-else class="mt-2 text-brand-red flex gap-2 items-center">
					<IssuesIcon />
					{{
						formatMessage(messages.noLoaderVersions, {
							loader: loader,
							version: gameVersion,
						})
					}}
				</div>
			</template>
			<div class="mt-4 flex flex-wrap gap-2">
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
												platform: (() => {
													const message = getTagMessageOrDefault(loader, 'loader')
													return typeof message === 'string' ? message : formatMessage(message)
												})(),
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
		<template v-else-if="instance.linked_data && instance.linked_data.locked && modpackProject">
			<div>
				<h2 class="m-0 mb-1 text-lg font-extrabold text-contrast block mt-4">
					{{ formatMessage(messages.reinstallModpackTitle) }}
				</h2>
				<p class="m-0">
					{{ formatMessage(messages.reinstallModpackDescription) }}
				</p>
			</div>
			<ButtonStyled color="red" type="outlined">
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
					class="mt-2"
					:disabled="repairing || installing || offline || fetching || !modpackVersions"
					@click="modalConfirmReinstall.show()"
				>
					<SpinnerIcon v-if="reinstalling" class="animate-spin" />
					<DownloadIcon v-else />
					{{
						reinstalling
							? formatMessage(messages.reinstallingModpackButton)
							: formatMessage(messages.reinstallModpackButton)
					}}
				</button>
			</ButtonStyled>
		</template>
	</div>
</template>
