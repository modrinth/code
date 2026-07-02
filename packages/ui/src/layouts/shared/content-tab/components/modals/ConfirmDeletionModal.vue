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
	sharedAdmonitionHeader: {
		id: 'content.confirm-deletion.shared-instance.admonition-header',
		defaultMessage: 'This is part of the shared instance',
	},
	sharedSingleBody: {
		id: 'content.confirm-deletion.shared-instance.single-body',
		defaultMessage:
			'Deleting it only changes your local copy. Future shared instance updates may restore or change it again.',
	},
	sharedBulkBody: {
		id: 'content.confirm-deletion.shared-instance.bulk-body',
		defaultMessage:
			'Some selected projects are part of the shared instance. Deleting them only changes your local copy, and future shared instance updates may restore or change them again.',
	},
	sharedDeleteButton: {
		id: 'content.confirm-deletion.shared-instance.delete-button',
		defaultMessage: 'Delete anyway',
	},
	sharedDeleteManyButton: {
		id: 'content.confirm-deletion.shared-instance.delete-many-button',
		defaultMessage: 'Delete {count, number} {itemType} anyway',
	},
})

const props = withDefaults(
	defineProps<{
		count: number
		itemType: string
		mode?: 'default' | 'shared-instance'
		variant?: 'instance' | 'server'
		backupTip?: string
		actionDisabled?: boolean
		actionDisabledTooltip?: string
	}>(),
	{
		mode: 'default',
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
const visibleMode = ref(props.mode)

const formattedItemType = computed(() =>
	formatContentTypeSentence(formatMessage, visibleItemType.value, visibleCount.value),
)

const admonitionHeader = computed(() =>
	visibleMode.value === 'shared-instance'
		? formatMessage(messages.sharedAdmonitionHeader)
		: formatMessage(messages.admonitionHeader),
)

const admonitionBody = computed(() => {
	if (visibleMode.value !== 'shared-instance') {
		return formatMessage(messages.admonitionBody)
	}

	return visibleCount.value === 1
		? formatMessage(messages.sharedSingleBody)
		: formatMessage(messages.sharedBulkBody)
})

const deleteButtonLabel = computed(() => {
	if (visibleMode.value !== 'shared-instance') {
		return formatMessage(messages.deleteButton, {
			count: visibleCount.value,
			itemType: formattedItemType.value,
		})
	}

	if (visibleCount.value === 1) {
		return formatMessage(messages.sharedDeleteButton)
	}

	return formatMessage(messages.sharedDeleteManyButton, {
		count: visibleCount.value,
		itemType: formattedItemType.value,
	})
})

async function show() {
	await nextTick()
	visibleCount.value = props.count
	visibleItemType.value = props.itemType
	visibleMode.value = props.mode
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
