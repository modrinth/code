<script setup lang="ts">
import { SparklesIcon } from '@modrinth/assets'
import {
	Checkbox,
	commonMessages,
	defineMessages,
	IntlFormatted,
	normalizeChildren,
	SettingsFormGroup,
	SettingsToggleCard,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'

import type { AiDisclosure, AiUsage } from './types'

const model = defineModel<AiDisclosure>({ required: true })

defineProps<{
	disabled?: boolean
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
			"Please refer to Section 6 of Modrinth's <rules>Content Rules</rules> for more information.",
	},
	typesDescription: {
		id: 'project.settings.disclosures.ai.types-description',
		defaultMessage: 'What is generative AI being used for?',
	},
	typeCode: {
		id: 'project.settings.disclosures.ai.types-code',
		defaultMessage: 'Code',
	},
	typeAssets: {
		id: 'project.settings.disclosures.ai.types-assets',
		defaultMessage: 'Assets',
	},
	typeText: {
		id: 'project.settings.disclosures.ai.types-text',
		defaultMessage: 'Text',
	},
	typeFunctionality: {
		id: 'project.settings.disclosures.ai.types-functionality',
		defaultMessage: 'Functionality',
	},
	notePlaceholder: {
		id: 'project.settings.disclosures.ai.note-placeholder',
		defaultMessage: 'e.g. The Chinese and Arabic translations are AI-generated.',
	},
})

const useLabels = {
	code: messages.typeCode,
	assets: messages.typeAssets,
	text: messages.typeText,
	functionality: messages.typeFunctionality,
} as const

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
	<SettingsToggleCard
		v-model="model.enabled"
		:disabled="disabled"
		:icon="SparklesIcon"
		:title="formatMessage(messages.title)"
		:description="formatMessage(messages.description)"
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
				<div class="grid gap-2 sm:grid-cols-4">
					<Checkbox
						v-for="use in AI_USES"
						:key="use"
						:model-value="hasUse(use)"
						:disabled="disabled"
						@update:model-value="(enabled) => setUse(use, enabled)"
					>
						{{ formatMessage(useLabels[use]) }}
					</Checkbox>
				</div>
			</SettingsFormGroup>
			<SettingsFormGroup
				:title="formatMessage(commonMessages.explanationLabel)"
				title-for="ai-disclosure-note"
				optional
			>
				<StyledInput
					id="ai-disclosure-note"
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
