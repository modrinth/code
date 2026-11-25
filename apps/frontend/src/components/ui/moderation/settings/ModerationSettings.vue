<script setup lang="ts">
import { RotateCounterClockwiseIcon, SearchIcon } from '@modrinth/assets'
import { moderationSettings } from '@modrinth/moderation'
import { Button, Combobox, IconButton, Input, Textarea, Toggle } from '@modrinth/ui'
import { computed, ref, useId } from 'vue'

import { useModerationSettings } from '~/composables/moderation'

const props = defineProps<{
	embedded?: boolean
	techReview?: boolean
	section?: 'public-pages' | 'general'
}>()

const localhostHostname = 'localhost:3000'

const search = ref('')
const settingsId = useId()
const general = moderationSettings.General
const groups = [
	{
		id: 'workflow',
		title: 'Review workflow and queue',
		settings: [
			general.AutoGoNextOnReviewOutcome,
			general.IncludeNonProcessingProjects,
			general.SendIssuesAsThreadMessage,
		],
	},
	{
		id: 'panels',
		title: 'Panels',
		settings: [general.LinkInfoPlacement],
	},
	{
		id: 'tech-review',
		title: 'Tech review',
		settings: [general.TechReviewMode, general.OpenTechReviewTabByDefault],
	},
	{
		id: 'tech-review-code',
		title: 'Code viewer',
		settings: [
			general.WrapTechReviewCode,
			general.TechReviewHighlightWords,
			general.TechReviewScrollTiming,
		],
	},
	{
		id: 'appearance',
		title: 'Appearance',
		settings: [
			general.GalleryColumns,
			general.SortDisclosuresByActiveFirst,
			general.ExpandVersionDetailsByDefault,
			general.ExpandVersionDependenciesByDefault,
			general.ExpandVersionChangelogByDefault,
			general.ShowModpackLoaderSuffix,
			general.EnableDelayedHoverMessagePreviews,
			general.PrivateMessageHighlight,
		],
	},
	{
		id: 'queue-bar',
		title: 'Queue Bar',
		settings: [general.ShowProjectReviewTimer, general.ProjectReviewTimerWarning],
	},
	{
		id: 'shortcuts',
		title: 'Editors and shortcut hints',
		settings: [
			general.HideMarkdownFormattingButtons,
			general.ShowShortcutKeybindHints,
			general.ShowToggleIssueButtonShortcutHint,
		],
	},
	{
		id: 'project-pages',
		title: 'Public pages and hosts',
		settings: [
			general.ShowCreatorStatsOnPublicPages,
			general.ShowModerationHeaderRow,
			general.ShowCopyButtonsInModerationRow,
			general.EnableProjectKeybinds,
			general.SlicerButtonInVersions,
			general.AlternativeHostname,
		],
	},
	{
		id: 'debug',
		title: 'Debug settings',
		settings: [general.ShowAllIssueControls],
	},
]

const filteredGroups = computed(() => {
	const query = search.value.trim().toLowerCase()
	return groups
		.filter(
			(group) => ['tech-review', 'tech-review-code'].includes(group.id) === !!props.techReview,
		)
		.filter((group) => {
			if (props.section === 'public-pages') return group.id === 'project-pages'
			if (props.section === 'general') return group.id !== 'project-pages'
			return true
		})
		.map((group) => ({
			...group,
			settings: group.settings.filter((setting) =>
				[group.title, setting.title, setting.description].some((text) =>
					text.toLowerCase().includes(query),
				),
			),
		}))
		.filter((group) => group.settings.length)
})

function defaultLabel(setting: (typeof groups)[number]['settings'][number]) {
	if (setting.type === 'toggle') return setting.default ? 'On' : 'Off'
	if (setting.type === 'number')
		return `${setting.default}${setting.unit ? ` ${setting.unit}` : ''}`
	if (setting.type === 'enum')
		return (
			setting.entries.find((entry) => entry.value === setting.default)?.label ?? setting.default
		)
	return setting.default || 'None'
}

const configuredSettings = useModerationSettings()

type ModerationSetting = (typeof groups)[number]['settings'][number]

function isSettingDisabled(setting: ModerationSetting) {
	return (
		setting.id === general.ShowCopyButtonsInModerationRow.id &&
		!configuredSettings.value.get(general.ShowModerationHeaderRow)
	)
}

function isSettingDefault(setting: ModerationSetting) {
	return configuredSettings.value.get(setting) === setting.default
}

function resetSetting(setting: ModerationSetting) {
	configuredSettings.value.set(setting, setting.default)
}

function controlGroupClass(setting: ModerationSetting) {
	if (setting.type === 'enum') return 'w-full sm:w-1/3'
	if (setting.type === 'string' && setting.id !== general.TechReviewHighlightWords.id) {
		return 'w-full sm:w-1/3'
	}
	if (setting.type === 'string') return 'w-full'
	return 'shrink-0'
}
</script>

