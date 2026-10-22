<script setup lang="ts">
import { PinIcon, RotateCounterClockwiseIcon, SearchIcon, XIcon } from '@modrinth/assets'
import { moderationSettings } from '@modrinth/moderation'
import {
	defineMessages,
	IconButton,
	Input,
	IntlFormatted,
	type MessageDescriptor,
	Toggle,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref, useId } from 'vue'

import ActionButton from '~/components/project-review/review-panel/controls/action-button.vue'
import Highlight from '~/components/project-review/review-panel/highlight.vue'
import { useFloatingReviewPanelAppearance } from '~/composables/floating-review-panel-appearance'
import { useModerationAppearanceTheme, useModerationSettings } from '~/composables/moderation'
import { appearanceFields } from '~/helpers/moderation-appearance'

import ModerationAppearanceThemes from './ModerationAppearanceThemes.vue'
import ModerationColorSetting from './ModerationColorSetting.vue'

defineProps<{
	embedded?: boolean
}>()

const { formatMessage } = useVIntl()
const search = ref('')
const settingsId = useId()
const messages = defineMessages({
	search: {
		id: 'moderation.settings.search',
		defaultMessage: 'Search settings…',
	},
	noResults: {
		id: 'moderation.settings.no-results',
		defaultMessage: 'No settings match your search.',
	},
	panelAppearance: {
		id: 'moderation.settings.review-panel-appearance',
		defaultMessage: 'Review panel appearance',
	},
	highlights: {
		id: 'moderation.settings.highlights-section',
		defaultMessage: 'Highlights',
	},
	floatingPanel: {
		id: 'moderation.settings.floating-panel-section',
		defaultMessage: 'Floating review panel',
	},
	issueToggles: {
		id: 'moderation.settings.issue-toggles-section',
		defaultMessage: 'Issue toggles',
	},
	controlLabel: {
		id: 'moderation.settings.control-label',
		defaultMessage: '{section}: {setting}',
	},
	defaultValue: {
		id: 'moderation.settings.default-value-styled',
		defaultMessage: 'Default: <value></value>',
	},
	on: { id: 'moderation.settings.on', defaultMessage: 'On' },
	off: { id: 'moderation.settings.off', defaultMessage: 'Off' },
	themeDefault: {
		id: 'moderation.settings.theme-default',
		defaultMessage: 'Theme orange',
	},
	themeBorder: {
		id: 'moderation.settings.theme-border',
		defaultMessage: 'Theme orange (25% opacity)',
	},
	reset: {
		id: 'moderation.settings.reset',
		defaultMessage: 'Reset to default',
	},
	highlightPreview: {
		id: 'moderation.settings.highlight-preview',
		defaultMessage: 'Review highlight preview',
	},
	issueTogglePreview: {
		id: 'moderation.settings.issue-toggle-preview',
		defaultMessage: 'Issue toggle preview',
	},
	floatingPanelPreview: {
		id: 'moderation.settings.floating-panel-preview',
		defaultMessage: 'Floating panel preview',
	},
	previewMissingInformation: {
		id: 'moderation.settings.highlight-preview.missing-information',
		defaultMessage: 'Missing information',
	},
	previewIncorrectFormatting: {
		id: 'moderation.settings.highlight-preview.incorrect-formatting',
		defaultMessage: 'Incorrect formatting',
	},
	previewNonEnglish: {
		id: 'moderation.settings.highlight-preview.non-english',
		defaultMessage: 'Non-English',
	},
})
const general = moderationSettings.General

function createPreviewIssues() {
	return [
		{ label: messages.previewMissingInformation, selected: false },
		{ label: messages.previewIncorrectFormatting, selected: true },
		{ label: messages.previewNonEnglish, selected: false },
	]
}

