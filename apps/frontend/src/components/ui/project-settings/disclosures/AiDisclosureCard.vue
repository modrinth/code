<script setup lang="ts">
import { SparklesIcon } from '@modrinth/assets'
import {
	Checkbox,
	commonMessages,
	defineMessages,
	disclosureAiUsageMessages,
	IntlFormatted,
	normalizeChildren,
	SettingsFormGroup,
	Textarea,
	useVIntl,
} from '@modrinth/ui'

import DisclosureToggleCard from './DisclosureToggleCard.vue'
import type { AiDisclosure, AiUsage, DisclosureCardMetaProps, DisclosureLockStatus } from './types'

const model = defineModel<AiDisclosure>({ required: true })

const props = defineProps<DisclosureCardMetaProps>()

const emit = defineEmits<{
	setLockStatus: [status: DisclosureLockStatus]
}>()

const { formatMessage } = useVIntl()

const AI_USES: AiUsage[] = ['code', 'assets', 'text', 'functionality']

const messages = defineMessages({
	title: {
		id: 'project.settings.disclosures.ai.title',
		defaultMessage: 'Contains AI-generated content',
	},
	description: {
		id: 'project.settings.disclosures.ai.description',
		defaultMessage: `You must enable this if this project contains a substantial amount of AI-generated code, any
				assets that are substantially AI-generated, the project's functionality relies on the use of
				generative AI, or if any element of your project's page such as description or publishing
				relies on generative AI.`,
	},
	contentRules: {
		id: 'project.settings.disclosures.ai.content-rules',
		defaultMessage:
			"Please refer to Section 6 of <rules>Modrinth's Content Rules</rules> for more information.",
	},
	typesDescription: {
		id: 'project.settings.disclosures.ai.types-description',
		defaultMessage: 'What is generative AI being used for?',
	},
	notePlaceholder: {
		id: 'project.settings.disclosures.ai.note-placeholder',
		defaultMessage: 'e.g. The Chinese and Arabic translations are AI-generated.',
	},
})

function hasUse(use: AiUsage): boolean {
	return model.value.uses.includes(use)
}

function setUse(use: AiUsage, enabled: boolean) {
	if (enabled) {
		if (!model.value.uses.includes(use)) {
			model.value.uses = [...model.value.uses, use]
		}
		return
	}
	model.value.uses = model.value.uses.filter((entry) => entry !== use)
}
</script>

<template>
	<DisclosureToggleCard
		v-bind="props"
		v-model="model.enabled"
		:icon="SparklesIcon"
		:title="formatMessage(messages.title)"
		:description="formatMessage(messages.description)"
		@set-lock-status="emit('setLockStatus', $event)"
	>
		<p class="text-secondary">
			<IntlFormatted :message-id="messages.contentRules">
				<template #rules="{ children }">
					<nuxt-link
						to="/legal/rules#generative-ai"
						target="_blank"
						class="smart-clickable:allow-pointer-events underline hover:text-primary"
					>
						<component :is="() => normalizeChildren(children)" />
					</nuxt-link>
				</template>
			</IntlFormatted>
		</p>
		<template #expanded>
			<SettingsFormGroup :title="formatMessage(messages.typesDescription)">
				<div class="flex flex-col gap-2">
					<Checkbox
						v-for="use in AI_USES"
						:key="use"
						:model-value="hasUse(use)"
						:disabled="disabled"
						@update:model-value="(enabled) => setUse(use, enabled)"
					>
						{{ formatMessage(disclosureAiUsageMessages[use]) }}
					</Checkbox>
				</div>
			</SettingsFormGroup>
			<SettingsFormGroup
				:title="formatMessage(commonMessages.explanationLabel)"
				title-for="ai-disclosure-note"
				optional
			>
				<Textarea
					id="ai-disclosure-note"
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
