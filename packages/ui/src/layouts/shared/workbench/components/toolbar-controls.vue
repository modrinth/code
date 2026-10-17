<template>
	<div class="flex items-center gap-1">
		<IconButton
			v-for="region in workbench.usedRegions.value"
			:key="region"
			v-tooltip="formatMessage(toggleMessages[region])"
			type="quiet"
			size="xs"
			:label="formatMessage(toggleMessages[region])"
			:aria-pressed="workbench.visible[region].value"
			@click="workbench.toggleRegion(region)"
		>
			<component
				:is="workbench.visible[region].value ? PanelRightCloseIcon : PanelRightOpenIcon"
				:class="iconRotation[region]"
				aria-hidden="true"
			/>
		</IconButton>
		<TeleportOverflowMenu
			v-if="workbench.mode.closePanels"
			type="quiet"
			size="xs"
			:circular="false"
			:label="formatMessage(messages.panels)"
			:tooltip="formatMessage(messages.panels)"
			:options="workbench.panelsMenuOptions.value"
		>
			<LayersIcon aria-hidden="true" />
		</TeleportOverflowMenu>
		<IconButton
			v-tooltip="formatMessage(variantMessage)"
			type="quiet"
			size="xs"
			:label="formatMessage(variantMessage)"
			@click="workbench.setVariant(nextVariant)"
		>
			<component :is="nextVariant === 'square' ? SquareIcon : PaletteIcon" aria-hidden="true" />
		</IconButton>
		<IconButton
			v-tooltip="formatMessage(messages.resetLayout)"
			type="quiet"
			size="xs"
			:label="formatMessage(messages.resetLayout)"
			@click="workbench.resetLayout()"
		>
			<LayoutTemplateIcon aria-hidden="true" />
		</IconButton>
		<IconButton
			v-tooltip="formatMessage(messages.collapseToolbar)"
			type="quiet"
			size="xs"
			:label="formatMessage(messages.collapseToolbar)"
			@click="emit('collapse')"
		>
			<ChevronUpIcon aria-hidden="true" />
		</IconButton>
	</div>
</template>

<script setup lang="ts">
import {
	ChevronUpIcon,
	LayersIcon,
	LayoutTemplateIcon,
	PaletteIcon,
	PanelRightCloseIcon,
	PanelRightOpenIcon,
	SquareIcon,
} from '@modrinth/assets'
import { computed } from 'vue'

import { IconButton, TeleportOverflowMenu } from '#ui/components/base/buttons'
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
	useRounded: {
		id: 'workbench.use-rounded',
		defaultMessage: 'Use rounded style',
	},
	useSquare: {
		id: 'workbench.use-square',
		defaultMessage: 'Use square style',
	},
	collapseToolbar: {
		id: 'workbench.collapse-toolbar',
		defaultMessage: 'Hide toolbar',
	},
})

const nextVariant = computed(() => (workbench.variant.value === 'rounded' ? 'square' : 'rounded'))
const variantMessage = computed(() =>
	nextVariant.value === 'square' ? messages.useSquare : messages.useRounded,
)

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
