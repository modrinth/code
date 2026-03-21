<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.header, { type: item?.type })"
		max-width="500px"
	>
		<form class="space-y-6 md:min-w-[400px]" @submit.prevent="handleSubmit">
			<div class="flex flex-col gap-1">
				<span class="font-semibold text-contrast">{{
					formatMessage(messages.currentLocation)
				}}</span>
				<span class="text-secondary">{{ `${currentPath}/${item?.name}`.replace('//', '/') }}</span>
			</div>
			<label class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{
					formatMessage(messages.destinationPath)
				}}</span>
				<StyledInput
					ref="destinationInput"
					v-model="destination"
					:placeholder="formatMessage(messages.destinationPlaceholder)"
					wrapper-class="w-full"
				/>
			</label>
		</form>
		<template #actions>
			<div class="flex gap-2 justify-end">
				<ButtonStyled type="outlined">
					<button class="!border !border-surface-4" @click="hide">
						<XIcon class="h-5 w-5" />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button @click="handleSubmit">
						<RightArrowIcon class="h-5 w-5" />
						{{ formatMessage(messages.moveButton) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { RightArrowIcon, XIcon } from '@modrinth/assets'
import { nextTick, ref } from 'vue'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import StyledInput from '#ui/components/base/StyledInput.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import type { FileItem } from '../../types'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	header: {
		id: 'files.move-modal.header',
		defaultMessage: '{type, select, directory {Move folder} other {Move file}}',
	},
	currentLocation: {
		id: 'files.move-modal.current-location',
		defaultMessage: 'Current location',
	},
	destinationPath: {
		id: 'files.move-modal.destination-path',
		defaultMessage: 'Destination path',
	},
	moveButton: {
		id: 'files.move-modal.move-button',
		defaultMessage: 'Move',
	},
	destinationPlaceholder: {
		id: 'files.move-modal.destination-placeholder',
		defaultMessage: 'e.g. /mods',
	},
})

const destinationInput = ref<HTMLInputElement | null>(null)

defineProps<{
	item: Pick<FileItem, 'name' | 'type'> | null
	currentPath: string
}>()

const emit = defineEmits<{
	move: [destination: string]
}>()

const modal = ref<InstanceType<typeof NewModal>>()
const destination = ref('')

const handleSubmit = () => {
	const path = destination.value.replace('//', '/')
	const normalized = path.startsWith('/') ? path : `/${path}`
	emit('move', normalized)
	hide()
}

const show = () => {
	destination.value = ''
	modal.value?.show()
	nextTick(() => {
		setTimeout(() => {
			destinationInput.value?.focus()
		}, 100)
	})
}

const hide = () => {
	modal.value?.hide()
}

defineExpose({ show, hide })
</script>
