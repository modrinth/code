<template>
	<div class="flex flex-col gap-4">
		<div class="flex items-start gap-2">
			<div
				class="flex min-w-0 flex-1 flex-wrap gap-1"
				role="tablist"
				:aria-label="formatMessage(messages.pages)"
			>
				<Button
					v-for="page in shownPages"
					:key="page.id"
					type="quiet"
					size="sm"
					role="tab"
					:aria-selected="page.id === activePage?.id"
					:class="{ '!bg-surface-4 !text-contrast': page.id === activePage?.id }"
					@click="pageId = page.id"
				>
					<component :is="page.icon" aria-hidden="true" />
					{{ formatMessage(page.name) }}
				</Button>
			</div>
			<slot name="actions" />
		</div>
		<template v-if="activePage">
			<SettingsPanel
				v-if="activePage.groups"
				:key="activePage.id"
				:groups="activePage.groups"
				:store="activePage.store ?? store"
			>
				<template #setting-actions="{ setting }">
					<slot name="setting-actions" :setting="setting" :page="activePage" />
				</template>
			</SettingsPanel>
			<component
				:is="activePage.content"
				v-else-if="activePage.content"
				:key="activePage.id"
				v-bind="activePage.contentProps"
			/>
		</template>
		<slot name="footer" />
	</div>
</template>

<script setup lang="ts">
import { useLocalStorage } from '@vueuse/core'
import { computed, ref } from 'vue'

import { defineMessages, useVIntl } from '../../composables/i18n'
import Button from '../base/buttons/Button.vue'
import type { SettingsPage } from './SettingsModal.vue'
import SettingsPanel from './SettingsPanel.vue'
import type { SettingsStore } from './store'
import type { SettingDefinition } from './types'

/**
 * Settings split into pages, picked from a row of tabs above them, each laid out by
 * `SettingsPanel` unless it has its own content; for settings shown in a panel rather than a
 * modal. `storageKey` remembers the page last shown.
 */
const props = withDefaults(
	defineProps<{
		pages: SettingsPage[]
		/** Where pages without their own store keep their settings. */
		store?: SettingsStore
		storageKey?: string
	}>(),
	{ store: undefined, storageKey: undefined },
)

defineSlots<{
	'setting-actions'?(props: { setting: SettingDefinition; page: SettingsPage }): unknown
	/** Shown at the end of the row of pages, e.g. to move the settings elsewhere. */
	actions?(): unknown
	footer?(): unknown
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	pages: { id: 'settings.pages.label', defaultMessage: 'Settings pages' },
})

const shownPages = computed(() => props.pages.filter((page) => page.shown !== false))
const pageId = props.storageKey ? useLocalStorage(props.storageKey, '') : ref('')
const activePage = computed(
	() => shownPages.value.find((page) => page.id === pageId.value) ?? shownPages.value[0],
)
</script>
