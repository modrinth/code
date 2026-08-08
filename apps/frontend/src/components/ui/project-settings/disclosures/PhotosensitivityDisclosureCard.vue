<script setup lang="ts">
import { EyeIcon } from '@modrinth/assets'
import {
	commonMessages,
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
		id: 'project.settings.disclosures.photosensitivity.title',
		defaultMessage: 'Photosensitivity warning',
	},
	description: {
		id: 'project.settings.disclosures.photosensitivity.description',
		defaultMessage:
			'Enable this if your project contains anything that you think may be dangerous to certain people who are sensitive to flashing lights or patterns.',
	},
	notePlaceholder: {
		id: 'project.settings.disclosures.photosensitivity.note-placeholder',
		defaultMessage:
			'e.g. It adds a flashlight item that has a strobe mode. It can be disabled in Accessibility settings in-game.',
	},
})
</script>

<template>
	<SettingsToggleCard
		v-model="model.enabled"
		:disabled="disabled"
		:icon="EyeIcon"
		:title="formatMessage(messages.title)"
		:description="formatMessage(messages.description)"
	>
		<template #expanded>
			<SettingsFormGroup
				:title="formatMessage(commonMessages.explanationLabel)"
				title-for="photosensitivity-disclosure-note"
			>
				<StyledInput
					id="photosensitivity-disclosure-note"
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
