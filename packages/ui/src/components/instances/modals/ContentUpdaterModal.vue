<template>
	<NewModal
		ref="modal"
		:max-width="'min(928px, calc(95vw - 10rem))'"
		:width="'min(928px, calc(95vw - 10rem))'"
		no-padding
	>
		<template #title>
			<Avatar v-if="projectIconUrl" :src="projectIconUrl" size="3rem" :tint-by="projectName" />
			<span class="text-lg font-extrabold text-contrast">{{
				header ??
				formatMessage(
					isModpack ? messages.switchModpackVersionHeader : messages.updateVersionHeader,
				)
			}}</span>
		</template>
		<div
			class="flex h-[min(550px,calc(95vh-10rem))] border-solid border-transparent border-[1px] border-b-surface-4"
		>
			<div class="w-[300px] flex flex-col relative">
				<div class="p-4 pb-2">
					<StyledInput
						v-model="searchQuery"
						:icon="SearchIcon"
						type="text"
						:placeholder="formatMessage(messages.searchVersionPlaceholder)"
						wrapper-class="w-full"
					/>
				</div>

				<div class="flex-1 overflow-y-auto px-4 pb-16">
					<div v-if="loading" class="flex flex-col items-center justify-center h-full gap-2">
						<SpinnerIcon class="h-8 w-8 animate-spin text-secondary" />
						<span class="text-sm text-secondary">{{
							formatMessage(messages.loadingVersions)
						}}</span>
					</div>
					<template v-else>
						<div class="flex flex-col gap-1.5">
							<button
								v-for="version in filteredVersions"
								:key="version.id"
								class="flex items-center h-10 px-4 py-2.5 rounded-xl border-none cursor-pointer transition-colors"
								:class="[
									selectedVersion?.id === version.id
										? 'bg-brand-highlight'
										: 'bg-transparent hover:bg-button-bg',
								]"
								@mouseenter="emit('versionHover', version)"
								@click="handleVersionSelect(version)"
							>
								<div class="flex items-center justify-between w-full gap-2">
									<div class="flex items-center gap-2 min-w-0">
										<VersionChannelIndicator
											:channel="version.version_type"
											size="sm"
											class="shrink-0"
										/>
										<span
											v-tooltip="version.version_number"
											class="font-semibold text-contrast truncate"
										>
											{{ version.version_number }}
										</span>
									</div>
									<span
										v-if="shouldShowBadge(version)"
										class="rounded-full text-sm font-medium flex items-center flex-shrink-0 border border-solid"
										:class="[
											getBadgeClasses(version),
											isVersionCompatible(version) ? 'px-2.5 py-0.5' : 'p-1',
										]"
									>
										<CircleAlertIcon
											v-if="!isVersionCompatible(version)"
											v-tooltip="formatMessage(messages.incompatibleBadge)"
											class="size-4"
										/>
										<template v-else>{{ getBadgeLabel(version) }}</template>
									</span>
								</div>
							</button>
						</div>
						<div
							v-if="filteredVersions.length === 0"
							class="p-4 text-center text-secondary text-sm"
						>
							{{ formatMessage(messages.noVersionsFound) }}
						</div>
					</template>
				</div>

				<div
					class="absolute bottom-0 left-0 right-0 pointer-events-none flex flex-col items-center justify-end bg-gradient-to-b from-transparent to-bg-raised to-70% pb-3 h-24"
				>
					<div class="pointer-events-auto">
						<ButtonStyled type="transparent" :circular="true">
							<button
								class="flex items-center gap-1.5"
								@click="hideIncompatibleState = !hideIncompatibleState"
							>
								<EyeIcon v-if="hideIncompatibleState" class="h-6 w-6" />
								<EyeOffIcon v-else class="h-6 w-6" />
								<span class="font-medium">{{
									hideIncompatibleState
										? formatMessage(messages.showIncompatible)
										: formatMessage(messages.hideIncompatible)
								}}</span>
							</button>
						</ButtonStyled>
					</div>
				</div>
			</div>

			<div class="w-px bg-divider" />

			<div class="flex-1 flex flex-col min-w-0 relative">
				<template v-if="selectedVersion">
					<div class="bg-bg p-4">
						<div class="flex flex-col gap-1.5">
							<div class="flex items-center justify-between">
								<div class="flex items-center gap-2">
									<span class="font-semibold text-xl text-contrast">
										{{ selectedVersion.version_number }}
									</span>
									<span
										class="px-2.5 py-0.5 rounded-full text-sm font-medium flex items-center flex-shrink-0 border border-solid"
										:class="getVersionTypeBadgeClasses(selectedVersion)"
									>
										{{ capitalizeString(selectedVersion.version_type) }}
									</span>
								</div>
								<span class="font-medium text-primary">
									{{ formatLongDate(selectedVersion.date_published) }}
								</span>
							</div>
							<div class="flex items-center gap-2">
								<div class="flex items-center gap-2 rounded-xl">
									<FileTextIcon class="h-6 w-6 text-primary" />
									<span class="font-medium text-primary">{{
										formatMessage(commonMessages.changelogLabel)
									}}</span>
								</div>
								<span class="w-1.5 h-1.5 rounded-full bg-divider" />
								<span class="font-medium text-primary">
									{{ formatLoaderGameVersion(selectedVersion) }}
								</span>
							</div>
						</div>
					</div>

					<div class="h-px bg-divider" />

					<div class="flex-1 bg-bg p-4 overflow-y-auto">
						<div
							v-if="loadingChangelog"
							class="flex flex-col items-center justify-center h-full gap-2"
						>
							<SpinnerIcon class="h-6 w-6 animate-spin text-secondary" />
							<span class="text-sm text-secondary">{{
								formatMessage(messages.loadingChangelog)
							}}</span>
						</div>
						<div
							v-else-if="selectedVersion.changelog"
							class="markdown"
							v-html="renderHighlightedString(selectedVersion.changelog)"
						/>
						<div v-else class="text-secondary italic">
							{{ formatMessage(messages.noChangelog) }}
						</div>
					</div>

					<div
						class="absolute bottom-0 left-0 right-0 h-14 bg-gradient-to-t from-bg to-transparent pointer-events-none"
					/>
				</template>
				<div v-else class="flex-1 flex items-center justify-center text-secondary bg-bg">
					{{ formatMessage(messages.selectVersionPrompt) }}
				</div>
			</div>
		</div>

		<div
			class="w-full flex flex-row items-center gap-4 p-4 border-solid border-x-0 border-b-0 border-t border-surface-4"
		>
			<div class="flex flex-row items-center gap-2 max-w-[55%] flex-1 text-orange mr-auto">
				<TriangleAlertIcon class="size-6 shrink-0" />
				<span>{{
					formatMessage(isApp ? messages.updateWarningApp : messages.updateWarningWeb)
				}}</span>
			</div>
			<div class="flex flex-row gap-2 shrink-0">
				<ButtonStyled type="outlined">
					<button class="!border-[1px] !border-surface-4" @click="handleCancel">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button
						:disabled="!selectedVersion || selectedVersion.id === currentVersionId"
						@click="handleUpdate"
					>
						<DownloadIcon />
						{{
							formatMessage(isDowngrade ? messages.downgradeToVersion : messages.updateToVersion, {
								version: selectedVersion?.version_number ?? '...',
							})
						}}
					</button>
				</ButtonStyled>
			</div>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	CircleAlertIcon,
	DownloadIcon,
	EyeIcon,
	EyeOffIcon,
	FileTextIcon,
	SearchIcon,
	SpinnerIcon,
	TriangleAlertIcon,
	XIcon,
} from '@modrinth/assets'
import { capitalizeString, renderHighlightedString } from '@modrinth/utils'
import { computed, ref, watch } from 'vue'

