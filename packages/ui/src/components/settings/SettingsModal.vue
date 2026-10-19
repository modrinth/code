<template>
	<TabbedModal
		ref="modal"
		:header="header ?? formatMessage(commonMessages.settingsLabel)"
		:tabs="tabs"
		:width="width"
		:max-width="maxWidth"
	>
		<template #content="{ index }">
			<template v-if="shownPages[index]">
				<SettingsPanel
					v-if="shownPages[index].groups"
					:groups="shownPages[index].groups!"
					:store="shownPages[index].store ?? store"
				>
					<template #setting-actions="{ setting }">
						<slot name="setting-actions" :setting="setting" :page="shownPages[index]" />
					</template>
				</SettingsPanel>
				<component
					:is="shownPages[index].content"
					v-else-if="shownPages[index].content"
					v-bind="shownPages[index].contentProps"
				/>
			</template>
		</template>
		<template v-if="$slots.footer" #footer>
			<slot name="footer" />
		</template>
	</TabbedModal>
</template>

<script setup lang="ts">
import type { Component } from 'vue'
import { computed, useTemplateRef } from 'vue'

import { type MessageDescriptor, useVIntl } from '../../composables/i18n'
import { commonMessages } from '../../utils/common-messages'
import TabbedModal, { type Tab } from '../modal/TabbedModal.vue'
import SettingsPanel from './SettingsPanel.vue'
import type { SettingsStore } from './store'
import type { SettingDefinition, SettingGroup } from './types'

/** A page of the settings modal: groups of settings, or content of its own (e.g. keybinds). */
export interface SettingsPage {
	id: string
	name: MessageDescriptor
	icon: Component
	groups?: SettingGroup[]
	/** Where the page's settings are kept, when it isn't the modal's `store`. */
	store?: SettingsStore
	content?: Component
	/** Props given to the page's own content. */
	contentProps?: Record<string, unknown>
	shown?: boolean
}

/** Settings split into pages, one per tab, each laid out by `SettingsPanel` unless it has its own content. */
const props = withDefaults(
	defineProps<{
		pages: SettingsPage[]
		/** Where pages without their own store keep their settings. */
		store?: SettingsStore
		header?: string
		width?: string
		maxWidth?: string
	}>(),
	{ store: undefined, header: undefined, width: '60rem', maxWidth: undefined },
)

defineSlots<{
	'setting-actions'?(props: { setting: SettingDefinition; page: SettingsPage }): unknown
	footer?(): unknown
}>()

const { formatMessage } = useVIntl()
const modal = useTemplateRef<InstanceType<typeof TabbedModal>>('modal')

const shownPages = computed(() => props.pages.filter((page) => page.shown !== false))

/** The pages as the modal's tabs; their content is rendered through its `content` slot. */
const tabs = computed<Tab[]>(() =>
	shownPages.value.map((page) => ({ name: page.name, icon: page.icon })),
)

defineExpose({
	show: (event?: MouseEvent) => modal.value?.show(event),
	hide: () => modal.value?.hide(),
})
</script>
