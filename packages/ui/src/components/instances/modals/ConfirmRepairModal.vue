<template>
	<NewModal ref="modal" :header="formatMessage(server ? messages.serverHeader : messages.header)" fade="warning" max-width="500px">
		<div class="flex flex-col gap-6">
			<Admonition type="warning" :header="formatMessage(messages.admonitionHeader)">
				{{ formatMessage(server ? messages.serverAdmonitionBody : messages.admonitionBody) }}
			</Admonition>
		</div>

		<template #actions>
			<div class="flex gap-2 justify-end">
				<ButtonStyled type="outlined">
					<button class="!border !border-surface-4" @click="modal?.hide()">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="orange">
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

import { defineMessages, useVIntl } from '../../../composables/i18n'
import { commonMessages } from '../../../utils/common-messages'
import Admonition from '../../base/Admonition.vue'
import ButtonStyled from '../../base/ButtonStyled.vue'
import NewModal from '../../modal/NewModal.vue'

defineProps<{
	server?: boolean
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	header: {
		id: 'instance.confirm-repair.header',
		defaultMessage: 'Repair instance',
	},
	serverHeader: {
		id: 'instance.confirm-repair.server-header',
		defaultMessage: 'Repair server',
	},
	admonitionHeader: {
		id: 'instance.confirm-repair.admonition-header',
		defaultMessage: 'Repair warning',
	},
	admonitionBody: {
		id: 'instance.confirm-repair.admonition-body',
		defaultMessage:
			'Repairing reinstalls Minecraft dependencies and checks for corruption. This may resolve issues if your game is not launching due to launcher-related errors, but will not resolve issues or crashes related to installed mods.',
	},
	serverAdmonitionBody: {
		id: 'instance.confirm-repair.server-admonition-body',
		defaultMessage:
			'Repairing reinstalls the loader and Minecraft dependencies without deleting your content. This may resolve issues if your server is not starting correctly.',
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
