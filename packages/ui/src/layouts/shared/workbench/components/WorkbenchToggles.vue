<template>
	<div class="flex items-center gap-1">
		<Button
			v-for="region in workbench.usedRegions.value"
			:key="region"
			v-tooltip="formatMessage(toggleMessages[region])"
			type="quiet"
			size="xs"
			:aria-label="formatMessage(toggleMessages[region])"
			:aria-pressed="workbench.visible[region].value"
			@click="workbench.toggleRegion(region)"
		>
			<component
				:is="workbench.visible[region].value ? PanelRightCloseIcon : PanelRightOpenIcon"
				:class="iconRotation[region]"
				aria-hidden="true"
			/>
		</Button>
		<Button
			v-tooltip="formatMessage(messages.panels)"
			type="quiet"
			size="xs"
			:aria-label="formatMessage(messages.panels)"
			@click="(event: MouseEvent) => workbench.openPanelsMenu(event)"
		>
			<LayersIcon aria-hidden="true" />
		</Button>
		<Button
			v-tooltip="formatMessage(messages.resetLayout)"
			type="quiet"
			size="xs"
			:aria-label="formatMessage(messages.resetLayout)"
			@click="workbench.resetLayout()"
		>
			<LayoutTemplateIcon aria-hidden="true" />
		</Button>
		<Button
			v-tooltip="formatMessage(messages.collapseToolbar)"
			type="quiet"
			size="xs"
			:aria-label="formatMessage(messages.collapseToolbar)"
			@click="emit('collapse')"
		>
			<ChevronUpIcon aria-hidden="true" />
		</Button>
	</div>
</template>

<script setup lang="ts">
import {
	ChevronUpIcon,
	LayersIcon,
	LayoutTemplateIcon,
	PanelRightCloseIcon,
	PanelRightOpenIcon,
} from '@modrinth/assets'

import { Button } from '#ui/components/base/buttons'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

import { injectWorkbench } from '../providers/workbench'
import type { WorkbenchRegion } from '../types'

const emit = defineEmits<{
	collapse: []
}>()

const { formatMessage } = useVIntl()
const workbench = injectWorkbench()

const messages = defineMessages({
	toggleLeft: {
		id: 'workbench.toggle-left',
		defaultMessage: 'Toggle left panel',
	},
	toggleRight: {
		id: 'workbench.toggle-right',
		defaultMessage: 'Toggle right panel',
	},
	toggleBottom: {
		id: 'workbench.toggle-bottom',
		defaultMessage: 'Toggle bottom panel',
	},
	panels: {
		id: 'workbench.panels',
		defaultMessage: 'Panels',
	},
	resetLayout: {
		id: 'workbench.reset-layout',
		defaultMessage: 'Reset layout',
	},
	collapseToolbar: {
		id: 'workbench.collapse-toolbar',
		defaultMessage: 'Hide toolbar',
	},
})

const toggleMessages = {
	left: messages.toggleLeft,
	right: messages.toggleRight,
	bottom: messages.toggleBottom,
} satisfies Record<WorkbenchRegion, unknown>

/** The icons point right, so they're turned to face the region they toggle. */
const iconRotation: Record<WorkbenchRegion, string> = {
	left: 'rotate-180',
	right: '',
	bottom: 'rotate-90',
}
</script>
