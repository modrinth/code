<template>
	<NewModal
		ref="modal"
		:max-width="'min(928px, calc(95vw - 10rem))'"
		:width="'min(928px, calc(95vw - 10rem))'"
		:on-hide="handleModalHide"
		no-padding
	>
		<template #title>
			<Avatar v-if="projectIconUrl" :src="projectIconUrl" size="3rem" :tint-by="projectName" />
			<span class="text-lg font-extrabold text-contrast">{{ header ?? defaultHeader }}</span>
		</template>
		<div
			class="flex h-[min(550px,calc(95vh-10rem))] border-solid border-transparent border-[1px] border-b-surface-4"
		>
			<div class="w-[300px] flex flex-col relative bg-surface-3">
				<div class="p-4 pb-2">
					<StyledInput
						v-model="searchQuery"
						:icon="SearchIcon"
						type="text"
						:placeholder="formatMessage(messages.searchVersionPlaceholder)"
						wrapper-class="w-full"
					/>
				</div>

				<div class="flex-1 overflow-y-auto px-4" :class="isModpack ? 'pb-4' : 'pb-16'">
					<div v-if="loading" class="flex flex-col items-center justify-center h-full gap-2">
						<SpinnerIcon class="h-8 w-8 animate-spin text-secondary" />
						<span class="text-sm text-secondary">{{
							formatMessage(messages.loadingVersions)
						}}</span>
					</div>
					<template v-else>
						<div class="flex flex-col gap-1.5" role="listbox">
							<button
								v-for="version in filteredVersions"
								:key="version.id"
								role="option"
								:aria-selected="selectedVersion?.id === version.id"
								class="flex items-center h-10 px-4 py-2.5 rounded-xl border-none cursor-pointer transition-colors"
								:class="[
									selectedVersion?.id === version.id
										? 'bg-brand-highlight'
										: 'bg-transparent hover:bg-button-bg',
								]"
								@mouseenter="handleVersionMouseEnter(version)"
								@mouseleave="handleVersionMouseLeave"
								@focus="emit('versionHover', version)"
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
											shouldShowIncompatibleBadge(version) ? 'p-1' : 'px-2.5 py-0.5',
										]"
									>
										<CircleAlertIcon
											v-if="shouldShowIncompatibleBadge(version)"
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
					v-if="!isModpack && !incompatibilityWarningMode"
					class="absolute bottom-0 left-0 right-0 pointer-events-none flex flex-col items-center justify-end bg-gradient-to-b from-transparent to-bg-raised to-70% pb-3 h-24"
				>
					<div class="pointer-events-auto">
						<ButtonStyled type="transparent" :circular="true">
							<button
								class="flex items-center gap-1.5"
								:aria-label="
									hideIncompatibleState
										? formatMessage(messages.showIncompatible)
										: formatMessage(messages.hideIncompatible)
								"
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

			<div class="flex-1 flex flex-col min-w-0 relative bg-surface-1" aria-live="polite">
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
							class="markdown [&_img]:max-w-full [&_img]:h-auto"
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
			<div
				v-if="showUpdateWarning"
				class="flex flex-row items-center gap-2 max-w-[55%] flex-1 text-orange mr-auto"
			>
				<TriangleAlertIcon class="size-6 shrink-0" />
				<span>{{
					warning ??
					formatMessage(
						incompatibilityWarningMode
							? messages.incompatibilityWarning
							: messages.updateWarning,
						{
							type: updateWarningTargetType,
						},
					)
				}}</span>
			</div>
			<div class="flex flex-row gap-2 shrink-0 ml-auto">
				<ButtonStyled type="outlined">
					<button @click="handleCancel">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled :color="incompatibilityWarningMode ? 'orange' : 'brand'">
					<button
						v-tooltip="props.actionDisabled ? props.actionDisabledTooltip : undefined"
						:disabled="
							actionLoading ||
							props.actionDisabled ||
							!selectedVersion ||
							(!incompatibilityWarningMode && selectedVersion.id === currentVersionId)
						"
						@click="handleUpdate"
					>
						<SpinnerIcon v-if="actionLoading" class="size-5 animate-spin" />
						<DownloadIcon v-else />
						{{
							actionLoading
								? formatMessage(commonMessages.installingLabel)
								: incompatibilityWarningMode
									? formatMessage(messages.installAnywayButton)
									: formatMessage(
											isDowngrade
												? messages.downgradeToVersion
												: switchMode
													? messages.switchToVersion
													: messages.updateToVersion,
											{
												version: selectedVersion?.version_number ?? '...',
											},
										)
						}}
					</button>
				</ButtonStyled>
			</div>
		</div>
	</NewModal>

	<ConfirmModal
		ref="incompatibleUpdateModal"
		:title="formatMessage(messages.incompatibleUpdateHeader)"
		:description="
			formatMessage(messages.incompatibleUpdateDescription, {
				version: pendingIncompatibleUpdate?.version.version_number ?? '...',
			})
		"
		:proceed-icon="DownloadIcon"
		:proceed-label="formatMessage(messages.updateAnywayButton)"
		:danger="false"
		:markdown="false"
		@proceed="confirmIncompatibleUpdate"
	/>
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
import {
	capitalizeString,
	formatVersionsForDisplay,
	type GameVersionTag,
	renderHighlightedString,
} from '@modrinth/utils'
import { useTimeoutFn } from '@vueuse/core'
import { computed, ref, watch } from 'vue'

