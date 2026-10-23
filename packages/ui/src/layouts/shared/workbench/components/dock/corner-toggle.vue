<template>
	<div v-if="shown" class="flex h-full items-center" :class="{ 'px-2': square }">
		<IconButton
			v-tooltip="label"
			type="quiet"
			size="sm"
			:circular="false"
			:label="label"
			:aria-expanded="visible"
			class="opacity-70 hover:opacity-100"
			@click="workbench.toggleRegion(side)"
		>
			<component :is="icon" :class="{ 'rotate-180': side === 'bottom' }" aria-hidden="true" />
		</IconButton>
	</div>
</template>

<script setup lang="ts">
import {
	PanelLeftCloseIcon,
	PanelLeftOpenIcon,
	PanelRightCloseIcon,
	PanelRightOpenIcon,
	PanelTopCloseIcon,
	PanelTopOpenIcon,
} from '@modrinth/assets'
import type { DockviewGroupPanel } from 'dockview-vue'
import { computed } from 'vue'

import { IconButton } from '#ui/components/base/buttons'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

import { injectWorkbench, injectWorkbenchDock } from '../../providers/workbench'
import type { WorkbenchRegion } from '../../types'

defineOptions({ inheritAttrs: false })

/**
 * Shows and hides (or collapses and expands, as the settings choose) a region: the left and right
 * ones from the outer corners of the center's tab bars, and the bottom one from the top-right
 * corner of its own.
 */
const props = withDefaults(
	defineProps<{
		params: { group: DockviewGroupPanel }
		side?: WorkbenchRegion
	}>(),
	{ side: 'left' },
)

const { formatMessage } = useVIntl()
const workbench = injectWorkbench()
const host = injectWorkbenchDock()

const messages = defineMessages({
	showPanel: { id: 'workbench.corner.show-panel', defaultMessage: 'Show {name}' },
	hidePanel: { id: 'workbench.corner.hide-panel', defaultMessage: 'Hide {name}' },
	expandPanel: { id: 'workbench.corner.expand-panel', defaultMessage: 'Expand {name}' },
	collapsePanel: { id: 'workbench.corner.collapse-panel', defaultMessage: 'Collapse {name}' },
	showRegion: {
		id: 'workbench.corner.show-region',
		defaultMessage:
			'{side, select, left {Show left panel} right {Show right panel} other {Show bottom panel}}',
	},
	hideRegion: {
		id: 'workbench.corner.hide-region',
		defaultMessage:
			'{side, select, left {Hide left panel} right {Hide right panel} other {Hide bottom panel}}',
	},
	expandRegion: {
		id: 'workbench.corner.expand-region',
		defaultMessage:
			'{side, select, left {Expand left panel} right {Expand right panel} other {Expand bottom panel}}',
	},
	collapseRegion: {
		id: 'workbench.corner.collapse-region',
		defaultMessage:
			'{side, select, left {Collapse left panel} right {Collapse right panel} other {Collapse bottom panel}}',
	},
})

/**
 * Only the group in the center's outer corner on this side shows the toggle, once; the bottom
 * region's shows in its own top-right group.
 */
const shown = computed(() => {
	if (!workbench.cornerToggles.value || !workbench.usedRegions.value.includes(props.side)) {
		return false
	}
	if (props.side === 'bottom') {
		return host === 'bottom' && workbench.regionCornerGroups.value.bottom === props.params.group.id
	}
	return workbench.cornerGroups.value[props.side] === props.params.group.id
})
const collapsing = computed(() => workbench.toggleAction.value === 'collapse')
/** Whether the region is shown in full, i.e. neither hidden nor collapsed. */
const visible = computed(
	() => workbench.visible[props.side].value && !workbench.collapsed[props.side].value,
)
/** Square tab bars run edge to edge, so the toggle keeps some room from the edges and the tabs. */
const square = computed(() => workbench.variant.value === 'square')
const icon = computed(() => {
	if (props.side === 'left') return visible.value ? PanelLeftCloseIcon : PanelLeftOpenIcon
	if (props.side === 'right') return visible.value ? PanelRightCloseIcon : PanelRightOpenIcon
	return visible.value ? PanelTopCloseIcon : PanelTopOpenIcon
})
/** Named after the region's panel when it holds just one, e.g. "Hide Summary". */
const label = computed(() => {
	const held = workbench.panels.value.filter(
		(panel) => workbench.isOpen(panel.id) && workbench.regionOf(panel) === props.side,
	)
	const show = collapsing.value ? messages.expandPanel : messages.showPanel
	const hide = collapsing.value ? messages.collapsePanel : messages.hidePanel
	if (held.length === 1) {
		return formatMessage(visible.value ? hide : show, { name: held[0].title })
	}
	const showRegion = collapsing.value ? messages.expandRegion : messages.showRegion
	const hideRegion = collapsing.value ? messages.collapseRegion : messages.hideRegion
	return formatMessage(visible.value ? hideRegion : showRegion, { side: props.side })
})
</script>
