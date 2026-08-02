<script setup lang="ts">
import {
	CircleDollarSignIcon,
	ListPlusIcon,
	MegaphoneIcon,
	PlusIcon,
	RadioTowerIcon,
	SparklesIcon,
} from '@modrinth/assets'
import {
	ButtonStyled,
	Checkbox,
	defineMessages,
	IntlFormatted,
	normalizeChildren,
	StyledInput,
	ToggleCard,
	useVIntl,
} from '@modrinth/ui'

import Chips from '~/components/ui/Chips.vue'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'project.settings.disclosures.content-disclosures',
		defaultMessage: 'Content disclosures',
	},
	description: {
		id: 'project.settings.disclosures.description',
		defaultMessage: `You must add any applicable content disclosures to your project in compliance with Modrinth's <rules>Content Rules</rules>.`,
	},
	addAnother: {
		id: 'project.settings.disclosures.add-another',
		defaultMessage: 'Add another',
	},
})

const aiDisclosure = ref(false)
const aiDisclosureTypes = ref({
	code: false,
	assets: false,
	text: false,
})
const aiDisclosureMessages = defineMessages({
	title: {
		id: 'project.settings.disclosures.ai.title',
		defaultMessage: 'Contains AI-generated content',
	},
	description: {
		id: 'project.settings.disclosures.ai.description',
		defaultMessage: `You must enable this if this project contains a substantial amount of AI-generated code, any
				assets that are substantially AI-generated, the project's design relies on the use of
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
		defaultMessage: 'Select what this project uses generative AI for.',
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
	noteDescription: {
		id: 'project.settings.disclosures.ai.note-description',
		defaultMessage:
			'You may optionally provide a note to explain how you use generative AI in this project.',
	},
	notePlaceholder: {
		id: 'project.settings.disclosures.ai.note-placeholder',
		defaultMessage: 'e.g. The Chinese and Arabic translations are AI-generated.',
	},
})

const advertisingDisclosure = ref(false)
const advertisingDisclosureMessages = defineMessages({
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
	noteDescription: {
		id: 'project.settings.disclosures.advertising.note-description',
		defaultMessage:
			'Please explain how your project utilizes advertising so that users can know what to expect.',
	},
	notePlaceholder: {
		id: 'project.settings.disclosures.advertising.note-placeholder',
		defaultMessage: 'e.g. "Adds the Modrinth SMP server to your server list automatically."',
	},
})

const paidFeaturesDisclosure = ref(false)
const paidFeaturesDisclosureMessages = defineMessages({
	title: {
		id: 'project.settings.disclosures.paid-features.title',
		defaultMessage: 'Contains paid features',
	},
	description: {
		id: 'project.settings.disclosures.paid-features.description',
		defaultMessage:
			'You must enable this if your project contains features that can be obtained by spending real-world money.',
	},
	featuresDescription: {
		id: 'project.settings.disclosures.paid-features.features-description',
		defaultMessage: 'What kinds of paid features does it add?',
	},
	featurePlaceholder: {
		id: 'project.settings.disclosures.paid-features.feature-placeholder',
		defaultMessage: 'e.g. “Cosmetics available as Patreon reward.”',
	},
})

const telemetryDisclosure = ref(false)
type TelemetryConsentModel = 'opt_in' | 'opt_out' | 'always_active'
const telemetryConsentModels: TelemetryConsentModel[] = ['opt_in', 'opt_out', 'always_active']
const telemetryConsentModelMessages = defineMessages({
	opt_in: {
		id: 'project.settings.disclosures.telemetry.consent-opt-in',
		defaultMessage: 'Opt-in',
	},
	opt_out: {
		id: 'project.settings.disclosures.telemetry.consent-opt-out',
		defaultMessage: 'Opt-out',
	},
	always_active: {
		id: 'project.settings.disclosures.telemetry.consent-always-active',
		defaultMessage: 'Always active',
	},
})
const telemetryConsentModel = ref<TelemetryConsentModel>(telemetryConsentModels[0])
const telemetryDisclosureMessages = defineMessages({
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
	consentOptIn: {
		id: 'project.settings.disclosures.telemetry.consent-opt-in',
		defaultMessage: 'Opt-in',
	},
	consentOptOut: {
		id: 'project.settings.disclosures.telemetry.consent-opt-out',
		defaultMessage: 'Opt-out',
	},
	consentAlwaysActive: {
		id: 'project.settings.disclosures.telemetry.consent-always-active',
		defaultMessage: 'Always active',
	},
	dataLabel: {
		id: 'project.settings.disclosures.telemetry.data-label',
		defaultMessage: 'What data is being collected?',
	},
	dataDescription: {
		id: 'project.settings.disclosures.telemetry.data-description',
		defaultMessage:
			'You can either add a privacy policy, or a list of types of data that are collected. Remember to mention if it is anonymous or contains personally identifiable information (PII).',
	},
	dataPlaceholder: {
		id: 'project.settings.disclosures.telemetry.data-placeholder',
		defaultMessage:
			'e.g. “Anonymous launch analytics to track Minecraft version and mod loader usage.”',
	},
	addPrivacyPolicy: {
		id: 'project.settings.disclosures.telemetry.add-privacy-policy',
		defaultMessage: 'Add privacy policy',
	},
	addDataType: {
		id: 'project.settings.disclosures.telemetry.add-data-type',
		defaultMessage: 'Add type of data',
	},
})
</script>

<template>
	<div>
		<h2 class="m-0 text-2xl font-semibold">
			{{ formatMessage(messages.title) }}
		</h2>
		<p class="mb-4 mt-2">
			<IntlFormatted :message-id="messages.description">
				<template #rules="{ children }">
					<nuxt-link to="/legal/rules" target="_blank" class="underline hover:text-contrast">
						<component :is="() => normalizeChildren(children)" />
					</nuxt-link>
				</template>
			</IntlFormatted>
		</p>
		<ToggleCard v-model="aiDisclosure">
			<h3 class="mb-1 mt-0 flex items-center gap-2 text-base font-semibold text-contrast">
				<SparklesIcon class="size-5 text-primary" />
				{{ formatMessage(aiDisclosureMessages.title) }}
			</h3>
			<p class="mb-2 mt-0 text-sm leading-normal">
				{{ formatMessage(aiDisclosureMessages.description) }}
			</p>
			<p class="m-0 text-sm leading-normal text-secondary">
				<IntlFormatted :message-id="aiDisclosureMessages.contentRules">
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
				<div class="flex flex-col gap-4">
					<div class="flex flex-col gap-2">
						<p class="m-0 leading-normal text-contrast">
							{{ formatMessage(aiDisclosureMessages.typesDescription) }}
						</p>
						<div class="grid gap-2 sm:grid-cols-4">
							<Checkbox v-model="aiDisclosureTypes.code">
								{{ formatMessage(aiDisclosureMessages.typeCode) }}
							</Checkbox>
							<Checkbox v-model="aiDisclosureTypes.assets">
								{{ formatMessage(aiDisclosureMessages.typeAssets) }}
							</Checkbox>
							<Checkbox v-model="aiDisclosureTypes.text">
								{{ formatMessage(aiDisclosureMessages.typeText) }}
							</Checkbox>
						</div>
					</div>
					<div class="flex flex-col gap-2">
						<label for="ai-disclosure-note" class="leading-normal text-contrast">
							{{ formatMessage(aiDisclosureMessages.noteDescription) }}
						</label>
						<StyledInput
							id="ai-disclosure-note"
							multiline
							:rows="3"
							class="max-w-[40rem]"
							:placeholder="formatMessage(aiDisclosureMessages.notePlaceholder)"
						/>
					</div>
				</div>
			</template>
		</ToggleCard>
		<ToggleCard v-model="advertisingDisclosure" class="mt-4">
			<h3 class="mb-1 mt-0 flex items-center gap-2 text-base font-semibold text-contrast">
				<MegaphoneIcon class="size-5 text-primary" />
				{{ formatMessage(advertisingDisclosureMessages.title) }}
			</h3>
			<p class="mb-2 mt-0 text-sm leading-normal">
				{{ formatMessage(advertisingDisclosureMessages.description1) }}
			</p>
			<p class="m-0 text-sm leading-normal">
				<IntlFormatted :message-id="advertisingDisclosureMessages.description2">
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
				<div class="flex flex-col gap-4">
					<div class="flex flex-col gap-2">
						<label for="advertising-disclosure-note" class="leading-normal text-contrast">
							{{ formatMessage(advertisingDisclosureMessages.noteDescription) }}
						</label>
						<StyledInput
							id="advertising-disclosure-note"
							multiline
							:rows="3"
							class="max-w-[40rem]"
							:placeholder="formatMessage(advertisingDisclosureMessages.notePlaceholder)"
						/>
					</div>
				</div>
			</template>
		</ToggleCard>
		<ToggleCard v-model="paidFeaturesDisclosure" class="mt-4">
			<h3 class="mb-1 mt-0 flex items-center gap-2 text-base font-semibold text-contrast">
				<CircleDollarSignIcon class="size-5 text-primary" />
				{{ formatMessage(paidFeaturesDisclosureMessages.title) }}
			</h3>
			<p class="m-0 text-sm leading-normal">
				{{ formatMessage(paidFeaturesDisclosureMessages.description) }}
			</p>
			<template #expanded>
				<div class="flex flex-col gap-4">
					<div class="flex flex-col gap-2">
						<p class="m-0 leading-normal text-contrast">
							{{ formatMessage(paidFeaturesDisclosureMessages.featuresDescription) }}
						</p>
						<StyledInput
							:placeholder="formatMessage(paidFeaturesDisclosureMessages.featurePlaceholder)"
						/>
					</div>
					<ButtonStyled>
						<button class="w-fit">
							<ListPlusIcon />
							{{ formatMessage(messages.addAnother) }}
						</button>
					</ButtonStyled>
				</div>
			</template>
		</ToggleCard>
		<ToggleCard v-model="telemetryDisclosure" class="mt-4">
			<h3 class="mb-1 mt-0 flex items-center gap-2 text-base font-semibold text-contrast">
				<RadioTowerIcon class="size-5 text-primary" />
				{{ formatMessage(telemetryDisclosureMessages.title) }}
			</h3>
			<p class="m-0 text-sm leading-normal">
				{{ formatMessage(telemetryDisclosureMessages.description) }}
			</p>
			<template #expanded>
				<div class="flex flex-col gap-4">
					<div class="flex flex-col gap-2">
						<p class="m-0 leading-normal text-contrast">
							{{ formatMessage(telemetryDisclosureMessages.consentDescription) }}
						</p>
						<Chips
							v-model="telemetryConsentModel"
							:items="telemetryConsentModels"
							:format-label="
								(item: TelemetryConsentModel) => formatMessage(telemetryConsentModelMessages[item])
							"
						/>
					</div>
					<div class="flex flex-col gap-2">
						<div class="flex flex-col gap-1">
							<p class="m-0 leading-normal text-contrast">
								{{ formatMessage(telemetryDisclosureMessages.dataLabel) }}
							</p>
							<p class="m-0 leading-normal text-primary">
								{{ formatMessage(telemetryDisclosureMessages.dataDescription) }}
							</p>
						</div>
						<StyledInput
							:placeholder="formatMessage(telemetryDisclosureMessages.dataPlaceholder)"
						/>
					</div>
					<div class="flex gap-2">
						<ButtonStyled>
							<button>
								<PlusIcon />
								{{ formatMessage(telemetryDisclosureMessages.addPrivacyPolicy) }}
							</button>
						</ButtonStyled>
						<ButtonStyled>
							<button>
								<ListPlusIcon />
								{{ formatMessage(telemetryDisclosureMessages.addDataType) }}
							</button>
						</ButtonStyled>
					</div>
				</div>
			</template>
		</ToggleCard>
	</div>
</template>
