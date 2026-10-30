<script setup lang="ts">
import { ArchiveIcon } from '@modrinth/assets'
import {
	ArchivedProjectBanner,
	commonMessages,
	defineMessages,
	SettingsFormGroup,
	useVIntl,
} from '@modrinth/ui'

import DisclosureExplanation from './DisclosureExplanation.vue'
import DisclosureToggleCard from './DisclosureToggleCard.vue'
import type { DisclosureCardMetaProps, DisclosureLockStatus, NoteDisclosure } from './types'

const model = defineModel<NoteDisclosure>({ required: true })

const props = defineProps<
	DisclosureCardMetaProps & {
		projectTitle: string
	}
>()

const emit = defineEmits<{
	setLockStatus: [status: DisclosureLockStatus]
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
			'Archived projects remain discoverable when their visibility is set to Public. If you wish to de-list your project, set its visibility to Unlisted.',
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
	<DisclosureToggleCard
		v-bind="props"
		v-model="model.enabled"
		:icon="ArchiveIcon"
		:title="props.title ?? formatMessage(messages.title)"
		info-link="https://support.modrinth.com/en/articles/16567675#h_76b62c699c"
		@set-lock-status="emit('setLockStatus', $event)"
	>
		<p>{{ formatMessage(messages.description1) }}</p>
		<p>{{ formatMessage(messages.description2) }}</p>
		<template #expanded>
			<DisclosureExplanation
				v-model="model.note"
				:title="formatMessage(commonMessages.explanationLabel)"
				title-for="archived-disclosure-note"
				optional
				:variant="variant"
				:resize="showLockControls ? 'vertical' : 'none'"
				:disabled="disabled"
				:placeholder="formatMessage(messages.notePlaceholder)"
			/>
			<SettingsFormGroup v-if="variant !== 'review'" :title="formatMessage(messages.bannerPreview)">
				<ArchivedProjectBanner :title="projectTitle" :reason="model.note" />
			</SettingsFormGroup>
		</template>
	</DisclosureToggleCard>
</template>
