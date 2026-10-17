<template>
	<div v-if="shown" class="flex h-full items-center">
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
			<component :is="icon" aria-hidden="true" />
		</IconButton>
	</div>
</template>

<script setup lang="ts">
import {
	PanelLeftCloseIcon,
	PanelLeftOpenIcon,
	PanelRightCloseIcon,
	PanelRightOpenIcon,
} from '@modrinth/assets'
import type { DockviewGroupPanel } from 'dockview-vue'
import { computed } from 'vue'

import { IconButton } from '#ui/components/base/buttons'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

import { injectWorkbench } from '../../providers/workbench'

defineOptions({ inheritAttrs: false })

const props = withDefaults(
	defineProps<{
		params: { group: DockviewGroupPanel }
		side?: 'left' | 'right'
	}>(),
	{ side: 'left' },
)

const { formatMessage } = useVIntl()
const workbench = injectWorkbench()

const messages = defineMessages({
	showPanel: { id: 'workbench.corner.show-panel', defaultMessage: 'Show {name}' },
	hidePanel: { id: 'workbench.corner.hide-panel', defaultMessage: 'Hide {name}' },
	showRegion: {
		id: 'workbench.corner.show-region',
		defaultMessage: '{side, select, left {Show left panel} other {Show right panel}}',
	},
	hideRegion: {
		id: 'workbench.corner.hide-region',
		defaultMessage: '{side, select, left {Hide left panel} other {Hide right panel}}',
	},
})

/** Only the group in the center's outer corner on this side shows the toggle, once. */
const shown = computed(
	() =>
		workbench.usedRegions.value.includes(props.side) &&
		workbench.cornerGroups.value[props.side] === props.params.group.id,
)
const visible = computed(() => workbench.visible[props.side].value)
const icon = computed(() => {
	if (props.side === 'left') return visible.value ? PanelLeftCloseIcon : PanelLeftOpenIcon
	return visible.value ? PanelRightCloseIcon : PanelRightOpenIcon
})
/** Named after the region's panel when it holds just one, e.g. "Show Project details". */
const label = computed(() => {
	const held = workbench.panels.value.filter(
		(panel) => workbench.isOpen(panel.id) && workbench.regionOf(panel) === props.side,
	)
	if (held.length === 1) {
		return formatMessage(visible.value ? messages.hidePanel : messages.showPanel, {
			name: held[0].title,
		})
	}
	return formatMessage(visible.value ? messages.hideRegion : messages.showRegion, {
		side: props.side,
	})
})
</script>