import Avatar from '#ui/components/base/Avatar.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import StyledInput from '#ui/components/base/StyledInput.vue'
import ConfirmModal from '#ui/components/modal/ConfirmModal.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import VersionChannelIndicator from '#ui/components/version/VersionChannelIndicator.vue'
import { useDebugLogger } from '#ui/composables/debug-logger'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { injectTags } from '#ui/providers'
import { commonMessages } from '#ui/utils/common-messages'
import {
	versionChangesGameVersion,
	versionMatchesCompatibilityTarget,
} from '#ui/utils/version-compatibility'

const { formatMessage } = useVIntl()
const debug = useDebugLogger('ContentUpdaterModal')
const tags = injectTags(null)

const messages = defineMessages({
	updateVersionHeader: {
		id: 'instances.updater-modal.header',
		defaultMessage: 'Update version',
	},
	incompatibilityWarningHeader: {
		id: 'instances.updater-modal.incompatibility-warning-header',
		defaultMessage: 'Choose version',
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
	updateWarning: {
		id: 'instances.updater-modal.warning',
		defaultMessage:
			'Updating can break your {type, select, server {server} other {instance}}. Review version changelogs and back up first.',
	},
	incompatibilityWarning: {
		id: 'instances.updater-modal.incompatibility-warning',
		defaultMessage:
			'This version is not marked as compatible with this instance. Dependencies will not be installed automatically.',
	},
	downgradeToVersion: {
		id: 'instances.updater-modal.downgrade-to',
		defaultMessage: 'Downgrade to {version}',
	},
	updateToVersion: {
		id: 'instances.updater-modal.update-to',
		defaultMessage: 'Update to {version}',
	},
	switchVersionHeader: {
		id: 'instances.updater-modal.header-switch',
		defaultMessage: 'Switch version',
	},
	switchToVersion: {
		id: 'instances.updater-modal.switch-to',
		defaultMessage: 'Switch to {version}',
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
	incompatibleUpdateHeader: {
		id: 'instances.updater-modal.incompatible-update.header',
		defaultMessage: 'Update to incompatible version?',
	},
	incompatibleUpdateDescription: {
		id: 'instances.updater-modal.incompatible-update.description',
		defaultMessage:
			'{version} is not marked as compatible with this installation. It may fail to launch or behave unexpectedly.',
	},
	updateAnywayButton: {
		id: 'instances.updater-modal.incompatible-update.proceed',
		defaultMessage: 'Update anyway',
	},
	installAnywayButton: {
		id: 'instances.updater-modal.install-anyway',
		defaultMessage: 'Install anyway',
	},
})

const props = withDefaults(
	defineProps<{
		versions: Labrinth.Versions.v2.Version[]
		currentGameVersion: string
		currentLoader: string
		currentVersionId: string
		isApp: boolean
		targetType?: 'server' | 'instance'
		/** The project type (e.g. mod, shader, resourcepack, datapack, modpack). */
		projectType?: string
		projectIconUrl?: string
		projectName?: string
		header?: string
		mode?: 'version' | 'incompatibility-warning'
		warning?: string
		actionLoading?: boolean
		/** Whether versions are currently being loaded */
		loading?: boolean
		/** Whether changelog is being loaded for the selected version */
		loadingChangelog?: boolean
		actionDisabled?: boolean
		actionDisabledTooltip?: string
	}>(),
	{
		targetType: undefined,
		projectType: undefined,
		projectIconUrl: undefined,
		projectName: undefined,
		header: undefined,
		mode: 'version',
		warning: undefined,
		actionLoading: false,
		loading: false,
		loadingChangelog: false,
		actionDisabled: false,
		actionDisabledTooltip: undefined,
	},
)

const isModpack = computed(() => props.projectType === 'modpack')
const incompatibilityWarningMode = computed(() => props.mode === 'incompatibility-warning')
const defaultHeader = computed(() => {
	if (incompatibilityWarningMode.value) {
		return formatMessage(messages.incompatibilityWarningHeader)
	}

	return formatMessage(
		isModpack.value
			? messages.switchModpackVersionHeader
			: switchMode.value
				? messages.switchVersionHeader
				: messages.updateVersionHeader,
	)
})
const updateWarningTargetType = computed(() => props.targetType ?? 'instance')

const emit = defineEmits<{
	update: [version: Labrinth.Versions.v2.Version, event: MouseEvent]
	cancel: []
	/** Emitted when user selects a version, so parent can fetch full version data with changelog */
	versionSelect: [version: Labrinth.Versions.v2.Version]
	versionHover: [version: Labrinth.Versions.v2.Version]
}>()

const modal = ref<InstanceType<typeof NewModal>>()
const incompatibleUpdateModal = ref<InstanceType<typeof ConfirmModal>>()
const searchQuery = ref('')
const hideIncompatibleState = ref(true)
const switchMode = ref(false)
const selectedVersion = ref<Labrinth.Versions.v2.Version | null>(null)
const pendingIncompatibleUpdate = ref<{
	version: Labrinth.Versions.v2.Version
	event: MouseEvent
} | null>(null)
const suppressCancelOnHide = ref(false)
// Store the initial version ID to select when versions become available
const pendingInitialVersionId = ref<string | undefined>(undefined)
const pinnedInitialVersionId = ref<string | undefined>(undefined)

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
			const pendingFound = newVersions.find((v) => v.id === pendingInitialVersionId.value)
			debug('versions watcher: initial selection', {
				pendingInitialVersionId: pendingInitialVersionId.value,
				foundPending: !!pendingFound,
				currentVersionId: props.currentVersionId,
				currentInList: newVersions.some((v) => v.id === props.currentVersionId),
				totalVersions: newVersions.length,
				loaderDistribution: [...new Set(newVersions.flatMap((v) => v.loaders))],
				gameVersionDistribution: [...new Set(newVersions.flatMap((v) => v.game_versions))].slice(
					0,
					10,
				),
			})
			const version = pendingFound ?? newVersions[0]
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
	const compatible = versionMatchesCompatibilityTarget(version, {
		gameVersion: props.currentGameVersion,
		loader: props.currentLoader,
		projectType: props.projectType,
	})

	if (!compatible) {
		debug('isVersionCompatible: INCOMPATIBLE', {
			versionId: version.id,
			versionNumber: version.version_number,
			versionLoaders: version.loaders,
			versionGameVersions: version.game_versions,
			currentLoader: props.currentLoader,
			currentGameVersion: props.currentGameVersion,
			projectType: props.projectType,
		})
	}
	return compatible
}

const currentVersion = computed(() => props.versions.find((v) => v.id === props.currentVersionId))
const showUpdateWarning = computed(() => !isModpack.value)

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
			(v) =>
				v.name.toLowerCase().includes(query) ||
				v.version_number.toLowerCase().includes(query) ||
				(incompatibilityWarningMode.value &&
					[...v.loaders, ...v.game_versions].some((value) => value.toLowerCase().includes(query))),
		)
	}

	const beforeFilterCount = versions.length
	if (!incompatibilityWarningMode.value && !isModpack.value && hideIncompatibleState.value) {
		versions = versions.filter(
			(version) =>
				version.id === props.currentVersionId ||
				version.id === selectedVersion.value?.id ||
				version.id === pinnedInitialVersionId.value ||
				isVersionCompatible(version),
		)
	}

	debug('filteredVersions computed', {
		totalVersions: props.versions.length,
		afterSearchFilter: beforeFilterCount,
		afterCompatibilityFilter: versions.length,
		hiddenByCompatibility: beforeFilterCount - versions.length,
		hideIncompatible: hideIncompatibleState.value,
		filteringCompatibility: !isModpack.value && hideIncompatibleState.value,
	})

	return versions
})

