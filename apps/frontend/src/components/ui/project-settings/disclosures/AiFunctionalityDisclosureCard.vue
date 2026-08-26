<script setup lang="ts">
import { BrainCircuitIcon } from '@modrinth/assets'
import { commonMessages, defineMessages, SettingsFormGroup, Textarea, useVIntl } from '@modrinth/ui'

import DisclosureToggleCard from './DisclosureToggleCard.vue'
import type { DisclosureCardMetaProps, DisclosureLockStatus, NoteDisclosure } from './types'

const model = defineModel<NoteDisclosure>({ required: true })

const props = defineProps<DisclosureCardMetaProps>()

const emit = defineEmits<{
	setLockStatus: [status: DisclosureLockStatus]
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'project.settings.disclosures.ai-functionality.title',
		defaultMessage: 'Contains generative AI functionality',
	},
	description: {
		id: 'project.settings.disclosures.ai-functionality.description',
		defaultMessage:
			'Must be enabled if the project has functionality that makes use of generative AI, such as an in-game chatbot or dynamically generated textures.',
	},
	notePlaceholder: {
		id: 'project.settings.disclosures.ai-functionality.note-placeholder',
		defaultMessage: 'e.g. The NPCs uses generative AI for dialogue options.',
	},
})
</script>

<template>
	<DisclosureToggleCard
		v-bind="props"
		v-model="model.enabled"
		:icon="BrainCircuitIcon"
		:title="formatMessage(messages.title)"
		:description="formatMessage(messages.description)"
		@set-lock-status="emit('setLockStatus', $event)"
	>
		<template #expanded>
			<SettingsFormGroup
				:title="formatMessage(commonMessages.explanationLabel)"
				title-for="ai-functionality-disclosure-note"
				optional
			>
				<Textarea
					id="ai-functionality-disclosure-note"
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
