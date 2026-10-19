<template>
	<div class="flex flex-col gap-6">
		<Input
			v-if="searchable"
			v-model="search"
			type="search"
			:icon="SearchIcon"
			:placeholder="formatMessage(messages.search)"
			:aria-label="formatMessage(messages.search)"
			wrapper-class="w-full"
		/>
		<p v-if="!filteredGroups.length" class="m-0 text-secondary" role="status">
			{{ formatMessage(messages.noResults) }}
		</p>
		<section
			v-for="group in filteredGroups"
			:key="group.id"
			class="flex flex-col gap-5"
			:aria-labelledby="`${baseId}-${group.id}`"
		>
			<div class="flex flex-col gap-1">
				<h3 :id="`${baseId}-${group.id}`" class="m-0 text-lg font-semibold text-contrast">
					{{ text(group.title) }}
				</h3>
				<p v-if="group.description" class="m-0 text-sm text-secondary">
					{{ text(group.description) }}
				</p>
			</div>
			<div
				v-for="setting in group.settings"
				:key="setting.id"
				class="flex flex-row flex-wrap items-center justify-between gap-x-4 gap-y-2"
			>
				<div class="flex min-w-[12rem] flex-1 flex-col gap-1">
					<span :id="`${baseId}-${setting.id}`" class="font-semibold text-contrast">
						{{ text(setting.title) }}
					</span>
					<span v-if="setting.description" class="text-sm text-secondary">
						{{ text(setting.description) }}
					</span>
					<span v-if="showDefaults" class="text-xs text-secondary">
						<IntlFormatted :message-id="messages.defaultValue">
							<template #value>
								<span
									class="font-medium"
									:class="{
										'text-green': setting.type === 'toggle' && setting.default,
										'text-red': setting.type === 'toggle' && !setting.default,
									}"
									>{{ defaultLabel(setting) }}</span
								>
							</template>
						</IntlFormatted>
					</span>
				</div>

				<Toggle
					v-if="setting.type === 'toggle'"
					:aria-labelledby="`${baseId}-${setting.id}`"
					:model-value="storeOf(group).get(setting)"
					class="shrink-0"
					@update:model-value="(value) => storeOf(group).set(setting, !!value)"
				/>
				<Chips
					v-else-if="setting.type === 'enum' && setting.display === 'chips'"
					:aria-label="text(setting.title)"
					:items="setting.entries.map((entry) => entry.value)"
					:format-label="(value) => entryLabel(setting, value)"
					:model-value="storeOf(group).get(setting)"
					size="small"
					hide-checkmark-icon
					class="shrink-0"
					@update:model-value="(value) => value != null && storeOf(group).set(setting, value)"
				/>
				<Combobox
					v-else-if="setting.type === 'enum'"
					:aria-label="text(setting.title)"
					:model-value="storeOf(group).get(setting)"
					:options="
						setting.entries.map((entry) => ({ value: entry.value, label: text(entry.label) }))
					"
					class="w-full sm:!w-1/3"
					@update:model-value="(value) => value != null && storeOf(group).set(setting, value)"
				/>
				<div v-else-if="setting.type === 'string'" class="flex w-full flex-col gap-2 sm:!w-1/3">
					<Input
						type="text"
						:aria-labelledby="`${baseId}-${setting.id}`"
						:placeholder="setting.placeholder ? text(setting.placeholder) : undefined"
						:model-value="storeOf(group).get(setting)"
						@update:model-value="(value) => storeOf(group).set(setting, String(value ?? ''))"
					/>
				</div>
				<slot name="setting-actions" :setting="setting" :store="storeOf(group)" />
			</div>
		</section>
	</div>
</template>

<script setup lang="ts">
import { SearchIcon } from '@modrinth/assets'
import { computed, ref, useId } from 'vue'

import { defineMessages, useVIntl } from '../../composables/i18n'
import Chips from '../base/Chips.vue'
import Combobox from '../base/Combobox.vue'
import Input from '../base/inputs/Input.vue'
import IntlFormatted from '../base/IntlFormatted.vue'
import Toggle from '../base/Toggle.vue'
import type { SettingsStore } from './store'
import type { EnumSettingDefinition, SettingDefinition, SettingGroup, SettingText } from './types'

/**
 * Settings laid out by group, each with its title, description, default and the control for its
 * kind, read from and written to `store`. Searching filters them by group, title and description.
 */
const props = withDefaults(
	defineProps<{
		groups: SettingGroup[]
		/** Where groups without their own store keep their settings. */
		store?: SettingsStore
		searchable?: boolean
		showDefaults?: boolean
	}>(),
	{ store: undefined, searchable: true, showDefaults: true },
)

defineSlots<{
	/** Shown after a setting's control, e.g. to fill in a commonly used value. */
	'setting-actions'?(props: { setting: SettingDefinition; store: SettingsStore }): unknown
}>()

const { formatMessage } = useVIntl()
const baseId = useId()
const search = ref('')

const messages = defineMessages({
	search: { id: 'settings.panel.search', defaultMessage: 'Search settings...' },
	noResults: { id: 'settings.panel.no-results', defaultMessage: 'No settings match your search.' },
	defaultValue: { id: 'settings.panel.default-value', defaultMessage: 'Default: <value></value>' },
	on: { id: 'settings.panel.on', defaultMessage: 'On' },
	off: { id: 'settings.panel.off', defaultMessage: 'Off' },
	none: { id: 'settings.panel.none', defaultMessage: 'None' },
})

const noStore: SettingsStore = {
	get: (definition) => definition.default,
	set: () => {},
}

function storeOf(group: SettingGroup) {
	return group.store ?? props.store ?? noStore
}

function text(value: SettingText) {
	return typeof value === 'string' ? value : formatMessage(value)
}

function entryLabel(setting: EnumSettingDefinition, value: string) {
	const entry = setting.entries.find((candidate) => candidate.value === value)
	return entry ? text(entry.label) : value
}

function defaultLabel(setting: SettingDefinition) {
	if (setting.type === 'toggle') return formatMessage(setting.default ? messages.on : messages.off)
	if (setting.type === 'enum') return entryLabel(setting, setting.default)
	return setting.default || formatMessage(messages.none)
}

const filteredGroups = computed(() => {
	const query = search.value.trim().toLocaleLowerCase()
	if (!query) return props.groups
	return props.groups
		.map((group) => {
			const groupMatches = text(group.title).toLocaleLowerCase().includes(query)
			return {
				...group,
				settings: group.settings.filter(
					(setting) =>
						groupMatches ||
						[setting.title, setting.description]
							.filter((value): value is SettingText => value != null)
							.some((value) => text(value).toLocaleLowerCase().includes(query)),
				),
			}
		})
		.filter((group) => group.settings.length)
})
</script>