function shouldShowBadge(version: Labrinth.Versions.v2.Version): boolean {
	if (incompatibilityWarningMode.value) return false
	return version.id === props.currentVersionId || shouldShowIncompatibleBadge(version)
}

function shouldShowIncompatibleBadge(version: Labrinth.Versions.v2.Version): boolean {
	return version.id !== props.currentVersionId && !isModpack.value && !isVersionCompatible(version)
}

function getBadgeLabel(version: Labrinth.Versions.v2.Version): string {
	if (version.id === props.currentVersionId) return formatMessage(messages.currentBadge)
	if (shouldShowIncompatibleBadge(version)) return formatMessage(messages.incompatibleBadge)
	return ''
}

function getBadgeClasses(version: Labrinth.Versions.v2.Version): string {
	// Current badge
	if (version.id === props.currentVersionId) {
		return 'bg-surface-4 border-surface-5 text-primary'
	}

	if (shouldShowIncompatibleBadge(version)) {
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
	const gameVersions = formatGameVersions(version)
	return [loader, gameVersions].filter(Boolean).join(' ')
}

function formatGameVersions(version: Labrinth.Versions.v2.Version): string {
	if (!incompatibilityWarningMode.value) {
		return version.game_versions[0] || ''
	}

	const gameVersions = tags?.gameVersions.value?.length
		? formatVersionsForDisplay(version.game_versions, tags.gameVersions.value as GameVersionTag[])
		: version.game_versions

	return gameVersions.join(', ')
}

let prefetchTimeout: ReturnType<typeof useTimeoutFn> | null = null
const HOVER_DURATION_TO_PREFETCH_MS = 500
function handleVersionMouseEnter(version: Labrinth.Versions.v2.Version) {
	prefetchTimeout = useTimeoutFn(
		() => emit('versionHover', version),
		HOVER_DURATION_TO_PREFETCH_MS,
		{ immediate: false },
	)
	prefetchTimeout.start()
}

function handleVersionMouseLeave() {
	if (prefetchTimeout) prefetchTimeout.stop()
}

function handleVersionSelect(version: Labrinth.Versions.v2.Version) {
	if (prefetchTimeout) prefetchTimeout.stop()
	selectedVersion.value = version
	// Emit event so parent can fetch full version data with changelog
	emit('versionSelect', version)
}

function handleUpdate(event: MouseEvent) {
	if (props.actionLoading || props.actionDisabled) return
	if (selectedVersion.value) {
		if (incompatibilityWarningMode.value) {
			emitUpdate(selectedVersion.value, event, { hide: false })
			return
		}

		const changesGameVersion = versionChangesGameVersion(
			selectedVersion.value,
			props.currentGameVersion,
		)
		const shouldShowParentWarning =
			isModpack.value && !event.shiftKey && (changesGameVersion || isDowngrade.value)
		if (
			isModpack.value &&
			!event.shiftKey &&
			!isVersionCompatible(selectedVersion.value) &&
			!changesGameVersion
		) {
			pendingIncompatibleUpdate.value = {
				version: selectedVersion.value,
				event,
			}
			incompatibleUpdateModal.value?.show()
			return
		}

		emitUpdate(selectedVersion.value, event, {
			hide: !shouldShowParentWarning,
		})
	}
}

function confirmIncompatibleUpdate() {
	const pendingUpdate = pendingIncompatibleUpdate.value
	pendingIncompatibleUpdate.value = null

	if (pendingUpdate) {
		const current = currentVersion.value
		const isPendingDowngrade = current
			? new Date(pendingUpdate.version.date_published) < new Date(current.date_published)
			: false
		const changesGameVersion = versionChangesGameVersion(
			pendingUpdate.version,
			props.currentGameVersion,
		)
		const shouldShowParentWarning =
			isModpack.value && !pendingUpdate.event.shiftKey && (changesGameVersion || isPendingDowngrade)

		emitUpdate(pendingUpdate.version, pendingUpdate.event, {
			hide: !shouldShowParentWarning,
		})
	}
}

function emitUpdate(
	version: Labrinth.Versions.v2.Version,
	event: MouseEvent,
	options: { hide?: boolean } = {},
) {
	emit('update', version, event)
	if (options.hide ?? true) {
		hide()
	}
}

function handleCancel() {
	emit('cancel')
	hide()
}

function handleModalHide() {
	if (suppressCancelOnHide.value) {
		suppressCancelOnHide.value = false
		return
	}

	emit('cancel')
}

function show(initialVersionId?: string, options?: { switchMode?: boolean }) {
	searchQuery.value = ''
	hideIncompatibleState.value = incompatibilityWarningMode.value ? false : !isModpack.value
	pendingIncompatibleUpdate.value = null
	pinnedInitialVersionId.value = initialVersionId
	switchMode.value = options?.switchMode ?? false

	debug('show() called', {
		initialVersionId,
		currentVersionId: props.currentVersionId,
		currentGameVersion: props.currentGameVersion,
		currentLoader: props.currentLoader,
		projectType: props.projectType,
		versionsAvailable: props.versions.length,
	})

	if (props.versions.length > 0) {
		const currentInList = props.versions.find((v) => v.id === props.currentVersionId)
		debug('show(): currentVersionId lookup', {
			currentVersionId: props.currentVersionId,
			foundInList: !!currentInList,
			allVersionIds: props.versions.map((v) => v.id),
		})

		if (initialVersionId) {
			selectedVersion.value =
				props.versions.find((v) => v.id === initialVersionId) ?? props.versions[0]
		} else {
			selectedVersion.value = props.versions[0]
		}
		pendingInitialVersionId.value = undefined
		if (selectedVersion.value) {
			emit('versionSelect', selectedVersion.value)
		}
	} else {
		selectedVersion.value = null
		pendingInitialVersionId.value = initialVersionId
		debug('show(): no versions yet, deferring selection', {
			pendingInitialVersionId: initialVersionId,
		})
	}

	modal.value?.show()
}

function hide() {
	suppressCancelOnHide.value = true
	modal.value?.hide()
}

defineExpose({ show, hide })
</script>
