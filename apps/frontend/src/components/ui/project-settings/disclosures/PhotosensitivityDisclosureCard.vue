<script setup lang="ts">
import { EyeIcon } from '@modrinth/assets'
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
	<DisclosureToggleCard
		v-bind="props"
		v-model="model.enabled"
		:icon="EyeIcon"
		:title="formatMessage(messages.title)"
		:description="formatMessage(messages.description)"
		@set-lock-status="emit('setLockStatus', $event)"
	>
		<template #expanded>
			<SettingsFormGroup
				:title="formatMessage(commonMessages.explanationLabel)"
				title-for="photosensitivity-disclosure-note"
			>
				<Textarea
					id="photosensitivity-disclosure-note"
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
