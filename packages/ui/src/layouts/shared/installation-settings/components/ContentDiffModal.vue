<template>
	<NewModal
		ref="modal"
		:header="header"
		:closable="true"
		:disable-close="disableClose"
		:on-hide="handleHide"
		max-width="544px"
		width="544px"
		no-padding
		scrollable
	>
		<div class="flex flex-col gap-4" :class="hasExternalDiffs ? 'px-6 py-4' : 'p-4'">
			<template v-if="hasExternalDiffs">
				<p v-if="description" class="m-0 text-primary">{{ description }}</p>
				<Admonition type="warning" :header="formatMessage(messages.unknownFilesWarning)">
					{{ externalWarningDescription || formatMessage(messages.unknownFilesDescription) }}
				</Admonition>
			</template>
			<Admonition v-else :type="hasUnknownContent ? 'warning' : 'info'" :header="admonitionHeader">
				<div class="flex flex-col gap-2">
					<span>{{ description }}</span>
					<span v-if="hasUnknownContent">{{ formatMessage(messages.unknownContentBody) }}</span>
				</div>
			</Admonition>

			<div v-if="diffs.length" class="flex flex-col gap-1">
				<span v-if="versionDate" class="font-semibold text-contrast">{{ versionDate }}</span>
				<div class="flex flex-wrap items-center gap-2 text-primary">
					<div v-if="updatedCount" class="flex items-center gap-1">
						<RefreshCwIcon class="size-4" />
						{{ formatMessage(messages.updatedCount, { count: updatedCount }) }}
					</div>
					<div v-if="addedCount" class="flex items-center gap-1">
						<PlusIcon class="size-4" />
						{{ formatMessage(messages.addedCount, { count: addedCount }) }}
					</div>
					<div v-if="removedCount" class="flex items-center gap-1">
						<MinusIcon class="size-4" />
						{{ formatMessage(messages.removedCount, { count: removedCount }) }}
					</div>
					<div v-if="removedDisabledCount" class="flex items-center gap-1">
						<MinusIcon class="size-4" />
						{{ formatMessage(messages.removedDisabledCount, { count: removedDisabledCount }) }}
					</div>
				</div>
			</div>
		</div>

		<div
			class="flex max-h-[272px] flex-col overflow-y-auto border-0 border-y border-solid border-surface-5 bg-surface-2 px-3 py-2"
		>
			<div v-if="!diffs.length" class="flex h-10 min-h-10 items-center gap-2 px-2">
				<div class="flex w-4 shrink-0 items-center justify-center">
					<MinusIcon class="size-4" />
				</div>
				<span class="text-sm font-medium text-contrast">
					{{ formatMessage(messages.noContentChanges) }}
				</span>
			</div>
			<div
				v-for="(diff, index) in sortedDiffs"
				:key="diff.projectName || diff.fileName || (isConfigurationDiff(diff) ? diff.type : index)"
				class="flex h-10 min-h-10 items-center gap-2"
				:class="showExternalWarning(diff) ? '-mx-3 px-5' : 'px-2'"
				:style="
					showExternalWarning(diff)
						? {
								backgroundColor: 'color-mix(in srgb, var(--color-orange) 10%, transparent)',
							}
						: undefined
				"
			>
				<div class="relative flex w-4 shrink-0 self-stretch items-center justify-center">
					<div
						v-if="index > 0"
						class="absolute left-1/2 top-0 h-3 w-px -translate-x-1/2 bg-surface-5"
					/>
					<PlusIcon
						v-if="diff.type === 'added' || diff.type === 'modpack_linked'"
						class="relative z-[1] size-4"
					/>
					<MinusIcon
						v-else-if="diff.type === 'removed' || diff.type === 'modpack_unlinked'"
						class="relative z-[1] size-4 text-red"
					/>
					<RefreshCwIcon v-else class="relative z-[1] size-4" />
					<div
						v-if="index < sortedDiffs.length - 1"
						class="absolute bottom-0 left-1/2 top-7 w-px -translate-x-1/2 bg-surface-5"
					/>
				</div>

				<div class="flex min-w-0 flex-1 items-center gap-1 text-sm">
					<span class="shrink-0 whitespace-nowrap text-primary">{{ getDiffTypeLabel(diff) }}</span>
					<template v-if="showExternalWarning(diff)">
						<CircleAlertIcon class="size-4 shrink-0 text-orange" />
						<span class="truncate font-medium text-orange">
							{{ formatMessage(messages.unknownProject) }}
						</span>
					</template>
					<span
						v-else-if="!isConfigurationDiff(diff) || diff.type === 'modpack_updated'"
						class="truncate font-medium text-contrast"
					>
						{{ diff.projectName || (diff.fileName ? decodeURIComponent(diff.fileName) : '') }}
					</span>
				</div>
				<span
					v-if="getVersionLabel(diff)"
					class="ml-2 max-w-[60%] min-w-0 shrink truncate text-right text-xs"
					:class="showExternalWarning(diff) ? 'text-orange' : 'text-primary'"
					:title="getVersionLabel(diff)"
				>
					{{ getVersionLabel(diff) }}
				</span>
			</div>
		</div>

		<div
			v-if="$slots['additional-content']"
			class="px-4 pt-4"
		>
			<slot name="additional-content" />
		</div>

		<div
			v-if="showBackupCreator"
			class="p-4 border-t border-solid border-surface-5 border-b-0 border-l-0 border-r-0"
		>
			<InlineBackupCreator
				ref="backupCreator"
				backup-name="Before version change"
				hide-shift-click-hint
				@update:buttons-disabled="buttonsDisabled = $event"
			/>
		</div>

		<template #actions>
			<div v-if="hasExternalDiffs" class="flex flex-col gap-6 p-2">
				<p class="m-0 text-primary">{{ formatMessage(messages.reviewedFiles) }}</p>
				<div class="flex justify-between gap-2">
					<div>
						<ButtonStyled v-if="showReportButton" color="red" type="transparent">
							<button @click="emit('report', $event)">
								<ReportIcon />
								{{ formatMessage(commonMessages.reportButton) }}
							</button>
						</ButtonStyled>
					</div>
					<div class="flex gap-2">
						<ButtonStyled type="transparent" color="orange">
							<button :disabled="buttonsDisabled || confirmDisabled" @click="handleConfirm">
								{{ formatMessage(messages.installAnyway) }}
							</button>
						</ButtonStyled>
						<ButtonStyled color="brand">
							<button @click="handleCancel">
								<BanIcon />
								{{ formatMessage(messages.dontInstall) }}
							</button>
						</ButtonStyled>
					</div>
				</div>
			</div>
			<div
				v-else
				class="flex justify-between gap-2"
				:class="{ 'pt-4': !$slots['additional-content'] }"
			>
				<div>
					<ButtonStyled v-if="showReportButton" color="red" type="transparent">
						<button @click="emit('report', $event)">
							<ReportIcon />
							{{ formatMessage(commonMessages.reportButton) }}
						</button>
					</ButtonStyled>
				</div>
				<div class="flex gap-2">
					<ButtonStyled type="outlined">
						<button class="!border" @click="handleCancel">
							<XIcon />
							{{ formatMessage(commonMessages.cancelButton) }}
						</button>
					</ButtonStyled>
					<ButtonStyled color="brand">
						<button :disabled="buttonsDisabled || confirmDisabled" @click="handleConfirm">
							<component :is="confirmIcon" v-if="confirmIcon" />
							{{ confirmLabel || formatMessage(commonMessages.confirmButton) }}
						</button>
					</ButtonStyled>
				</div>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import {
	BanIcon,
	CircleAlertIcon,
	MinusIcon,
	PlusIcon,
	RefreshCwIcon,
	ReportIcon,
	XIcon,
} from '@modrinth/assets'
import { type Component, computed, ref } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import InlineBackupCreator from '../../content-tab/components/modals/InlineBackupCreator.vue'
import type { ContentDiffItem } from '../types'

