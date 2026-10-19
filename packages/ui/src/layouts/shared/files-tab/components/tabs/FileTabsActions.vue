<template>
	<div
		v-if="shown"
		ref="root"
		class="flex h-full items-center gap-1"
		:class="ui.inWorkbench ? 'pr-1' : 'pr-3'"
	>
		<TeleportOverflowMenu
			v-if="overflowing"
			v-tooltip="formatMessage(messages.allTabs)"
			type="quiet"
			:label="formatMessage(messages.allTabs)"
			:options="tabOptions"
		>
			<ChevronDownIcon class="size-5" aria-hidden="true" />
		</TeleportOverflowMenu>
		<div
			v-if="!ui.inWorkbench && ui.showNewTabButton.value"
			class="flex min-w-[51px] shrink-0 justify-end"
		>
			<IconButton
				v-tooltip="formatMessage(messages.newTab)"
				type="quiet"
				:label="formatMessage(messages.newTab)"
				@click="openNewTab"
			>
				<PlusIcon />
			</IconButton>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ChevronDownIcon, HomeIcon, PlusIcon } from '@modrinth/assets'
import { useMutationObserver, useResizeObserver } from '@vueuse/core'
import type { DockviewGroupPanel } from 'dockview-vue'
import { computed, ref, shallowRef, useTemplateRef, watch } from 'vue'

import type { ButtonMenuOption } from '#ui/components/base/buttons'
import { IconButton, TeleportOverflowMenu } from '#ui/components/base/buttons'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

import { currentLocation } from '../../composables/file-tabs'
import { injectFileBrowserUI } from '../../providers/file-browser-ui'
import { fileIconFor, infoFrom } from '../../utils'

const props = defineProps<{
	params?: { group?: DockviewGroupPanel; panels?: { id: string }[] }
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	newTab: {
		id: 'files.tabs.new-tab',
		defaultMessage: 'New tab',
	},
	allTabs: {
		id: 'files.tabs.all-tabs',
		defaultMessage: 'All tabs',
	},
	home: {
		id: 'files.tabs.home',
		defaultMessage: 'Home',
	},
})

const ui = injectFileBrowserUI()

/**
 * Every open tab, so tabs scrolled out of the strip stay reachable; only offered while some are.
 * Teleported to the body, unlike dockview's own overflow list, so it isn't clipped by the files
 * viewer.
 */
const tabOptions = computed<ButtonMenuOption[]>(() =>
	ui.fileTabs.tabs.value.map((tab) => {
		const location = currentLocation(tab)
		const isHome = location.path === '/'
		return {
			id: tab.id,
			label: isHome ? formatMessage(messages.home) : location.name,
			icon: isHome ? HomeIcon : fileIconFor(location).icon,
			selected: tab.id === ui.fileTabs.activeTabId.value,
			action: () => ui.fileTabs.activateTab(tab.id),
		}
	}),
)

/**
 * Groups of a workbench may hold other panels too; the tab actions only belong to groups showing
 * file tabs, or nothing at all.
 */
const shown = computed(() => {
	const panels = props.params?.panels ?? []
	return panels.length === 0 || panels.some((panel) => ui.fileTabs.isTabPanel(panel))
})

const root = useTemplateRef<HTMLElement>('root')

/**
 * The strip of tabs these actions sit beside, within the same group's header, and the element
 * clipping it. Found a frame after mounting, once dockview has placed the actions in the header.
 */
const strip = shallowRef<HTMLElement | null>(null)
const scroller = shallowRef<HTMLElement | null>(null)

watch(
	root,
	(element) => {
		requestAnimationFrame(() => {
			strip.value =
				element
					?.closest('.dv-tabs-and-actions-container')
					?.querySelector<HTMLElement>('.dv-tabs-container') ?? null
			scroller.value = strip.value?.closest<HTMLElement>('.dv-scrollable') ?? strip.value
			measure()
		})
	},
	{ immediate: true, flush: 'post' },
)

const overflowing = ref(false)

/**
 * Whether the strip's tabs don't fit: dockview squeezes them until their names are cut short,
 * then scrolls them out of view. Either way, the tab list then reaches them.
 */
function measure() {
	const clip = scroller.value
	if (!clip || !strip.value) {
		overflowing.value = false
		return
	}
	overflowing.value =
		clip.scrollWidth > clip.clientWidth + 1 ||
		[...strip.value.querySelectorAll<HTMLElement>('.dv-tab .truncate')].some(
			(label) => label.scrollWidth > label.clientWidth + 1,
		)
}

useResizeObserver(() => [strip.value, scroller.value].filter((element) => element != null), measure)
useMutationObserver(strip, measure, { childList: true, subtree: true, characterData: true })

function openNewTab() {
	ui.fileTabs.openTab(infoFrom('/'), { group: props.params?.group })
}
</script>
