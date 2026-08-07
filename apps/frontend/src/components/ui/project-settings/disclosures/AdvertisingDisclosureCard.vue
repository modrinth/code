<script setup lang="ts">
import { MegaphoneIcon } from '@modrinth/assets'
import {
	commonMessages,
	defineMessages,
	IntlFormatted,
	normalizeChildren,
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
		id: 'project.settings.disclosures.advertising.title',
		defaultMessage: 'Contains advertisements',
	},
	description1: {
		id: 'project.settings.disclosures.advertising.description.1',
		defaultMessage: `You must enable this if your project contains advertisements, sponsorships, or promotions of other works.`,
	},
	description2: {
		id: 'project.settings.disclosures.advertising.description.2',
		defaultMessage: `If the promotion has no direct monetary value <emphasis>and</emphasis> it is for something that the average person would consider <italic>relevant</italic> and <italic>unobtrusive</italic> (such as a link to your Modrinth profile in the corner of the settings page for your own mod), we would not consider that an advertisement.`,
	},
	notePlaceholder: {
		id: 'project.settings.disclosures.advertising.note-placeholder',
		defaultMessage: 'e.g. Adds the Modrinth SMP server to your server list automatically.',
	},
})
</script>

<template>
	<SettingsToggleCard
		v-model="model.enabled"
		:disabled="disabled"
		:icon="MegaphoneIcon"
		:title="formatMessage(messages.title)"
	>
		<p>{{ formatMessage(messages.description1) }}</p>
		<p>
			<IntlFormatted :message-id="messages.description2">
				<template #italic="{ children }">
					<span class="italic">
						<component :is="() => normalizeChildren(children)" />
					</span>
				</template>
				<template #emphasis="{ children }">
					<span class="font-bold italic">
						<component :is="() => normalizeChildren(children)" />
					</span>
				</template>
			</IntlFormatted>
		</p>
		<template #expanded>
			<SettingsFormGroup
				:title="formatMessage(commonMessages.explanationLabel)"
				title-for="advertising-disclosure-note"
			>
				<StyledInput
					id="advertising-disclosure-note"
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
