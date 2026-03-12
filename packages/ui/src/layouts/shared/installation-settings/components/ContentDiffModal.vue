<template>
	<NewModal
		ref="modal"
		:header="header"
		:closable="true"
		no-padding
	>
		<div class="max-w-[500px]">
			<div class="flex flex-col gap-4 p-4">
				<Admonition
					:type="hasUnknownContent ? 'warning' : 'info'"
					:header="admonitionHeader"
				>
					{{ description }}
				</Admonition>

				<Admonition
					v-if="hasUnknownContent"
					type="warning"
					:header="formatMessage(messages.unknownContentHeader)"
				>
					{{ formatMessage(messages.unknownContentBody) }}
				</Admonition>

				<div v-if="diffs.length" class="flex gap-2">
					<div v-if="removedCount" class="flex gap-1 items-center">
						<MinusIcon />
						{{ formatMessage(messages.removedCount, { count: removedCount }) }}
					</div>
					<div v-if="addedCount" class="flex gap-1 items-center">
						<PlusIcon />
						{{ formatMessage(messages.addedCount, { count: addedCount }) }}
					</div>
					<div v-if="updatedCount" class="flex gap-1 items-center">
						<RefreshCwIcon />
						{{ formatMessage(messages.updatedCount, { count: updatedCount }) }}
					</div>
				</div>
			</div>

			<div
				v-if="diffs.length"
				class="flex flex-col bg-surface-2 p-4 max-h-[272px] overflow-y-auto border-t border-b border-r-0 border-l-0 border-solid border-surface-5"
			>
				<div
					v-for="(diff, index) in sortedDiffs"
					:key="diff.projectName || diff.fileName || index"
					class="grid items-center min-h-10 h-10 gap-2"
					:class="
						diff.projectName
							? 'grid-cols-[auto_auto_1fr]'
							: 'grid-cols-[auto_auto_1fr]'
					"
				>
					<div class="flex flex-col justify-between items-center">
						<div class="w-[1px] h-2"></div>
						<PlusIcon v-if="diff.type === 'added'" />
						<MinusIcon v-else-if="diff.type === 'removed'" />
						<RefreshCwIcon v-else />
						<div
							:class="
								index === sortedDiffs.length - 1
									? 'bg-transparent'
									: 'bg-surface-5'
							"
							class="w-[1px] h-2 relative top-1"
						></div>
					</div>

					<span class="text-sm shrink-0 whitespace-nowrap">{{
						diff.type === 'removed' && props.removedLabel
							? props.removedLabel
							: formatMessage(diffTypeMessages[diff.type])
					}}</span>
					<span
						v-if="diff.projectName"
						class="text-sm text-contrast font-medium whitespace-nowrap overflow-hidden text-ellipsis"
					>
						{{ diff.projectName }}
					</span>
					<span
						v-else-if="diff.fileName"
						class="text-sm text-contrast font-medium whitespace-nowrap overflow-hidden text-ellipsis"
					>
						{{ decodeURIComponent(diff.fileName) }}
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
					@update:buttons-disabled="buttonsDisabled = $event"
				/>
			</div>
		</div>

		<template #actions>
			<div class="flex justify-between gap-2">
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
	MinusIcon,
	PlusIcon,
	RefreshCwIcon,
	ReportIcon,
	XIcon,
} from '@modrinth/assets'
import { computed, ref, type Component } from 'vue'

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

const sortedDiffs = computed(() =>
	[...props.diffs].sort((a, b) => {
		const typeOrder = { added: 0, updated: 1, removed: 2 }
		return typeOrder[a.type] - typeOrder[b.type]
	}),
)

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
	unknownContentHeader: {
		id: 'content.diff-modal.unknown-content-header',
		defaultMessage: 'Unknown content',
	},
	unknownContentBody: {
		id: 'content.diff-modal.unknown-content-body',
		defaultMessage:
			'Some content on your server could not be analyzed and may be affected by this change.',
	},
})

const diffTypeMessages = defineMessages({
	added: {
		id: 'content.diff-modal.diff-type.added',
		defaultMessage: 'New dependency',
	},
	removed: {
		id: 'content.diff-modal.diff-type.removed',
		defaultMessage: 'Removed',
	},
	updated: {
		id: 'content.diff-modal.diff-type.updated',
		defaultMessage: 'Updated',
	},
})

defineExpose({ show, hide })
</script>