const previewIssues = ref({
	[general.ReviewHighlightOpacity.id]: createPreviewIssues(),
	[general.FloatingReviewPanelBorderWidth.id]: createPreviewIssues(),
	[general.IssueToggleColor.id]: createPreviewIssues(),
})
const groups = [
	{
		id: 'highlight-options',
		title: messages.highlights,
		settings: [general.ShowFloatingPanelFieldHoverHighlight, general.ShowInlinePanelHoverHighlight],
	},
	{
		id: 'highlights',
		title: messages.highlights,
		settings: [general.ReviewHighlightColor, general.ReviewHighlightOpacity],
	},
	{
		id: 'floating-review-panel',
		title: messages.floatingPanel,
		settings: [
			general.FloatingReviewPanelBorderColor,
			general.FloatingReviewPanelBackground,
			general.FloatingReviewPanelBackgroundOpacity,
			general.FloatingReviewPanelBorderWidth,
		],
	},
	{
		id: 'issue-toggles',
		title: messages.issueToggles,
		settings: [general.IssueToggleColor],
	},
]

const filteredGroups = computed(() => {
	const query = search.value.trim().toLocaleLowerCase()
	return groups
		.map((group) => ({
			...group,
			settings: group.settings.filter((setting) =>
				[formatMessage(group.title), setting.title, setting.description].some((text) =>
					text.toLocaleLowerCase().includes(query),
				),
			),
		}))
		.filter((group) => group.settings.length)
})

function controlLabel(section: MessageDescriptor, setting: string) {
	return formatMessage(messages.controlLabel, {
		section: formatMessage(section),
		setting,
	})
}

function defaultLabel(setting: (typeof groups)[number]['settings'][number]) {
	if (setting.type === 'color') {
		return formatMessage(
			setting.id === general.FloatingReviewPanelBorderColor.id
				? messages.themeBorder
				: messages.themeDefault,
		)
	}
	if (setting.type === 'toggle') return formatMessage(setting.default ? messages.on : messages.off)
	return `${setting.default}${setting.unit ?? ''}`
}

const configuredSettings = useModerationSettings()
const selectedTheme = useModerationAppearanceTheme()
function resetValue(setting: (typeof groups)[number]['settings'][number]) {
	const key = (Object.keys(appearanceFields) as (keyof typeof appearanceFields)[]).find(
		(key) => appearanceFields[key].id === setting.id,
	)
	return key ? selectedTheme.value.values[key] : setting.default
}
const { style: floatingPanelStyle } = useFloatingReviewPanelAppearance()
</script>

