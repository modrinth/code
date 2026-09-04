<script setup lang="ts">
import { MegaphoneIcon } from '@modrinth/assets'
import {
	commonMessages,
	defineMessages,
	IntlFormatted,
	normalizeChildren,
	SettingsFormGroup,
	Textarea,
	useVIntl,
} from '@modrinth/ui'

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
		id: 'project.settings.disclosures.advertising.title',
		defaultMessage: 'Contains advertisements',
	},
	description1: {
		id: 'project.settings.disclosures.advertising.description.1',
		defaultMessage: `Must be enabled if this project contains advertisements, sponsorships, or promotions of other works.`,
	},
	description2: {
		id: 'project.settings.disclosures.advertising.description.2',
		defaultMessage: `This does not pertain to unobtrusive promotion of relevant content with no direct monetary value (such as a link to a Modrinth profile in your own configuration menu), and does not pertain to advertisements in the project page such as sponsored banners in the description.`,
	},
	notePlaceholder: {
		id: 'project.settings.disclosures.advertising.note-placeholder',
		defaultMessage: 'e.g. Adds the Modrinth SMP server to your server list automatically.',
	},
})
</script>

<template>
	<DisclosureToggleCard
		v-bind="props"
		v-model="model.enabled"
		:icon="MegaphoneIcon"
		:title="formatMessage(messages.title)"
		info-link="https://support.modrinth.com/en/articles/16567675#h_d486036510"
		@set-lock-status="emit('setLockStatus', $event)"
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
				<Textarea
					id="advertising-disclosure-note"
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
