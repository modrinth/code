<template>
	<NewModal ref="modal" :max-width="'90vw'" :width="'90vw'" no-padding>
		<template #title>
			<Avatar v-if="projectIconUrl" :src="projectIconUrl" size="3rem" :tint-by="projectName" />
			<span class="text-lg font-extrabold text-contrast">{{
				header ?? formatMessage(messages.updateVersionHeader)
			}}</span>
		</template>
		<div class="flex h-[550px] border-solid border-transparent border-[1px] border-b-surface-4">
			<div class="w-[300px] flex flex-col relative">
				<div class="p-4 pb-2">
					<div class="iconified-input w-full border-solid border-[1px] border-surface-4 rounded-xl">
						<SearchIcon class="transition-colors" />
						<input
							v-model="searchQuery"
							type="text"
							:placeholder="formatMessage(messages.searchVersionPlaceholder)"
							class="!bg-transparent rounded-xl transition-colors"
						/>
					</div>
				</div>

				<div class="flex-1 overflow-y-auto px-4 pb-16">
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
							@click="selectedVersion = version"
						>
							<div class="flex items-center justify-between w-full">
								<span
									v-tooltip="'v' + version.version_number"
									class="font-semibold text-contrast truncate"
								>
									v{{ version.version_number }}
								</span>
								<span
									class="px-2.5 py-0.5 rounded-full text-sm font-medium flex items-center flex-shrink-0 border border-solid"
									:class="getBadgeClasses(version)"
								>
									{{ getBadgeLabel(version) }}
								</span>
							</div>
						</button>
					</div>
					<div v-if="filteredVersions.length === 0" class="p-4 text-center text-secondary text-sm">
						{{ formatMessage(messages.noVersionsFound) }}
					</div>
				</div>

				<div class="absolute bottom-0 left-0 right-0 pointer-events-none">
					<div class="h-14 bg-gradient-to-t from-bg-raised to-transparent" />
					<div class="bg-bg-raised pb-5 flex justify-center pointer-events-auto">
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
								<div class="flex items-center gap-1.5">
									<span class="font-semibold text-xl text-contrast">
										v{{ selectedVersion.version_number }}
									</span>
									<span
										class="px-2.5 py-0.5 rounded-full text-sm font-medium flex items-center flex-shrink-0 border border-solid"
										:class="getBadgeClasses(selectedVersion)"
									>
										{{ getBadgeLabel(selectedVersion) }}
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
							v-if="selectedVersion.changelog"
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
			class="bg-highlight-orange h-9 text-orange p-2 border-solid border-x-0 border-[1px] flex flex-row gap-2"
		>
			<TriangleAlertIcon class="size-4" />
			<span>{{
				formatMessage(isApp ? messages.updateWarningApp : messages.updateWarningWeb)
			}}</span>
		</div>

		<div class="w-full flex flex-row gap-2 justify-end p-4">
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
	</NewModal>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	DownloadIcon,
	EyeIcon,
	EyeOffIcon,
	FileTextIcon,
	SearchIcon,
	TriangleAlertIcon,
	XIcon,
} from '@modrinth/assets'
import { capitalizeString, renderHighlightedString } from '@modrinth/utils'
import { computed, ref } from 'vue'

import { useFormatDateTime } from '../../../composables'
import { defineMessages, useVIntl } from '../../../composables/i18n'
import { commonMessages } from '../../../utils/common-messages'
import Avatar from '../../base/Avatar.vue'
import ButtonStyled from '../../base/ButtonStyled.vue'
import NewModal from '../../modal/NewModal.vue'

const { formatMessage } = useVIntl()
const formatDate = useFormatDateTime({ dateStyle: 'long' })

const messages = defineMessages({
	updateVersionHeader: {
		id: 'instances.updater-modal.header',
		defaultMessage: 'Update version',
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
		id: 'instances.updater-modal.warning.app',
		defaultMessage:
			"We can't guarantee updates are safe for your instance. Review the changelog for all intermediate versions and consider a backup.",
	},
	updateWarningWeb: {
		id: 'instances.updater-modal.warning.web',
		defaultMessage:
			"We can't guarantee updates are safe for your worlds. Review the changelog for all intermediate versions and consider a backup.",
	},
	downgradeToVersion: {
		id: 'instances.updater-modal.downgrade-to',
		defaultMessage: 'Downgrade to v{version}',
	},
	updateToVersion: {
		id: 'instances.updater-modal.update-to',
		defaultMessage: 'Update to v{version}',
	},
	currentBadge: {
		id: 'instances.updater-modal.badge.current',
		defaultMessage: 'Current',
	},
	incompatibleBadge: {
		id: 'instances.updater-modal.badge.incompatible',
		defaultMessage: 'Incompatible',
	},
})

const props = withDefaults(
	defineProps<{
		versions: Labrinth.Versions.v2.Version[]
		currentGameVersion: string
		currentLoader: string
		currentVersionId: string
		isApp: boolean
		projectIconUrl?: string
		projectName?: string
		header?: string
	}>(),
	{
		projectIconUrl: undefined,
		projectName: undefined,
		header: undefined,
	},
)

const emit = defineEmits<{
	update: [version: Labrinth.Versions.v2.Version]
	cancel: []
}>()

const modal = ref<InstanceType<typeof NewModal>>()
const searchQuery = ref('')
const hideIncompatibleState = ref(true)
const selectedVersion = ref<Labrinth.Versions.v2.Version | null>(null)

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

	// Filter by search query
	if (searchQuery.value) {
		const query = searchQuery.value.toLowerCase()
		versions = versions.filter(
			(v) => v.name.toLowerCase().includes(query) || v.version_number.toLowerCase().includes(query),
		)
	}

	// Filter by compatibility
	if (hideIncompatibleState.value) {
		versions = versions.filter(isVersionCompatible)
	}

	return versions
})

function getBadgeLabel(version: Labrinth.Versions.v2.Version): string {
	if (version.id === props.currentVersionId) return formatMessage(messages.currentBadge)
	if (!isVersionCompatible(version)) return formatMessage(messages.incompatibleBadge)
	return capitalizeString(version.version_type)
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

function formatLongDate(dateString: string): string {
	return formatDate(new Date(dateString))
}

function formatLoaderGameVersion(version: Labrinth.Versions.v2.Version): string {
	const loader = capitalizeString(version.loaders[0] || '')
	const gameVersion = version.game_versions[0] || ''
	return `${loader} ${gameVersion}`
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

	// Pre-select a version
	if (initialVersionId) {
		selectedVersion.value = props.versions.find((v) => v.id === initialVersionId) ?? null
	} else if (props.versions.length > 0) {
		// Default to first version if none specified
		selectedVersion.value = props.versions[0]
	} else {
		selectedVersion.value = null
	}

	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

defineExpose({ show, hide })
</script>
