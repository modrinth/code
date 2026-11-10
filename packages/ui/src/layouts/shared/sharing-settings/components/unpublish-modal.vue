<script setup lang="ts">
import { SpinnerIcon, UnlinkIcon, XIcon } from '@modrinth/assets'
import { ref } from 'vue'

import { Admonition, Button } from '#ui/components/base'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, type MessageDescriptor, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

defineProps<{
	busy: boolean
	unpublishing?: boolean
	description: MessageDescriptor
}>()
defineEmits<{ confirm: [] }>()

const { formatMessage } = useVIntl()
const modal = ref<InstanceType<typeof NewModal>>()
const messages = defineMessages({
	unpublishModalHeader: {
		id: 'installation-settings.unpublish-shared-instance.modal.header',
		defaultMessage: 'Unpublish shared instance',
	},
	unpublishModalAdmonitionHeader: {
		id: 'installation-settings.unpublish-shared-instance.modal.admonition-header',
		defaultMessage: 'Unpublishing shared instance',
	},
	unpublishButton: {
		id: 'installation-settings.shared-instance.unpublish-button',
		defaultMessage: 'Unpublish shared instance',
	},
	unpublishingButton: {
		id: 'installation-settings.shared-instance.unpublishing-button',
		defaultMessage: 'Unpublishing...',
	},
})

defineExpose({
	show: () => modal.value?.show(),
	hide: () => modal.value?.hide(),
})
</script>

<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.unpublishModalHeader)"
		fade="warning"
		max-width="500px"
	>
		<Admonition type="warning" :header="formatMessage(messages.unpublishModalAdmonitionHeader)">
			{{ formatMessage(description) }}
		</Admonition>
		<template #actions>
			<div class="flex justify-end gap-2">
				<Button type="outlined" class="!border" @click="modal?.hide()">
					<XIcon />{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button type="colored" color="orange" :disabled="busy" @click="$emit('confirm')">
					<SpinnerIcon v-if="unpublishing" class="animate-spin" />
					<UnlinkIcon v-else />
					{{ formatMessage(unpublishing ? messages.unpublishingButton : messages.unpublishButton) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>
