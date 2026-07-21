<template>
	<NewModal
		ref="modal"
		:header="header"
		:closable="true"
		:disable-close="disableClose"
		max-width="544px"
		width="544px"
		no-padding
	>
		<div class="flex flex-col gap-4" :class="hasExternalDiffs ? 'px-6 py-4' : 'p-4'">
			<template v-if="hasExternalDiffs">
				<p v-if="description" class="m-0 text-primary">{{ description }}</p>
				<Admonition
					v-if="hasExternalDiffs"
					type="warning"
					:header="formatMessage(messages.unknownFilesWarning)"
				>
					{{ formatMessage(messages.unknownFilesDescription) }}
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
				</div>
			</div>
		</div>

		<div
			v-if="diffs.length"
			class="flex max-h-[272px] flex-col overflow-y-auto border-0 border-y border-solid border-surface-5 bg-surface-2 px-3 py-4"
		>
			<div
				v-for="(diff, index) in sortedDiffs"
				:key="diff.projectName || diff.fileName || index"
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
					<PlusIcon v-if="diff.type === 'added'" class="relative z-[1] size-4" />
					<MinusIcon v-else-if="diff.type === 'removed'" class="relative z-[1] size-4 text-red" />
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
					<span v-else class="truncate font-medium text-contrast">
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
				<div class="flex justify-end gap-2">
					<ButtonStyled type="transparent" color="orange">
						<button :disabled="buttonsDisabled" @click="handleConfirm">
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
			<div v-else class="flex justify-between gap-2 pt-4">
				<div>
					<ButtonStyled v-if="showReportButton" color="red" type="transparent">
						<button @click="emit('report')">
							<ReportIcon />
							{{ formatMessage(commonMessages.reportButton) }}
						</button>
					</ButtonStyled>
				</div>
				<div class="flex gap-2">
					<ButtonStyled>
						<button @click="handleCancel">
							<XIcon />
							{{ formatMessage(commonMessages.cancelButton) }}
						</button>
					</ButtonStyled>
					<ButtonStyled color="brand">
						<button :disabled="buttonsDisabled" @click="handleConfirm">
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
	removedLabel?: string
	disableClose?: boolean
	showExternalWarnings?: boolean
	versionDate?: string
}>()

const emit = defineEmits<{
	confirm: []
	cancel: []
	report: []
}>()

const { formatMessage } = useVIntl()

const modal = ref<InstanceType<typeof NewModal>>()
const backupCreator = ref<InstanceType<typeof InlineBackupCreator>>()
const buttonsDisabled = ref(false)

const removedCount = computed(() => props.diffs.filter((d) => d.type === 'removed').length)
const addedCount = computed(() => props.diffs.filter((d) => d.type === 'added').length)
const updatedCount = computed(() => props.diffs.filter((d) => d.type === 'updated').length)
const hasExternalDiffs = computed(() => props.diffs.some(showExternalWarning))

const sortedDiffs = computed(() =>
	[...props.diffs].sort((a, b) => {
		const aExternal = showExternalWarning(a)
		const bExternal = showExternalWarning(b)
		if (aExternal !== bExternal) return aExternal ? -1 : 1

		const typeOrder = { added: 0, updated: 1, removed: 2 }
		return typeOrder[a.type] - typeOrder[b.type]
	}),
)

function getDiffTypeLabel(diff: ContentDiffItem) {
	if (showExternalWarning(diff)) return formatMessage(externalDiffTypeMessages[diff.type])
	if (diff.type === 'removed' && props.removedLabel) return props.removedLabel
	return formatMessage(diffTypeMessages[diff.type])
}

function getVersionLabel(diff: ContentDiffItem) {
	if (showExternalWarning(diff) && diff.fileName) return decodeURIComponent(diff.fileName)
	return diff.type === 'removed' ? diff.currentVersionName : diff.newVersionName
}

function showExternalWarning(diff: ContentDiffItem) {
	return Boolean(props.showExternalWarnings && diff.external && diff.type !== 'removed')
}

function show(e?: MouseEvent) {
	modal.value?.show(e)
}

function hide() {
	modal.value?.hide()
}

function handleConfirm() {
	hide()
	emit('confirm')
}

function handleCancel() {
	hide()
	emit('cancel')
}

const messages = defineMessages({
	removedCount: {
		id: 'content.diff-modal.removed-count',
		defaultMessage: '{count} removed',
	},
	addedCount: {
		id: 'content.diff-modal.added-count',
		defaultMessage: '{count} added',
	},
	updatedCount: {
		id: 'content.diff-modal.updated-count',
		defaultMessage: '{count} updated',
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

const diffTypeMessages = defineMessages({
	added: {
		id: 'content.diff-modal.diff-type.added',
		defaultMessage: 'Added (dependency)',
	},
	removed: {
		id: 'content.diff-modal.diff-type.removed',
		defaultMessage: 'Disabled',
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
