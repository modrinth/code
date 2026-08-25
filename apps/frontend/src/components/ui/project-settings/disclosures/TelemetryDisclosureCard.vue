<script setup lang="ts">
import { ListPlusIcon, RadioTowerIcon, TrashIcon } from '@modrinth/assets'
import {
	Button,
	Chips,
	commonMessages,
	defineMessages,
	disclosureTelemetryConsentMessages,
	IconButton,
	Input,
	SettingsFormGroup,
	useVIntl,
} from '@modrinth/ui'
import { watch } from 'vue'

import DisclosureToggleCard from './DisclosureToggleCard.vue'
import type {
	DisclosureCardMetaProps,
	DisclosureLockStatus,
	TelemetryConsent,
	TelemetryDisclosure,
} from './types'

const model = defineModel<TelemetryDisclosure>({ required: true })

const props = defineProps<DisclosureCardMetaProps>()

const emit = defineEmits<{
	setLockStatus: [status: DisclosureLockStatus]
}>()

const { formatMessage } = useVIntl()

const CONSENT_MODELS: TelemetryConsent[] = ['opt_in', 'opt_out', 'always_active']

const messages = defineMessages({
	title: {
		id: 'project.settings.disclosures.telemetry.title',
		defaultMessage: 'Contains telemetry',
	},
	description: {
		id: 'project.settings.disclosures.telemetry.description',
		defaultMessage:
			'You must enable this if your project sends usage data back to yourself or a third party.',
	},
	consentDescription: {
		id: 'project.settings.disclosures.telemetry.consent-description',
		defaultMessage: 'What is the consent model of your telemetry?',
	},
	dataLabel: {
		id: 'project.settings.disclosures.telemetry.data-label',
		defaultMessage: 'What data is collected?',
	},
	dataDescription: {
		id: 'project.settings.disclosures.telemetry.data-description',
		defaultMessage:
			'You can either add a privacy policy, or a list of types of data that are collected. Remember to mention if it is anonymous or contains personally identifiable information (PII).',
	},
	dataPlaceholder: {
		id: 'project.settings.disclosures.telemetry.data-placeholder',
		defaultMessage:
			'e.g. Anonymous launch analytics to track Minecraft version and mod loader usage.',
	},
})

watch(
	() => model.value.enabled,
	(enabled) => {
		if (enabled && model.value.entries.length === 0) {
			model.value.entries = ['']
		}
	},
)

function addEntry() {
	model.value.entries = [...model.value.entries, '']
}

function removeEntry(index: number) {
	if (model.value.entries.length <= 1) return
	model.value.entries = model.value.entries.filter((_, i) => i !== index)
}
</script>

<template>
	<DisclosureToggleCard
		v-bind="props"
		v-model="model.enabled"
		:icon="RadioTowerIcon"
		:title="formatMessage(messages.title)"
		:description="formatMessage(messages.description)"
		@set-lock-status="emit('setLockStatus', $event)"
	>
		<template #expanded>
			<SettingsFormGroup :title="formatMessage(messages.consentDescription)">
				<Chips
					v-model="model.consent"
					:items="CONSENT_MODELS"
					:capitalize="false"
					:format-label="
						(item: TelemetryConsent) => formatMessage(disclosureTelemetryConsentMessages[item])
					"
				/>
			</SettingsFormGroup>
			<SettingsFormGroup
				:title="formatMessage(messages.dataLabel)"
				:description="formatMessage(messages.dataDescription)"
			>
				<div v-for="(_, index) in model.entries" :key="index" class="flex items-center gap-2">
					<Input
						v-model="model.entries[index]"
						class="min-w-0 flex-1"
						:disabled="disabled"
						:placeholder="formatMessage(messages.dataPlaceholder)"
					/>
					<IconButton
						v-if="model.entries.length > 1"
						:label="formatMessage(commonMessages.removeButton)"
						:disabled="disabled"
						@click="removeEntry(index)"
					>
						<TrashIcon />
					</IconButton>
				</div>
			</SettingsFormGroup>
			<Button class="w-fit" :disabled="disabled" @click="addEntry">
				<ListPlusIcon />
				{{ formatMessage(commonMessages.addAnotherButton) }}
			</Button>
		</template>
	</DisclosureToggleCard>
</template>
