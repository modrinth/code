<template>
	<NewModal
		ref="modal"
		:header="formatMessage(headerMessage)"
		fade="warning"
		max-width="500px"
		:on-hide="() => backupCreator?.cancelBackup()"
	>
		<div class="flex flex-col gap-6">
			<Admonition type="warning" :header="formatMessage(admonitionHeaderMessage)">
				{{ formatMessage(admonitionBodyMessage) }}
			</Admonition>
			<InlineBackupCreator
				ref="backupCreator"
				:backup-name="props.backupTip ? `Before unlink (${props.backupTip})` : 'Before unlink'"
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
						<UnlinkIcon />
						{{ formatMessage(actionMessage) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { UnlinkIcon, XIcon } from '@modrinth/assets'
import { computed, ref } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { useDebugLogger } from '#ui/composables/debug-logger'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import InlineBackupCreator from './InlineBackupCreator.vue'

const props = defineProps<{
	mode?: 'default' | 'share-instance' | 'shared-instance-managed'
	server?: boolean
	backupTip?: string
	actionDisabled?: boolean
	actionDisabledTooltip?: string
}>()

const { formatMessage } = useVIntl()
const debug = useDebugLogger('ConfirmUnlinkModal')

const messages = defineMessages({
	header: {
		id: 'content.confirm-unlink.header',
		defaultMessage: 'Unlink modpack',
	},
	admonitionHeader: {
		id: 'content.confirm-unlink.admonition-header',
		defaultMessage: 'Unlinking modpack',
	},
	admonitionBody: {
		id: 'content.confirm-unlink.admonition-body',
		defaultMessage:
			'Mods and content will be merged with what you added on top of the modpack, and it will stop receiving updates.',
	},
	shareInstanceAdmonitionHeader: {
		id: 'content.confirm-unlink.share-instance-admonition-header',
		defaultMessage: 'Sharing requires unlinking',
	},
	shareInstanceAdmonitionBody: {
		id: 'content.confirm-unlink.share-instance-admonition-body',
		defaultMessage: 'You must unlink this modpack to share your instance',
	},
	sharedInstanceManagedAdmonitionHeader: {
		id: 'content.confirm-unlink.shared-instance-managed-admonition-header',
		defaultMessage: 'This is managed by the shared instance',
	},
	sharedInstanceManagedAdmonitionBody: {
		id: 'content.confirm-unlink.shared-instance-managed-admonition-body',
		defaultMessage:
			'Unlinking only changes your local copy. Future shared instance updates may restore or change it again.',
	},
	unlinkButton: {
		id: 'content.confirm-unlink.unlink-button',
		defaultMessage: 'Unlink',
	},
})

const emit = defineEmits<{
	(e: 'unlink'): void
}>()

const modal = ref<InstanceType<typeof NewModal>>()
const backupCreator = ref<InstanceType<typeof InlineBackupCreator>>()
const buttonsDisabled = ref(false)
const headerMessage = computed(() => messages.header)
const admonitionHeaderMessage = computed(() => {
	if (props.mode === 'share-instance') return messages.shareInstanceAdmonitionHeader
	if (props.mode === 'shared-instance-managed')
		return messages.sharedInstanceManagedAdmonitionHeader

	return messages.admonitionHeader
})
const admonitionBodyMessage = computed(() => {
	if (props.mode === 'share-instance') return messages.shareInstanceAdmonitionBody
	if (props.mode === 'shared-instance-managed') return messages.sharedInstanceManagedAdmonitionBody

	return messages.admonitionBody
})
const actionMessage = computed(() =>
	props.server && props.mode !== 'share-instance' ? messages.header : messages.unlinkButton,
)

function show() {
	debug('show: called', {
		hasModalRef: !!modal.value,
		hasBackupCreatorRef: !!backupCreator.value,
		buttonsDisabled: buttonsDisabled.value,
		actionDisabled: props.actionDisabled,
	})
	modal.value?.show()
	debug('show: returned from modal.show', {
		hasModalRef: !!modal.value,
		hasBackupCreatorRef: !!backupCreator.value,
		buttonsDisabled: buttonsDisabled.value,
		actionDisabled: props.actionDisabled,
	})
}

function confirm() {
	debug('confirm: called', {
		hasModalRef: !!modal.value,
		buttonsDisabled: buttonsDisabled.value,
		actionDisabled: props.actionDisabled,
	})
	if (props.actionDisabled) {
		debug('confirm: ignored actionDisabled')
		return
	}
	modal.value?.hide()
	emit('unlink')
	debug('confirm: emitted unlink')
}

defineExpose({
	show,
})
</script>