import { defineMessages, useVIntl } from '../../../composables/i18n'
import { commonMessages } from '../../../utils/common-messages'
import Avatar from '../../base/Avatar.vue'
import ButtonStyled from '../../base/ButtonStyled.vue'
import StyledInput from '../../base/StyledInput.vue'
import NewModal from '../../modal/NewModal.vue'
import VersionChannelIndicator from '../../version/VersionChannelIndicator.vue'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	updateVersionHeader: {
		id: 'instances.updater-modal.header',
		defaultMessage: 'Update version',
	},
	switchModpackVersionHeader: {
		id: 'instances.updater-modal.header-modpack',
		defaultMessage: 'Switch modpack version',
	},
	searchVersionPlaceholder: {
		id: 'instances.updater-modal.search-placeholder',
		defaultMessage: 'Search version...',
	},
	noVersionsFound: {
		id: 'instances.updater-modal.no-versions',
		defaultMessage: 'No versions found',
	},
	showIncompatible: {
		id: 'instances.updater-modal.show-incompatible',
		defaultMessage: 'Show incompatible',
	},
	hideIncompatible: {
		id: 'instances.updater-modal.hide-incompatible',
		defaultMessage: 'Hide incompatible',
	},
	noChangelog: {
		id: 'instances.updater-modal.no-changelog',
		defaultMessage: 'No changelog provided for this version.',
	},
	selectVersionPrompt: {
		id: 'instances.updater-modal.select-version',
		defaultMessage: 'Select a version to view its changelog',
	},
	updateWarningApp: {
		id: 'instances.updater-modal.warning-app',
		defaultMessage:
			'Updating can break your instance. Review version changelogs and back up first.',
	},
	updateWarningWeb: {
		id: 'instances.updater-modal.warning-web',
		defaultMessage: 'Updating can break your world. Review version changelogs and back up first.',
	},
	downgradeToVersion: {
		id: 'instances.updater-modal.downgrade-to',
		defaultMessage: 'Downgrade to {version}',
	},
	updateToVersion: {
		id: 'instances.updater-modal.update-to',
		defaultMessage: 'Update to {version}',
	},
	currentBadge: {
		id: 'instances.updater-modal.badge.current',
		defaultMessage: 'Current',
	},
	incompatibleBadge: {
		id: 'instances.updater-modal.badge.incompatible',
		defaultMessage: 'Incompatible',
	},
	loadingVersions: {
		id: 'instances.updater-modal.loading-versions',
		defaultMessage: 'Loading versions...',
	},
	loadingChangelog: {
		id: 'instances.updater-modal.loading-changelog',
		defaultMessage: 'Loading changelog...',
	},
})

