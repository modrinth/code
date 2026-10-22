<template>
	<div
		class="workbench-collapsed-strip flex shrink-0 items-center gap-1 overflow-hidden p-1"
		:class="vertical ? 'h-full w-full flex-col' : 'h-full w-full flex-row'"
		role="toolbar"
		:aria-label="formatMessage(messages.label)"
		:aria-orientation="vertical ? 'vertical' : 'horizontal'"
	>
		<IconButton
			v-for="tab in tabs"
			:key="tab.id"
			v-tooltip="tab.title"
			type="quiet"
			size="sm"
			:circular="false"
			:label="tab.title"
			class="shrink-0"
			:class="tab.active ? 'text-brand' : 'text-secondary'"
			:aria-pressed="tab.active"
			@click="workbench.activateTab(region, tab.id)"
		>
			<component :is="tab.icon ?? FileIcon" aria-hidden="true" />
		</IconButton>
		<IconButton
			v-if="!vertical"
			v-tooltip="expandLabel"
			type="quiet"
			size="sm"
			:circular="false"
			:label="expandLabel"
			class="ml-auto shrink-0 opacity-70 hover:opacity-100"
			@click="workbench.setRegionCollapsed(region, false)"
		>
			<PanelTopOpenIcon class="rotate-180" aria-hidden="true" />
		</IconButton>
	</div>
</template>

<script setup lang="ts">
import { FileIcon, PanelTopOpenIcon } from '@modrinth/assets'
import { computed } from 'vue'

import { IconButton } from '#ui/components/base/buttons'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

import { injectWorkbench } from '../../providers/workbench'
import type { WorkbenchRegion } from '../../types'

/**
 * A collapsed region: a strip of its tabs' icons, any of which brings the region back with that
 * tab in front. The side regions' toggles sit in the center's corners; the bottom region's keeps
 * its place at the strip's end.
 */
const props = defineProps<{ region: WorkbenchRegion }>()

const { formatMessage } = useVIntl()
const workbench = injectWorkbench()

const messages = defineMessages({
	label: { id: 'workbench.collapsed.label', defaultMessage: 'Collapsed panels' },
	expand: { id: 'workbench.collapsed.expand', defaultMessage: 'Expand bottom panel' },
})

const vertical = computed(() => props.region !== 'bottom')
const tabs = computed(() => workbench.dockTabs(props.region))
const expandLabel = computed(() => formatMessage(messages.expand))
</script>
