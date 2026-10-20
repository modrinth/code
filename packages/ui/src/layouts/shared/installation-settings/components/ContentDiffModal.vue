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

		<ContentDiffList
			:diffs="diffs"
			:added-label="addedLabel"
			:removed-label="removedLabel"
			:show-external-warnings="showExternalWarnings"
		/>

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
						<Button
							v-if="showReportButton"
							type="quiet"
							color="red"
							@click="emit('report', $event)"
						>
							<ReportIcon />
							{{ formatMessage(commonMessages.reportButton) }}
						</Button>
					</div>
					<div class="flex gap-2">
						<Button
							type="quiet"
							color="orange"
							:disabled="buttonsDisabled || confirmDisabled"
							@click="handleConfirm"
						>
							{{ formatMessage(messages.installAnyway) }}
						</Button>
						<Button type="colored" color="brand" @click="handleCancel">
							<BanIcon />
							{{ formatMessage(messages.dontInstall) }}
						</Button>
					</div>
				</div>
			</div>
			<div v-else class="flex justify-between gap-2 pt-4">
				<div>
					<Button v-if="showReportButton" type="quiet" color="red" @click="emit('report', $event)">
						<ReportIcon />
						{{ formatMessage(commonMessages.reportButton) }}
					</Button>
				</div>
				<div class="flex gap-2">
					<Button type="outlined" class="!border" @click="handleCancel">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</Button>
					<Button
						type="colored"
						color="brand"
						:disabled="buttonsDisabled || confirmDisabled"
						@click="handleConfirm"
					>
						<component :is="confirmIcon" v-if="confirmIcon" />
						{{ confirmLabel || formatMessage(commonMessages.confirmButton) }}
					</Button>
				</div>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { BanIcon, MinusIcon, PlusIcon, RefreshCwIcon, ReportIcon, XIcon } from '@modrinth/assets'
import { type Component, computed, ref } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import { Button } from '#ui/components/base/buttons'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import InlineBackupCreator from '../../content-tab/components/modals/InlineBackupCreator.vue'
import type { ContentDiffItem } from '../types'
import { showsExternalWarning } from '../utils'
import ContentDiffList from './ContentDiffList.vue'

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
const hasExternalDiffs = computed(() =>
	props.diffs.some((diff) => showsExternalWarning(diff, props.showExternalWarnings)),
)

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
	reviewedFiles: {
		id: 'content.diff-modal.reviewed-files',
		defaultMessage: "Files that aren't published to Modrinth aren't reviewed.",
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

defineExpose({ show, hide })
</script>
