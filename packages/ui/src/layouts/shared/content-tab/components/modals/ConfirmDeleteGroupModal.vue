<template>
	<NewModal ref="modal" :header="formatMessage(messages.header)" max-width="500px">
		<p class="m-0 max-w-[35rem]">
			{{ formatMessage(messages.description, { name: groupName }) }}
		</p>

		<template #actions>
			<div class="flex justify-end gap-2">
				<Button type="outlined" @click="modal?.hide()">
					<XIcon />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button type="colored" color="red" @click="confirm">
					<TrashIcon />
					{{ formatMessage(messages.header) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { TrashIcon, XIcon } from '@modrinth/assets'
import { ref } from 'vue'

import { Button } from '#ui/components/base/buttons'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

const emit = defineEmits<{
	delete: [groupId: string]
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	header: {
		id: 'content.groups.delete-modal.header',
		defaultMessage: 'Delete group',
	},
	description: {
		id: 'content.groups.delete-description',
		defaultMessage:
			'Content in "{name}" will become ungrouped. Nothing will be removed from your instance.',
	},
})

const modal = ref<InstanceType<typeof NewModal>>()
const groupId = ref<string | null>(null)
const groupName = ref('')

function show(id: string, name: string, event?: MouseEvent) {
	groupId.value = id
	groupName.value = name
	modal.value?.show(event)
}

function confirm() {
	modal.value?.hide()
	if (groupId.value) emit('delete', groupId.value)
}

defineExpose({ show })
</script>
