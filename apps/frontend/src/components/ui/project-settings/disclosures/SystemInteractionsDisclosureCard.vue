<script setup lang="ts">
import { CircuitBoardIcon } from '@modrinth/assets'
import {
	defineMessages,
	SettingsFormGroup,
	SettingsToggleCard,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'

import type { NoteDisclosure } from './types'

const model = defineModel<NoteDisclosure>({ required: true })

defineProps<{
	disabled?: boolean
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
		defaultMessage:
			'Please describe the external system interactions in the mod so users can know what to expect.',
	},
	notePlaceholder: {
		id: 'project.settings.disclosures.system-interactions.note-placeholder',
		defaultMessage: 'e.g. It adds a file to the desktop called wake_up.txt',
	},
})
</script>

<template>
	<SettingsToggleCard
		v-model="model.enabled"
		:disabled="disabled"
		:icon="CircuitBoardIcon"
		:title="formatMessage(messages.title)"
		:description="formatMessage(messages.description)"
	>
		<template #expanded>
			<SettingsFormGroup
				:title="formatMessage(messages.noteLabel)"
				title-for="system-interactions-disclosure-note"
			>
				<StyledInput
					id="system-interactions-disclosure-note"
					v-model="model.note"
					multiline
					:rows="3"
					class="max-w-[40rem]"
					:disabled="disabled"
					:placeholder="formatMessage(messages.notePlaceholder)"
				/>
			</SettingsFormGroup>
		</template>
	</SettingsToggleCard>
</template>
