<template>
	<TeleportOverflowMenu
		type="quiet"
		size="xs"
		:circular="false"
		:label="formatMessage(messages.layout)"
		:tooltip="formatMessage(messages.layout)"
		:options="options"
	>
		<LayoutTemplateIcon aria-hidden="true" />
	</TeleportOverflowMenu>
</template>

<script setup lang="ts">
import {
	LayoutTemplateIcon,
	PanelLeftOpenIcon,
	PanelRightOpenIcon,
	PanelTopIcon,
	UndoIcon,
} from '@modrinth/assets'
import { computed } from 'vue'

import { type ButtonMenuOption, TeleportOverflowMenu } from '#ui/components/base/buttons'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

import { injectWorkbench } from '../providers/workbench'
import type { WorkbenchRegion } from '../types'

/**
 * The layout menu: showing and hiding each region, opening and closing panels, and resetting the
 * layout to its defaults.
 */
const { formatMessage } = useVIntl()
const workbench = injectWorkbench()

const messages = defineMessages({
	layout: { id: 'workbench.layout-menu', defaultMessage: 'Layout' },
	regions: { id: 'workbench.layout-menu.regions', defaultMessage: 'Regions' },
	left: { id: 'workbench.layout-menu.left', defaultMessage: 'Left panel' },
	right: { id: 'workbench.layout-menu.right', defaultMessage: 'Right panel' },
	bottom: { id: 'workbench.layout-menu.bottom', defaultMessage: 'Bottom panel' },
	resetLayout: { id: 'workbench.reset-layout', defaultMessage: 'Reset layout' },
})

const regionIcons = {
	left: PanelLeftOpenIcon,
	right: PanelRightOpenIcon,
	bottom: PanelTopIcon,
} satisfies Record<WorkbenchRegion, unknown>

/** The icons have no bottom panel of their own, so the top one is turned over. */
const bottomIconClass = computed(() => 'rotate-180')

const options = computed<ButtonMenuOption[]>(() => [
	{ type: 'heading', id: 'regions', label: formatMessage(messages.regions) },
	...workbench.usedRegions.value.map((region) => ({
		id: `region-${region}`,
		label: formatMessage(messages[region]),
		icon: regionIcons[region],
		iconClass: region === 'bottom' ? bottomIconClass : undefined,
		selected: workbench.visible[region].value,
		remainOpen: true,
		action: () => workbench.toggleRegion(region),
	})),
	...(workbench.mode.closePanels
		? [{ type: 'divider' as const, id: 'panels-divider' }, ...workbench.panelsMenuOptions.value]
		: []),
	{ type: 'divider', id: 'reset-divider' },
	{
		id: 'reset-layout',
		label: formatMessage(messages.resetLayout),
		icon: UndoIcon,
		action: () => workbench.resetLayout(),
	},
])
</script>
