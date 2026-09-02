<script setup lang="ts">
import { CircuitBoardIcon } from '@modrinth/assets'
import { defineMessages, SettingsFormGroup, Textarea, useVIntl } from '@modrinth/ui'

import DisclosureToggleCard from './DisclosureToggleCard.vue'
import type {
	DisclosureCardMetaProps,
	DisclosureLockStatus,
	SystemInteractionsDisclosure,
} from './types'

const model = defineModel<SystemInteractionsDisclosure>({ required: true })

const props = defineProps<DisclosureCardMetaProps>()

const emit = defineEmits<{
	setLockStatus: [status: DisclosureLockStatus]
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'project.settings.disclosures.system-interactions.title',
		defaultMessage: 'External system interactions',
	},
	description: {
		id: 'project.settings.disclosures.system-interactions.description',
		defaultMessage:
			"You must enable this if your project reads or edits things on the user's system outside of the game.",
	},
	noteLabel: {
		id: 'project.settings.disclosures.system-interactions.note-label',
		defaultMessage: 'Describe the external system interactions',
	},
	notePlaceholder: {
		id: 'project.settings.disclosures.system-interactions.note-placeholder',
		defaultMessage: 'e.g. It adds a file to the desktop called wake_up.txt',
	},
})
</script>

<template>
	<DisclosureToggleCard
		v-bind="props"
		v-model="model.enabled"
		:icon="CircuitBoardIcon"
		:title="formatMessage(messages.title)"
		:description="formatMessage(messages.description)"
		@set-lock-status="emit('setLockStatus', $event)"
	>
		<template #expanded>
			<SettingsFormGroup
				:title="formatMessage(messages.noteLabel)"
				title-for="system-interactions-disclosure-note"
			>
				<Textarea
					id="system-interactions-disclosure-note"
					v-model="model.note"
					:rows="3"
					class="max-w-[40rem]"
					:disabled="disabled"
					:placeholder="formatMessage(messages.notePlaceholder)"
				/>
			</SettingsFormGroup>
		</template>
	</DisclosureToggleCard>
</template>
