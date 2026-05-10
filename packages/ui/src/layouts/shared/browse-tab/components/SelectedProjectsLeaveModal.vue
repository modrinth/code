<template>
	<NewModal ref="modal" fade="warning" :header="formatMessage(messages.header)" max-width="560px">
		<div class="flex flex-col gap-6">
			{{ formatMessage(messages.admonitionBody, { count }) }}
		</div>

		<template #actions>
			<div class="flex flex-wrap justify-end gap-2">
				<ButtonStyled type="outlined">
					<button @click="resolve('cancel')">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="red">
					<button :disabled="installing" @click="resolve('discard')">
						<TrashIcon />
						{{ formatMessage(messages.discardButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="green">
					<button :disabled="installing" @click="resolve('install')">
						<PlusIcon />
						{{ formatMessage(commonMessages.installButton) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { PlusIcon, TrashIcon, XIcon } from '@modrinth/assets'
import { ref } from 'vue'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	header: {
		id: 'browse.selected-projects-leave-modal.header',
		defaultMessage: 'Selected projects not installed yet',
	},
	admonitionHeader: {
		id: 'browse.selected-projects-leave-modal.admonition-header',
		defaultMessage: 'Selected projects not installed yet',
	},
	admonitionBody: {
		id: 'browse.selected-projects-leave-modal.admonition-body',
		defaultMessage:
			'You have selected {count, plural, one {# project} other {# projects}} to install. Install them now or go back without installing them.',
	},
	discardButton: {
		id: 'browse.selected-projects-leave-modal.discard',
		defaultMessage: 'Discard',
	},
})

defineProps<{
	count: number
	installing?: boolean
}>()

type SelectedProjectsLeaveResult = 'cancel' | 'discard' | 'install'

const modal = ref<InstanceType<typeof NewModal>>()
let resolvePromise: ((value: SelectedProjectsLeaveResult) => void) | null = null

function prompt(): Promise<SelectedProjectsLeaveResult> {
	return new Promise((resolve) => {
		resolvePromise = resolve
		modal.value?.show()
	})
}

function resolve(result: SelectedProjectsLeaveResult) {
	modal.value?.hide()
	resolvePromise?.(result)
	resolvePromise = null
}

defineExpose({ prompt })
</script>