<template>
	<div :class="{ 'universal-card': !embedded }">
		<div class="mb-2.5 flex flex-wrap items-center justify-between gap-2">
			<h2 class="m-0 text-2xl">
				{{ formatMessage(messages.panelAppearance) }}
			</h2>
			<ModerationAppearanceThemes />
		</div>
		<Input
			v-model="search"
			type="search"
			:icon="SearchIcon"
			:placeholder="formatMessage(messages.search)"
			:aria-label="formatMessage(messages.search)"
			class="w-full"
		/>
		<p v-if="!filteredGroups.length" class="text-secondary" role="status">
			{{ formatMessage(messages.noResults) }}
		</p>
		<section
			v-for="group in filteredGroups"
			:key="group.id"
			:aria-labelledby="group.id === 'highlight-options' ? undefined : `${settingsId}-${group.id}`"
			class="mt-8"
		>
			<h3
				v-if="group.id !== 'highlight-options'"
				:id="`${settingsId}-${group.id}`"
				class="mb-6 text-xl font-semibold"
			>
				{{ formatMessage(group.title) }}
			</h3>
			<div class="flex flex-col gap-6">
				<div
					v-for="setting in group.settings"
					:key="setting.id"
					class="flex flex-row flex-wrap items-center justify-between gap-2"
				>
					<div class="min-w-0 flex-1">
						<span class="mb-2.5 block font-semibold text-contrast">{{ setting.title }}</span>
						<span class="block text-secondary">{{ setting.description }}</span>
						<span
							v-if="
								setting.id === general.ShowFloatingPanelFieldHoverHighlight.id ||
								setting.id === general.ShowInlinePanelHoverHighlight.id
							"
							class="mt-2.5 block text-xs text-secondary"
						>
							<IntlFormatted :message-id="messages.defaultValue">
								<template #value>
									<span
										class="font-medium"
										:class="{
											'text-green': setting.type === 'toggle' && setting.default,
											'text-red': setting.type === 'toggle' && !setting.default,
											'italic opacity-50': setting.default === '',
										}"
										>{{ defaultLabel(setting) }}</span
									>
								</template>
							</IntlFormatted>
						</span>
					</div>

					<ModerationColorSetting
						v-if="setting.type === 'color'"
						:label="controlLabel(group.title, setting.title)"
						:default-label="defaultLabel(setting)"
						:reset-value="String(resetValue(setting))"
						default-color="var(--color-orange)"
						:model-value="configuredSettings.get(setting)"
						@update:model-value="(value) => configuredSettings.set(setting, value)"
					/>
					<div v-if="setting.type === 'number'" class="flex shrink-0 items-center gap-2">
						<Input
							type="number"
							clamp
							class="w-24"
							:aria-label="controlLabel(group.title, setting.title)"
							:model-value="configuredSettings.get(setting)"
							:min="setting.min"
							:max="setting.max"
							:step="setting.step"
							@update:model-value="
								(value) =>
									configuredSettings.set(setting, value === undefined ? undefined : Number(value))
							"
						>
							<template #trailing>
								<span class="text-secondary">{{ setting.unit }}</span>
							</template>
						</Input>
						<IconButton
							v-tooltip="formatMessage(messages.reset)"
							type="quiet"
							size="sm"
							:circular="false"
							:label="formatMessage(messages.reset)"
							:disabled="configuredSettings.get(setting) === resetValue(setting)"
							@click="configuredSettings.set(setting, Number(resetValue(setting)))"
						>
							<RotateCounterClockwiseIcon aria-hidden="true" />
						</IconButton>
					</div>
					<div
						v-if="
							setting.id === general.ReviewHighlightOpacity.id ||
							setting.id === general.IssueToggleColor.id
						"
						class="mt-4 box-border flex w-full flex-col gap-2.5 rounded-xl bg-surface-2"
					>
						<Highlight
							active
							:enabled="setting.id === general.ReviewHighlightOpacity.id"
							layer="behind"
						>
							<p class="m-0 font-semibold text-contrast">
								{{
									formatMessage(
										setting.id === general.IssueToggleColor.id
											? messages.issueTogglePreview
											: messages.highlightPreview,
									)
								}}
							</p>
							<div class="flex flex-wrap gap-1.5">
								<ActionButton
									v-for="issue in previewIssues[setting.id]"
									:key="issue.label.id"
									v-model="issue.selected"
									:color="configuredSettings.get(general.IssueToggleColor)"
									:label="formatMessage(issue.label)"
								/>
							</div>
						</Highlight>
					</div>
					<div
						v-if="setting.id === general.FloatingReviewPanelBorderWidth.id"
						class="mt-4 box-border flex w-full flex-col gap-2 rounded-xl border-solid p-2.5 pb-3 text-sm text-primary"
						:style="floatingPanelStyle"
					>
						<div class="flex items-center justify-between gap-3">
							<p class="m-0 font-semibold text-contrast">
								{{ formatMessage(messages.floatingPanelPreview) }}
							</p>
							<div class="flex items-center gap-3 text-secondary" aria-hidden="true">
								<PinIcon class="size-4" />
								<XIcon class="size-4" />
							</div>
						</div>
						<div class="flex flex-wrap gap-1.5">
							<ActionButton
								v-for="issue in previewIssues[setting.id]"
								:key="issue.label.id"
								v-model="issue.selected"
								:color="configuredSettings.get(general.IssueToggleColor)"
								:label="formatMessage(issue.label)"
							/>
						</div>
					</div>

					<Toggle
						v-if="setting.type === 'toggle'"
						:aria-label="controlLabel(group.title, setting.title)"
						:model-value="configuredSettings.get(setting)"
						class="shrink-0"
						@update:model-value="(value) => configuredSettings.set(setting, value)"
					/>
				</div>
			</div>
		</section>
	</div>
</template>
