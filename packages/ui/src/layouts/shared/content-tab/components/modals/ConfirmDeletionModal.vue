<template>
	<NewModal
		ref="modal"
		:header="
			formatMessage(messages.header, {
				itemType: formatContentTypeSentence(formatMessage, visibleItemType, visibleCount),
			})
		"
		fade="warning"
		max-width="500px"
		:on-hide="() => backupCreator?.cancelBackup()"
	>
		<div class="flex flex-col gap-6">
			<Admonition type="warning" :header="admonitionHeader">
				{{ admonitionBody }}
			</Admonition>
			<InlineBackupCreator
				ref="backupCreator"
				:backup-name="props.backupTip ? `Before deletion (${props.backupTip})` : 'Before deletion'"
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
						<TrashIcon />
						{{ deleteButtonLabel }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { TrashIcon, XIcon } from '@modrinth/assets'
import { computed, nextTick, ref } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages, formatContentTypeSentence } from '#ui/utils/common-messages'

import type { ContentActionWarning } from '../../types'
import InlineBackupCreator from './InlineBackupCreator.vue'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	header: {
		id: 'content.confirm-deletion.header',
		defaultMessage: 'Delete {itemType}',
	},
	admonitionHeader: {
		id: 'content.confirm-deletion.admonition-header',
		defaultMessage: 'Deletion warning',
	},
	admonitionBody: {
		id: 'content.confirm-deletion.admonition-body',
		defaultMessage:
			'Deleting a mod can permanently affect your world and may cause missing content or unexpected issues when it loads again.',
	},
	deleteButton: {
		id: 'content.confirm-deletion.delete-button',
		defaultMessage: 'Delete {count, number} {itemType}',
	},
})

const props = withDefaults(
	defineProps<{
		count: number
		itemType: string
		warning?: ContentActionWarning | null
		variant?: 'instance' | 'server'
		backupTip?: string
		actionDisabled?: boolean
		actionDisabledTooltip?: string
	}>(),
	{
		warning: null,
		variant: 'instance',
		backupTip: undefined,
		actionDisabled: false,
		actionDisabledTooltip: undefined,
	},
)

const emit = defineEmits<{
	(e: 'delete'): void
}>()

const modal = ref<InstanceType<typeof NewModal>>()
const backupCreator = ref<InstanceType<typeof InlineBackupCreator>>()
const buttonsDisabled = ref(false)
const visibleCount = ref(props.count)
const visibleItemType = ref(props.itemType)
const visibleWarning = ref(props.warning)

const formattedItemType = computed(() =>
	formatContentTypeSentence(formatMessage, visibleItemType.value, visibleCount.value),
)

const admonitionHeader = computed(() =>
	visibleWarning.value?.admonitionHeader ?? formatMessage(messages.admonitionHeader),
)

const admonitionBody = computed(() => {
	return visibleWarning.value?.admonitionBody ?? formatMessage(messages.admonitionBody)
})

const deleteButtonLabel = computed(() => {
	return visibleWarning.value?.actionLabel ?? formatMessage(messages.deleteButton, {
		count: visibleCount.value,
		itemType: formattedItemType.value,
	})
})

async function show() {
	await nextTick()
	visibleCount.value = props.count
	visibleItemType.value = props.itemType
	visibleWarning.value = props.warning
	modal.value?.show()
}

function confirm() {
	if (props.actionDisabled) return
	modal.value?.hide()
	emit('delete')
}

defineExpose({
	show,
})
</script>
