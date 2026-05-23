<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.header)"
		fade="warning"
		max-width="500px"
		:on-hide="() => backupCreator?.cancelBackup()"
	>
		<div class="flex flex-col gap-6">
			<Admonition type="warning" :header="formatMessage(messages.admonitionHeader)">
				{{ formatMessage(messages.admonitionBody, { count: visibleCount }) }}
			</Admonition>
			<InlineBackupCreator
				ref="backupCreator"
				:backup-name="
					visibleBackupTip ? `Before bulk update (${visibleBackupTip})` : 'Before bulk update'
				"
				:target-type="props.targetType"
				:shift-click-hint-override="formatMessage(messages.shiftClickHint)"
				@update:buttons-disabled="buttonsDisabled = $event"
			/>
		</div>

		<template #actions>
			<div class="flex gap-2 justify-end">
				<ButtonStyled type="outlined">
					<button @click="modal?.hide()">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="orange">
					<button
						v-tooltip="props.actionDisabled ? props.actionDisabledTooltip : undefined"
						:disabled="buttonsDisabled || props.actionDisabled"
						@click="confirm"
					>
						<DownloadIcon />
						{{ formatMessage(messages.updateButton, { count: visibleCount }) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { DownloadIcon, XIcon } from '@modrinth/assets'
import { nextTick, ref } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import InlineBackupCreator from './InlineBackupCreator.vue'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	header: {
		id: 'content.confirm-bulk-update.header',
		defaultMessage: 'Update projects',
	},
	admonitionHeader: {
		id: 'content.confirm-bulk-update.admonition-header',
		defaultMessage: 'Update warning',
	},
	admonitionBody: {
		id: 'content.confirm-bulk-update.admonition-body',
		defaultMessage:
			"Are you sure you want to update {count, plural, one {# project} other {# projects}} to their latest compatible version? It's recommended to update content one-by-one.",
	},
	updateButton: {
		id: 'content.confirm-bulk-update.update-button',
		defaultMessage: 'Update {count, plural, one {# project} other {# projects}}',
	},
	shiftClickHint: {
		id: 'content.confirm-bulk-update.shift-click-hint',
		defaultMessage:
			'Hold Shift while clicking "Update all" to skip this confirmation in the future.',
	},
})

const props = defineProps<{
	count: number
	server?: boolean
	backupTip?: string
	actionDisabled?: boolean
	actionDisabledTooltip?: string
	targetType?: 'server' | 'instance'
}>()

const emit = defineEmits<{
	(e: 'update'): void
}>()

const modal = ref<InstanceType<typeof NewModal>>()
const backupCreator = ref<InstanceType<typeof InlineBackupCreator>>()
const buttonsDisabled = ref(false)
const visibleCount = ref(props.count)
const visibleBackupTip = ref(props.backupTip)

async function show() {
	await nextTick()
	visibleCount.value = props.count
	visibleBackupTip.value = props.backupTip
	modal.value?.show()
}

function confirm() {
	if (props.actionDisabled) return
	modal.value?.hide()
	emit('update')
}

defineExpose({
	show,
})
</script>
