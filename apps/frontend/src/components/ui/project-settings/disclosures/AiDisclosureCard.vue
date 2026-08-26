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

const AI_USES: AiUsage[] = ['code', 'assets', 'text']

const messages = defineMessages({
	title: {
		id: 'project.settings.disclosures.ai.title',
		defaultMessage: 'Contains AI-generated content',
	},
	description: {
		id: 'project.settings.disclosures.ai.description',
		defaultMessage: `Must be enabled if this project contains any AI-generated assets or text, or if it contains a substantial amount of AI-generated code.`,
	},
	description2: {
		id: 'project.settings.disclosures.ai.description.2',
		defaultMessage: `It is important to be honest if generative AI has played a significant role in the production or publishing of this content. An easy way to think about this is if you would consider AI a co-author of the project, you should disclose it here.`,
	},
	stillUnsure: {
		id: 'project.settings.disclosures.ai.still-unsure',
		defaultMessage:
			"Still unsure? Refer to our guide to <faq-link>Disclosure and Usage of AI</faq-link> and section 6 of <rules>Modrinth's Content Rules</rules> for more information.",
	},
	typesDescription: {
		id: 'project.settings.disclosures.ai.types-description',
		defaultMessage: 'What types of content have been generated with AI?',
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
		<p>
			{{ formatMessage(messages.description2) }}
		</p>
		<p class="text-secondary">
			<IntlFormatted :message-id="messages.stillUnsure">
				<template #faq-link="{ children }">
					<a
						href="https://support.modrinth.com/en/articles/16551575"
						target="_blank"
						class="smart-clickable:allow-pointer-events underline hover:text-primary"
					>
						<component :is="() => normalizeChildren(children)" />
					</a>
				</template>
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
