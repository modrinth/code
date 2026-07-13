<template>
	<div v-if="canUnpublish" class="flex flex-col gap-2.5">
		<span class="text-lg font-semibold text-contrast">{{ formatMessage(messages.title) }}</span>
		<div>
			<ButtonStyled color="orange">
				<button :disabled="busy" @click="unpublishModal?.show()">
					<SpinnerIcon v-if="unpublishing" class="animate-spin" />
					<UnlinkIcon v-else class="size-5" />
					{{ formatMessage(unpublishing ? messages.unpublishingButton : messages.unpublishButton) }}
				</button>
			</ButtonStyled>
		</div>
		<span class="text-primary">{{ formatMessage(messages.unpublishDescription) }}</span>
	</div>
	<div v-if="canUnlink" class="flex flex-col gap-2.5">
		<span class="text-lg font-semibold text-contrast">{{
			formatMessage(messages.linkedTitle)
		}}</span>
		<div>
			<ButtonStyled color="orange">
				<button :disabled="busy" @click="unlinkModal?.show()">
					<SpinnerIcon v-if="unlinking" class="animate-spin" />
					<UnlinkIcon v-else class="size-5" />
					{{ formatMessage(unlinking ? messages.unlinkingButton : messages.unlinkButton) }}
				</button>
			</ButtonStyled>
		</div>
		<span class="text-primary">{{ formatMessage(messages.unlinkDescription) }}</span>
	</div>

	<NewModal
		ref="unpublishModal"
		:header="formatMessage(messages.unpublishModalHeader)"
		fade="warning"
		max-width="500px"
	>
		<Admonition type="warning" :header="formatMessage(messages.unpublishModalAdmonitionHeader)">{{
			formatMessage(messages.unpublishModalBody)
		}}</Admonition>
		<template #actions
			><div class="flex justify-end gap-2">
				<ButtonStyled
					><button @click="unpublishModal?.hide()">
						<XIcon />{{ formatMessage(commonMessages.cancelButton) }}
					</button></ButtonStyled
				><ButtonStyled color="orange"
					><button :disabled="busy" @click="confirmUnpublish">
						<UnlinkIcon />{{ formatMessage(messages.unpublishButton) }}
					</button></ButtonStyled
				>
			</div></template
		>
	</NewModal>
	<NewModal
		ref="unlinkModal"
		:header="formatMessage(messages.unlinkModalHeader)"
		fade="warning"
		max-width="500px"
		:on-hide="() => backupCreator?.cancelBackup()"
	>
		<div class="flex flex-col gap-6">
			<Admonition type="warning" :header="formatMessage(messages.unlinkModalAdmonitionHeader)">{{
				formatMessage(messages.unlinkModalBody)
			}}</Admonition>
			<InlineBackupCreator
				ref="backupCreator"
				backup-name="Before unlinking shared instance"
				@update:buttons-disabled="backupBusy = $event"
			/>
		</div>
		<template #actions
			><div class="flex justify-end gap-2">
				<ButtonStyled
					><button @click="unlinkModal?.hide()">
						<XIcon />{{ formatMessage(commonMessages.cancelButton) }}
					</button></ButtonStyled
				><ButtonStyled color="orange"
					><button :disabled="busy || backupBusy" @click="confirmUnlink">
						<UnlinkIcon />{{ formatMessage(messages.unlinkButton) }}
					</button></ButtonStyled
				>
			</div></template
		>
	</NewModal>
</template>

<script setup lang="ts">
import { SpinnerIcon, UnlinkIcon, XIcon } from '@modrinth/assets'
import {
	Admonition,
	ButtonStyled,
	commonMessages,
	defineMessages,
	InlineBackupCreator,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { ref } from 'vue'

const props = defineProps<{
	canUnpublish: boolean
	canUnlink: boolean
	busy: boolean
	unpublishing: boolean
	unlinking: boolean
	unpublish: () => Promise<void>
	unlink: () => Promise<void>
}>()
const { formatMessage } = useVIntl()
const unpublishModal = ref<InstanceType<typeof NewModal>>()
const unlinkModal = ref<InstanceType<typeof NewModal>>()
const backupCreator = ref<InstanceType<typeof InlineBackupCreator>>()
const backupBusy = ref(false)

async function confirmUnpublish() {
	unpublishModal.value?.hide()
	await props.unpublish()
}
async function confirmUnlink() {
	unlinkModal.value?.hide()
	await props.unlink()
}

const messages = defineMessages({
	title: { id: 'installation-settings.shared-instance.title', defaultMessage: 'Shared instance' },
	linkedTitle: {
		id: 'installation-settings.shared-instance.linked-title',
		defaultMessage: 'Linked shared instance',
	},
	unpublishButton: {
		id: 'installation-settings.shared-instance.unpublish-button',
		defaultMessage: 'Unpublish shared instance',
	},
	unpublishingButton: {
		id: 'installation-settings.shared-instance.unpublishing-button',
		defaultMessage: 'Unpublishing...',
	},
	unpublishDescription: {
		id: 'installation-settings.shared-instance.unpublish-description',
		defaultMessage: 'Stop sharing this instance and remove it from Modrinth.',
	},
	unlinkButton: {
		id: 'installation-settings.shared-instance.unlink-button',
		defaultMessage: 'Unlink shared instance',
	},
	unlinkingButton: {
		id: 'installation-settings.shared-instance.unlinking-button',
		defaultMessage: 'Unlinking...',
	},
	unlinkDescription: {
		id: 'installation-settings.shared-instance.unlink-description',
		defaultMessage: 'Disconnect this local instance from future shared updates.',
	},
	unpublishModalHeader: {
		id: 'installation-settings.unpublish-shared-instance.modal.header',
		defaultMessage: 'Unpublish shared instance',
	},
	unpublishModalAdmonitionHeader: {
		id: 'installation-settings.unpublish-shared-instance.modal.admonition-header',
		defaultMessage: 'Unpublishing shared instance',
	},
	unpublishModalBody: {
		id: 'installation-settings.unpublish-shared-instance.modal.admonition-body',
		defaultMessage:
			"This deletes the shared instance from Modrinth's servers. People using it in the Modrinth App will stop receiving updates, but your local instance and its content will stay on this device.",
	},
	unlinkModalHeader: {
		id: 'installation-settings.unlink-shared-instance.modal.header',
		defaultMessage: 'Unlink shared instance',
	},
	unlinkModalAdmonitionHeader: {
		id: 'installation-settings.unlink-shared-instance.modal.admonition-header',
		defaultMessage: 'Unlinking shared instance',
	},
	unlinkModalBody: {
		id: 'installation-settings.unlink-shared-instance.modal.admonition-body',
		defaultMessage:
			'This only affects your local instance. Your installed content will stay on this device, and the shared instance and other people using it will not be affected.',
	},
})
</script>
