<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.header, { type: server ? 'server' : 'instance' })"
		max-width="500px"
	>
		<span class="text-primary">
			{{ formatMessage(messages.body, { type: server ? 'server' : 'instance' }) }}
		</span>

		<template #actions>
			<div class="flex gap-2 justify-end">
				<ButtonStyled type="outlined">
					<button class="!border !border-surface-4" @click="modal?.hide()">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="green">
					<button @click="confirm">
						<HammerIcon />
						{{ formatMessage(messages.repairButton) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { HammerIcon, XIcon } from '@modrinth/assets'
import { ref } from 'vue'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

defineProps<{
	server?: boolean
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	header: {
		id: 'instance.confirm-repair.header',
		defaultMessage: 'Repair {type, select, server {server} other {instance}}',
	},
	body: {
		id: 'instance.confirm-repair.body',
		defaultMessage:
			'Repairing reinstalls the loader and Minecraft dependencies without deleting your content. This may resolve issues if your {type, select, server {server is not starting correctly} other {game is not launching due to launcher-related errors}}.',
	},
	repairButton: {
		id: 'instance.confirm-repair.repair-button',
		defaultMessage: 'Repair',
	},
})

const emit = defineEmits<{
	(e: 'repair'): void
}>()

const modal = ref<InstanceType<typeof NewModal>>()

function show() {
	modal.value?.show()
}

function confirm() {
	modal.value?.hide()
	emit('repair')
}

defineExpose({
	show,
})
</script>