const props = defineProps<{
	header: string
	description?: string
	admonitionHeader?: string
	diffs: ContentDiffItem[]
	hasUnknownContent?: boolean
	confirmLabel?: string
	confirmIcon?: Component
	showReportButton?: boolean
	showBackupCreator?: boolean
	addedLabel?: string
	removedLabel?: string
	confirmDisabled?: boolean
	disableClose?: boolean
	showExternalWarnings?: boolean
	externalWarningDescription?: string
	versionDate?: string
}>()

const emit = defineEmits<{
	confirm: []
	cancel: []
	report: [event?: MouseEvent]
}>()

const { formatMessage } = useVIntl()

const modal = ref<InstanceType<typeof NewModal>>()
const backupCreator = ref<InstanceType<typeof InlineBackupCreator>>()
const buttonsDisabled = ref(false)
const closingFromAction = ref(false)

const removedCount = computed(
	() => props.diffs.filter((diff) => diff.type === 'removed' && !diff.disabled).length,
)
const removedDisabledCount = computed(
	() => props.diffs.filter((diff) => diff.type === 'removed' && diff.disabled).length,
)
const addedCount = computed(() => props.diffs.filter((diff) => diff.type === 'added').length)
const updatedCount = computed(
	() =>
		props.diffs.filter((diff) => diff.type === 'updated' || diff.type === 'config_files_updated')
			.length,
)
const hasExternalDiffs = computed(() => props.diffs.some(showExternalWarning))

