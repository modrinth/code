<script setup lang="ts">
import { SaveIcon, UndoIcon, XIcon } from '@modrinth/assets'
import {
	Avatar,
	Button,
	commonMessages,
	defineMessages,
	Input,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { ref, shallowRef, useId } from 'vue'

import type { SingleplayerWorld } from '@/platform/app-lib/worlds/types'

import type { EditWorldValues } from '../types'
import HideFromHomeOption from './HideFromHomeOption.vue'

const emit = defineEmits<{
	submit: [values: EditWorldValues]
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'instance.edit-world.title',
		defaultMessage: 'Edit world',
	},
	name: {
		id: 'instance.edit-world.name',
		defaultMessage: 'Name',
	},
	placeholderName: {
		id: 'instance.edit-world.placeholder-name',
		defaultMessage: 'Minecraft World',
	},
	resetIcon: {
		id: 'instance.edit-world.reset-icon',
		defaultMessage: 'Reset icon',
	},
})

const modal = ref<InstanceType<typeof NewModal> | null>(null)

const world = shallowRef<SingleplayerWorld | null>(null)
const name = ref('')
const removeIcon = ref(false)
const hideFromHome = ref(false)

const nameId = useId()

function submit() {
	if (!world.value) return
	emit('submit', {
		path: world.value.path,
		name: name.value,
		removeIcon: removeIcon.value,
		displayStatus: hideFromHome.value ? 'hidden' : 'normal',
		previousDisplayStatus: world.value.display_status,
	})
	hide()
}

function show(value: SingleplayerWorld, event?: MouseEvent) {
	world.value = value
	name.value = value.name
	removeIcon.value = false
	hideFromHome.value = value.display_status === 'hidden'
	modal.value?.show(event)
}

function hide() {
	modal.value?.hide()
}

defineExpose({ show, hide })
</script>

<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.title)"
		width="min(500px, calc(95vw - 10rem))"
	>
		<div class="flex flex-col gap-3">
			<div class="flex items-center gap-3">
				<Avatar :src="removeIcon || !world?.icon ? undefined : world.icon" size="48px" />
				<div class="flex min-w-0 flex-1 flex-col gap-2">
					<label :for="nameId" class="font-semibold text-contrast">
						{{ formatMessage(messages.name) }}
					</label>
					<Input
						:id="nameId"
						v-model="name"
						:placeholder="formatMessage(messages.placeholderName)"
						autocomplete="off"
						wrapper-class="w-full"
					/>
				</div>
			</div>
			<HideFromHomeOption v-model="hideFromHome" />
		</div>
		<template #actions>
			<div class="flex justify-end gap-2">
				<Button type="outlined" @click="hide">
					<XIcon aria-hidden="true" />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button :disabled="removeIcon || !world?.icon" @click="removeIcon = true">
					<UndoIcon aria-hidden="true" />
					{{ formatMessage(messages.resetIcon) }}
				</Button>
				<Button type="colored" color="brand" :disabled="!name" @click="submit">
					<SaveIcon aria-hidden="true" />
					{{ formatMessage(commonMessages.saveChangesButton) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>