const props = withDefaults(
	defineProps<{
		versions: Labrinth.Versions.v2.Version[]
		currentGameVersion: string
		currentLoader: string
		currentVersionId: string
		isApp: boolean
		/** Whether this is a modpack update (changes header text) */
		isModpack?: boolean
		projectIconUrl?: string
		projectName?: string
		header?: string
		/** Whether versions are currently being loaded */
		loading?: boolean
		/** Whether changelog is being loaded for the selected version */
		loadingChangelog?: boolean
	}>(),
	{
		isModpack: false,
		projectIconUrl: undefined,
		projectName: undefined,
		header: undefined,
		loading: false,
		loadingChangelog: false,
	},
)

const emit = defineEmits<{
	update: [version: Labrinth.Versions.v2.Version]
	cancel: []
	/** Emitted when user selects a version, so parent can fetch full version data with changelog */
	versionSelect: [version: Labrinth.Versions.v2.Version]
	versionHover: [version: Labrinth.Versions.v2.Version]
}>()

const modal = ref<InstanceType<typeof NewModal>>()
const searchQuery = ref('')
const hideIncompatibleState = ref(true)
const selectedVersion = ref<Labrinth.Versions.v2.Version | null>(null)
// Store the initial version ID to select when versions become available
const pendingInitialVersionId = ref<string | undefined>(undefined)

watch(
	() => props.versions,
	(newVersions) => {
		// If we have a selected version, check if it was updated with new data (e.g., changelog)
		if (selectedVersion.value) {
			const updatedVersion = newVersions.find((v) => v.id === selectedVersion.value?.id)
			if (updatedVersion && updatedVersion !== selectedVersion.value) {
				selectedVersion.value = updatedVersion
			}
		}

		// Handle initial selection when versions first arrive
		if (newVersions.length > 0 && !selectedVersion.value && pendingInitialVersionId.value) {
			const version =
				newVersions.find((v) => v.id === pendingInitialVersionId.value) ?? newVersions[0]
			selectedVersion.value = version
			if (version) {
				emit('versionSelect', version)
			}
			pendingInitialVersionId.value = undefined
		}
	},
	{ deep: true },
)

