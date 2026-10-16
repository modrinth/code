<script setup lang="ts">
import { SearchIcon } from '@modrinth/assets'
import { moderationSettings } from '@modrinth/moderation'
import {
	Button,
	Combobox,
	defineMessages,
	Input,
	IntlFormatted,
	Toggle,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

import { useModerationSettings } from '~/composables/moderation'

defineProps<{
	embedded?: boolean
}>()

const localhostHostname = 'localhost:3000'

const { formatMessage } = useVIntl()
const search = ref('')
const messages = defineMessages({
	title: { id: 'moderation.settings.title', defaultMessage: 'General settings' },
	search: { id: 'moderation.settings.search', defaultMessage: 'Search settings…' },
	noResults: {
		id: 'moderation.settings.no-results',
		defaultMessage: 'No settings match your search.',
	},
	workflow: {
		id: 'moderation.settings.workflow',
		defaultMessage: 'Review workflow and queue',
	},
	appearance: {
		id: 'moderation.settings.appearance',
		defaultMessage: 'Panels and appearance',
	},
	shortcuts: {
		id: 'moderation.settings.shortcuts',
		defaultMessage: 'Editors and shortcut hints',
	},
	projectPages: {
		id: 'moderation.settings.project-pages',
		defaultMessage: 'Public project pages and hosts',
	},
	defaultValue: {
		id: 'moderation.settings.default-value-styled',
		defaultMessage: 'Default: <value></value>',
	},
	on: { id: 'moderation.settings.on', defaultMessage: 'On' },
	off: { id: 'moderation.settings.off', defaultMessage: 'Off' },
	none: { id: 'moderation.settings.none', defaultMessage: 'None' },
	localhost: {
		id: 'moderation.settings.localhost',
		defaultMessage: 'Set to {hostname}',
	},
})
const general = moderationSettings.General
const groups = [
	{
		id: 'workflow',
		title: messages.workflow,
		settings: [
			general.SendIssuesAsThreadMessage,
			general.AutoGoNextOnReviewOutcome,
			general.IncludeNonProcessingProjects,
		],
	},
	{
		id: 'appearance',
		title: messages.appearance,
		settings: [
			general.LinkInfoPlacement,
			general.ShowFloatingPanelFieldHoverHighlight,
			general.ShowInlinePanelHoverHighlight,
			general.PrivateMessageHighlight,
		],
	},
	{
		id: 'shortcuts',
		title: messages.shortcuts,
		settings: [
			general.HideMarkdownFormattingButtons,
			general.ShowShortcutKeybindHints,
			general.ShowToggleIssueButtonShortcutHint,
		],
	},
	{
		id: 'project-pages',
		title: messages.projectPages,
		settings: [
			general.EnableProjectKeybinds,
			general.SlicerButtonInVersions,
			general.AlternativeHostname,
		],
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

function defaultLabel(setting: (typeof general)[keyof typeof general]) {
	if (setting.type === 'toggle') return formatMessage(setting.default ? messages.on : messages.off)
	if (setting.type === 'enum')
		return (
			setting.entries.find((entry) => entry.value === setting.default)?.label ?? setting.default
		)
	return setting.default || formatMessage(messages.none)
}

const configuredSettings = useModerationSettings()
</script>

<template>
	<div :class="{ 'universal-card': !embedded }">
		<h2 class="m-0 mb-2.5 text-2xl">{{ formatMessage(messages.title) }}</h2>
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
		<section v-for="group in filteredGroups" :key="group.id" class="mt-8">
			<h3 class="mb-6 text-xl font-semibold">{{ formatMessage(group.title) }}</h3>
			<div class="flex flex-col gap-6">
				<div
					v-for="setting in group.settings"
					:key="setting.id"
					class="flex flex-row flex-wrap items-center justify-between gap-2"
				>
					<label class="flex-1">
						<span class="mb-2.5 block font-semibold text-contrast">{{ setting.title }}</span>
						<span class="block text-secondary">{{ setting.description }}</span>
						<span class="mt-2.5 block text-xs text-secondary">
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
					</label>

					<Toggle
						v-if="setting.type === 'toggle'"
						:aria-label="setting.title"
						:model-value="configuredSettings.get(setting)"
						class="shrink-0"
						@update:model-value="(value) => configuredSettings.set(setting, value)"
					/>
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
						class="w-full sm:!w-1/3"
						@update:model-value="(value) => configuredSettings.set(setting, value)"
					/>
					<div
						v-if="setting.type === 'string'"
						class="flex w-full flex-col items-start gap-2 sm:!w-1/3"
					>
						<Input
							type="text"
							:aria-label="setting.title"
							:model-value="configuredSettings.get(setting) ?? ''"
							@update:model-value="(value) => configuredSettings.set(setting, String(value ?? ''))"
						/>
						<Button
							v-if="setting.id === 'alternative-hostname'"
							type="outlined"
							size="sm"
							class="shrink-0"
							:disabled="configuredSettings.get(setting) === localhostHostname"
							@click="configuredSettings.set(setting, localhostHostname)"
						>
							{{ formatMessage(messages.localhost, { hostname: localhostHostname }) }}
						</Button>
					</div>
				</div>
			</div>
		</section>
	</div>
</template>
