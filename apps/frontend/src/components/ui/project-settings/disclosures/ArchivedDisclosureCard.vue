<script setup lang="ts">
import { ArchiveIcon } from '@modrinth/assets'
import {
	ArchivedProjectBanner,
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
	projectTitle: string
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'project.settings.disclosures.archived.title',
		defaultMessage: 'Archive project',
	},
	description1: {
		id: 'project.settings.disclosures.archived.description.1',
		defaultMessage:
			'Mark your project as archived to let users know that you are no longer working on it.',
	},
	description2: {
		id: 'project.settings.disclosures.archived.description.2',
		defaultMessage:
			'Archived projects remain discoverable when their visibility is set to Public. If you wish to de-list your project, set its visibility to Unlisted',
	},
	noteLabel: {
		id: 'project.settings.disclosures.archived.note-label',
		defaultMessage: 'Optionally explain why this project is archived.',
	},
	notePlaceholder: {
		id: 'project.settings.disclosures.archived.note-placeholder',
		defaultMessage: `e.g. I don't have time to maintain this project anymore, feel free to fork it!`,
	},
	bannerPreview: {
		id: 'project.settings.disclosures.archived.banner-preview',
		defaultMessage: 'Banner preview',
	},
})
</script>

<template>
	<SettingsToggleCard
		v-model="model.enabled"
		:disabled="disabled"
		:icon="ArchiveIcon"
		:title="formatMessage(messages.title)"
	>
		<p>{{ formatMessage(messages.description1) }}</p>
		<p>{{ formatMessage(messages.description2) }}</p>
		<template #expanded>
			<SettingsFormGroup
				:title="formatMessage(messages.noteLabel)"
				title-for="archived-disclosure-note"
			>
				<StyledInput
					id="archived-disclosure-note"
					v-model="model.note"
					multiline
					:rows="3"
					class="max-w-[40rem]"
					:disabled="disabled"
					:placeholder="formatMessage(messages.notePlaceholder)"
				/>
			</SettingsFormGroup>
			<SettingsFormGroup :title="formatMessage(messages.bannerPreview)">
				<ArchivedProjectBanner :title="projectTitle" :reason="model.note" />
			</SettingsFormGroup>
		</template>
	</SettingsToggleCard>
</template>