function isVersionCompatible(version: Labrinth.Versions.v2.Version): boolean {
	const hasGameVersion = version.game_versions.includes(props.currentGameVersion)
	const hasLoader = version.loaders.some(
		(loader) => loader.toLowerCase() === props.currentLoader.toLowerCase(),
	)
	return hasGameVersion && hasLoader
}

const currentVersion = computed(() => props.versions.find((v) => v.id === props.currentVersionId))

const isDowngrade = computed(() => {
	if (!selectedVersion.value || !currentVersion.value) return false
	return (
		new Date(selectedVersion.value.date_published) < new Date(currentVersion.value.date_published)
	)
})

const filteredVersions = computed(() => {
	let versions = [...props.versions]

	if (searchQuery.value) {
		const query = searchQuery.value.toLowerCase()
		versions = versions.filter(
			(v) => v.name.toLowerCase().includes(query) || v.version_number.toLowerCase().includes(query),
		)
	}

	if (hideIncompatibleState.value) {
		versions = versions.filter(isVersionCompatible)
	}

	return versions
})

function shouldShowBadge(version: Labrinth.Versions.v2.Version): boolean {
	return version.id === props.currentVersionId || !isVersionCompatible(version)
}

function getBadgeLabel(version: Labrinth.Versions.v2.Version): string {
	if (version.id === props.currentVersionId) return formatMessage(messages.currentBadge)
	if (!isVersionCompatible(version)) return formatMessage(messages.incompatibleBadge)
	return ''
}

function getBadgeClasses(version: Labrinth.Versions.v2.Version): string {
	// Current badge
	if (version.id === props.currentVersionId) {
		return 'bg-surface-4 border-surface-5 text-primary'
	}

	// Incompatible badge (takes precedence over version type)
	if (!isVersionCompatible(version)) {
		return 'bg-highlight-orange border-brand-orange text-brand-orange'
	}

	// Version type badges
	switch (version.version_type) {
		case 'release':
			return 'bg-highlight-green border-brand text-brand'
		case 'beta':
			return 'bg-highlight-blue border-brand-blue text-brand-blue'
		case 'alpha':
			return 'bg-highlight-purple border-brand-purple text-brand-purple'
		default:
			return 'bg-surface-4 border-surface-5 text-primary'
	}
}

function getVersionTypeBadgeClasses(version: Labrinth.Versions.v2.Version): string {
	switch (version.version_type) {
		case 'release':
			return 'bg-highlight-green border-brand text-brand'
		case 'beta':
			return 'bg-highlight-blue border-brand-blue text-brand-blue'
		case 'alpha':
			return 'bg-highlight-purple border-brand-purple text-brand-purple'
		default:
			return 'bg-surface-4 border-surface-5 text-primary'
	}
}

function formatLongDate(dateString: string): string {
	return new Date(dateString).toLocaleDateString('en-US', {
		year: 'numeric',
		month: 'long',
		day: 'numeric',
	})
}

function formatLoaderGameVersion(version: Labrinth.Versions.v2.Version): string {
	const loader = capitalizeString(version.loaders[0] || '')
	const gameVersion = version.game_versions[0] || ''
	return `${loader} ${gameVersion}`
}

function handleVersionSelect(version: Labrinth.Versions.v2.Version) {
	selectedVersion.value = version
	// Emit event so parent can fetch full version data with changelog
	emit('versionSelect', version)
}

function handleUpdate() {
	if (selectedVersion.value) {
		emit('update', selectedVersion.value)
		hide()
	}
}

function handleCancel() {
	emit('cancel')
	hide()
}

function show(initialVersionId?: string) {
	searchQuery.value = ''
	hideIncompatibleState.value = true

	if (props.versions.length > 0) {
		if (initialVersionId) {
			selectedVersion.value =
				props.versions.find((v) => v.id === initialVersionId) ?? props.versions[0]
		} else {
			selectedVersion.value = props.versions[0]
		}
		pendingInitialVersionId.value = undefined
	} else {
		selectedVersion.value = null
		pendingInitialVersionId.value = initialVersionId
	}

	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

defineExpose({ show, hide })
</script>