<template>
	<h2 class="m-0 mb-3 mt-6 text-2xl font-semibold first:mt-0">
		{{
			techReview
				? 'Tech review settings'
				: section === 'public-pages'
					? 'Public pages and hosts'
					: 'General settings'
		}}
	</h2>
	<div :class="{ 'universal-card !m-0': !embedded }">
		<Input
			v-if="section !== 'public-pages'"
			v-model="search"
			type="search"
			:icon="SearchIcon"
			placeholder="Search settings…"
			aria-label="Search settings…"
			class="w-full"
		/>
		<p v-if="!filteredGroups.length" class="text-secondary" role="status">
			No settings match your search.
		</p>
		<div class="mt-4 flex flex-col gap-8">
			<section
				v-for="group in filteredGroups"
				:key="group.id"
				:aria-labelledby="section === 'public-pages' ? undefined : `${settingsId}-${group.id}`"
			>
				<h3
					v-if="section !== 'public-pages'"
					:id="`${settingsId}-${group.id}`"
					class="m-0 mb-4 text-xl font-semibold"
				>
					{{ group.title }}
				</h3>
				<div class="flex flex-col gap-6">
					<div
						v-for="setting in group.settings"
						:key="setting.id"
						class="flex flex-row flex-wrap items-center justify-between gap-2"
					>
						<div class="min-w-0 flex-1">
							<span class="mb-1 block font-medium text-contrast">{{ setting.title }}</span>
							<span class="block text-secondary">{{ setting.description }}</span>
							<span class="mt-1 block text-xs text-secondary">
								Default:
								<span
									class="font-medium"
									:class="{
										'text-green': setting.type === 'toggle' && setting.default,
										'text-red': setting.type === 'toggle' && !setting.default,
										'italic opacity-50': setting.default === '',
									}"
									>{{ defaultLabel(setting) }}</span
								>
							</span>
						</div>

						<div class="flex items-center gap-2" :class="controlGroupClass(setting)">
							<IconButton
								v-tooltip="isSettingDefault(setting) ? undefined : 'Reset to default'"
								type="outlined"
								size="sm"
								class="shrink-0"
								:class="{ invisible: isSettingDefault(setting) }"
								:aria-hidden="isSettingDefault(setting) || undefined"
								label="Reset to default"
								@click="resetSetting(setting)"
							>
								<RotateCounterClockwiseIcon />
							</IconButton>
							<Toggle
								v-if="setting.type === 'toggle'"
								:aria-label="setting.title"
								:model-value="configuredSettings.get(setting)"
								:disabled="isSettingDisabled(setting)"
								class="shrink-0"
								@update:model-value="(value) => configuredSettings.set(setting, value)"
							/>
							<Input
								v-if="setting.type === 'number'"
								type="number"
								clamp
								class="w-28 shrink-0"
								:aria-label="setting.title"
								:model-value="configuredSettings.get(setting)"
								:min="setting.min"
								:max="setting.max"
								:step="setting.step"
								@update:model-value="
									(value) =>
										configuredSettings.set(setting, value === undefined ? undefined : Number(value))
								"
							>
								<template #trailing
									><span class="text-secondary">{{ setting.unit }}</span></template
								>
							</Input>
							<Combobox
								v-if="setting.type === 'enum'"
								:aria-label="setting.title"
								:model-value="configuredSettings.get(setting)"
								:options="
									setting.entries.map((entry) => ({
										value: entry.value,
										label: entry.label,
									}))
								"
								class="min-w-0 flex-1"
								@update:model-value="(value) => configuredSettings.set(setting, value)"
							/>
							<div
								v-if="setting.type === 'string'"
								class="flex min-w-0 flex-1 flex-col items-start gap-2"
							>
								<Textarea
									v-if="setting.id === general.TechReviewHighlightWords.id"
									:aria-label="setting.title"
									:model-value="configuredSettings.get(setting)"
									:rows="5"
									:spellcheck="false"
									resize="vertical"
									wrapper-class="w-full"
									placeholder="Enter words separated by commas or newlines"
									@update:model-value="(value) => configuredSettings.set(setting, value ?? '')"
								/>
								<Input
									v-else
									type="text"
									:aria-label="setting.title"
									:model-value="configuredSettings.get(setting) ?? ''"
									@update:model-value="
										(value) => configuredSettings.set(setting, String(value ?? ''))
									"
								/>
								<Button
									v-if="setting.id === 'alternative-hostname'"
									type="outlined"
									size="sm"
									class="shrink-0"
									:disabled="configuredSettings.get(setting) === localhostHostname"
									@click="configuredSettings.set(setting, localhostHostname)"
								>
									Set to {{ localhostHostname }}
								</Button>
							</div>
						</div>
					</div>
				</div>
			</section>
		</div>
	</div>
</template>