type DependencyDiffType = Extract<ContentDiffItem['type'], 'added' | 'removed' | 'updated'>
type ConfigurationDiffType = Exclude<ContentDiffItem['type'], DependencyDiffType>

const configurationDiffTypes = new Set<ConfigurationDiffType>([
	'modpack_linked',
	'modpack_updated',
	'modpack_unlinked',
	'game_version_updated',
	'loader_updated',
	'config_files_updated',
])

function isDependencyDiff(
	diff: ContentDiffItem,
): diff is ContentDiffItem & { type: DependencyDiffType } {
	return diff.type === 'added' || diff.type === 'removed' || diff.type === 'updated'
}

function isConfigurationDiff(
	diff: ContentDiffItem,
): diff is ContentDiffItem & { type: ConfigurationDiffType } {
	return configurationDiffTypes.has(diff.type as ConfigurationDiffType)
}

function showExternalWarning(diff: ContentDiffItem) {
	return Boolean(
		props.showExternalWarnings &&
		diff.external &&
		isDependencyDiff(diff) &&
		diff.type !== 'removed',
	)
}

const sortedDiffs = computed(() =>
	[...props.diffs].sort((a, b) => {
		const aExternal = showExternalWarning(a)
		const bExternal = showExternalWarning(b)
		if (aExternal !== bExternal) return aExternal ? -1 : 1

		const typeOrder: Record<ContentDiffItem['type'], number> = {
			modpack_linked: 0,
			modpack_updated: 0,
			modpack_unlinked: 0,
			game_version_updated: 1,
			loader_updated: 2,
			config_files_updated: 3,
			added: 4,
			updated: 5,
			removed: 6,
		}
		return typeOrder[a.type] - typeOrder[b.type]
	}),
)

function getDiffTypeLabel(diff: ContentDiffItem) {
	if (showExternalWarning(diff) && isDependencyDiff(diff)) {
		return formatMessage(externalDiffTypeMessages[diff.type])
	}
	if (diff.type === 'modpack_updated') return formatMessage(diffTypeMessages.updated)
	if (isConfigurationDiff(diff)) return formatMessage(configurationDiffMessages[diff.type])
	if (diff.type === 'removed' && diff.disabled) {
		return formatMessage(diffTypeMessages.removedDisabled)
	}
	if (diff.type === 'added' && props.addedLabel) return props.addedLabel
	if (diff.type === 'removed' && props.removedLabel) return props.removedLabel
	return formatMessage(diffTypeMessages[diff.type])
}

function getVersionLabel(diff: ContentDiffItem) {
	if (showExternalWarning(diff) && diff.fileName) return decodeURIComponent(diff.fileName)
	if (diff.type === 'config_files_updated' && diff.fileCount !== undefined) {
		return formatMessage(messages.fileCount, { count: diff.fileCount })
	}
	if (diff.type === 'modpack_updated') return diff.newVersionName
	if (isConfigurationDiff(diff)) {
		if (diff.currentVersionName && diff.newVersionName) {
			return `${diff.currentVersionName} → ${diff.newVersionName}`
		}
		return diff.newVersionName ?? diff.currentVersionName
	}
	return diff.type === 'removed' ? diff.currentVersionName : diff.newVersionName
}

