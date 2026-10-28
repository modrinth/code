<template>
	<div v-if="canUnlink" class="flex flex-col gap-2.5">
		<span class="text-lg font-semibold text-contrast">{{
			formatMessage(messages.linkedTitle)
		}}</span>
		<div>
			<Button type="colored" color="orange" :disabled="busy" @click="unlinkModal?.show()">
				<SpinnerIcon v-if="unlinking" class="animate-spin" />
				<UnlinkIcon v-else class="size-5" />
				{{ formatMessage(unlinking ? messages.unlinkingButton : messages.unlinkButton) }}
			</Button>
		</div>
		<span class="text-primary">{{ formatMessage(messages.unlinkDescription) }}</span>
	</div>

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
				<Button type="outlined" class="!border" @click="unlinkModal?.hide()">
					<XIcon />{{ formatMessage(commonMessages.cancelButton) }} </Button
				><Button
					type="colored"
					color="orange"
					:disabled="busy || backupBusy"
					@click="confirmUnlink"
				>
					<UnlinkIcon />{{ formatMessage(messages.unlinkButton) }}
				</Button>
			</div></template
		>
	</NewModal>
</template>

<script setup lang="ts">
import { SpinnerIcon, UnlinkIcon, XIcon } from '@modrinth/assets'
import {
	Admonition,
	Button,
	commonMessages,
	defineMessages,
	InlineBackupCreator,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { ref } from 'vue'

const props = withDefaults(
	defineProps<{
		canUnlink?: boolean
		busy: boolean
		unlinking?: boolean
		unlink?: () => Promise<void>
	}>(),
	{
		canUnlink: false,
		unlinking: false,
		unlink: undefined,
	},
)
const { formatMessage } = useVIntl()
const unlinkModal = ref<InstanceType<typeof NewModal>>()
const backupCreator = ref<InstanceType<typeof InlineBackupCreator>>()
const backupBusy = ref(false)

async function confirmUnlink() {
	unlinkModal.value?.hide()
	await props.unlink?.()
}

const messages = defineMessages({
	linkedTitle: {
		id: 'installation-settings.shared-instance.linked-title',
		defaultMessage: 'Linked shared instance',
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