function show(e?: MouseEvent) {
	modal.value?.show(e)
}

function hide() {
	modal.value?.hide()
}

function handleConfirm() {
	closingFromAction.value = true
	hide()
	emit('confirm')
	closingFromAction.value = false
}

function handleCancel() {
	closingFromAction.value = true
	hide()
	emit('cancel')
	closingFromAction.value = false
}

function handleHide() {
	if (closingFromAction.value) {
		closingFromAction.value = false
		return
	}
	emit('cancel')
}

const messages = defineMessages({
	removedCount: {
		id: 'content.diff-modal.removed-count',
		defaultMessage: '{count} removed',
	},
	removedDisabledCount: {
		id: 'content.diff-modal.removed-disabled-count',
		defaultMessage: '{count} removed (disabled)',
	},
	addedCount: {
		id: 'content.diff-modal.added-count',
		defaultMessage: '{count} added',
	},
	updatedCount: {
		id: 'content.diff-modal.updated-count',
		defaultMessage: '{count} updated',
	},
	noContentChanges: {
		id: 'content.diff-modal.no-content-changes',
		defaultMessage: 'No content changes',
	},
	fileCount: {
		id: 'content.diff-modal.file-count',
		defaultMessage: '{count, plural, one {# file} other {# files}}',
	},
	unknownContentBody: {
		id: 'content.diff-modal.unknown-content-body',
		defaultMessage:
			'Some content on your server could not be analyzed and may be affected by this change.',
	},
	unknownFilesWarning: {
		id: 'content.diff-modal.unknown-files-warning',
		defaultMessage: 'Unknown files warning',
	},
	unknownFilesDescription: {
		id: 'content.diff-modal.unknown-files-description',
		defaultMessage:
			'This update contains files that aren’t published on Modrinth. We strongly recommend only installing files from sources you trust.',
	},
	unknownProject: {
		id: 'content.diff-modal.unknown-project',
		defaultMessage: 'Unknown',
	},
	reviewedFiles: {
		id: 'content.diff-modal.reviewed-files',
		defaultMessage:
			'A file is only reviewed if it’s published to Modrinth, regardless of its file format (including .mrpack).',
	},
	installAnyway: {
		id: 'content.diff-modal.install-anyway',
		defaultMessage: 'Install anyway',
	},
	dontInstall: {
		id: 'content.diff-modal.dont-install',
		defaultMessage: "Don't install",
	},
})

const configurationDiffMessages = defineMessages({
	modpack_linked: {
		id: 'content.diff-modal.modpack-linked',
		defaultMessage: 'Linked modpack',
	},
	modpack_updated: {
		id: 'content.diff-modal.modpack-updated',
		defaultMessage: 'Updated modpack',
	},
	modpack_unlinked: {
		id: 'content.diff-modal.modpack-unlinked',
		defaultMessage: 'Unlinked modpack',
	},
	game_version_updated: {
		id: 'content.diff-modal.game-version-updated',
		defaultMessage: 'Game version',
	},
	loader_updated: {
		id: 'content.diff-modal.loader-updated',
		defaultMessage: 'Loader',
	},
	config_files_updated: {
		id: 'content.diff-modal.config-files-updated',
		defaultMessage: 'Changed config files',
	},
})

const diffTypeMessages = defineMessages({
	added: {
		id: 'content.diff-modal.diff-type.added',
		defaultMessage: 'Added (dependency)',
	},
	removed: {
		id: 'content.diff-modal.diff-type.removed',
		defaultMessage: 'Disabled',
	},
	removedDisabled: {
		id: 'content.diff-modal.diff-type.removed-disabled',
		defaultMessage: 'Removed (disabled)',
	},
	updated: {
		id: 'content.diff-modal.diff-type.updated',
		defaultMessage: 'Updated',
	},
})

const externalDiffTypeMessages = defineMessages({
	added: {
		id: 'content.diff-modal.external-diff-type.added',
		defaultMessage: 'Added',
	},
	removed: {
		id: 'content.diff-modal.external-diff-type.removed',
		defaultMessage: 'Removed',
	},
	updated: {
		id: 'content.diff-modal.external-diff-type.updated',
		defaultMessage: 'Updated',
	},
})

defineExpose({ show, hide })
</script>
