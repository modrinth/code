<template>
	<div v-if="options.length" class="flex h-full items-center pr-2.5">
		<TeleportOverflowMenu
			v-tooltip="formatMessage(messages.add)"
			type="quiet"
			:label="formatMessage(messages.add)"
			:options="options"
			@open="version++"
		>
			<PlusIcon aria-hidden="true" />
		</TeleportOverflowMenu>
	</div>
</template>

<script setup lang="ts">
import { PlusIcon } from '@modrinth/assets'
import type { DockviewGroupPanel } from 'dockview-vue'
import { computed, ref } from 'vue'

import { type ButtonMenuOption, TeleportOverflowMenu } from '#ui/components/base/buttons'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

import { injectWorkbench, injectWorkbenchDock } from '../../providers/workbench'

/**
 * Adds to a group from its tab bar: a new document (e.g. a files tab at the home folder), or any
 * of the workbench's panels, brought into this group from wherever they are (or reopened).
 */
const props = defineProps<{
	group: DockviewGroupPanel
}>()

const { formatMessage } = useVIntl()
const workbench = injectWorkbench()
const host = injectWorkbenchDock()

const messages = defineMessages({
	add: { id: 'workbench.add-menu', defaultMessage: 'Add a tab' },
	panels: { id: 'workbench.add-menu.panels', defaultMessage: 'Panels' },
})

/** Bumped as the menu opens, so it shows which panels the group holds right then. */
const version = ref(0)

const options = computed<ButtonMenuOption[]>(() => {
	void version.value
	const result: ButtonMenuOption[] = []
	const create = workbench.documents?.create
	if (create && (create.shown?.() ?? true)) {
		result.push({
			id: 'new-document',
			label: create.label(),
			icon: create.icon,
			action: () => create.open(props.group),
		})
	}
	if (workbench.mode.movePanels && workbench.panels.value.length) {
		if (result.length) result.push({ type: 'divider', id: 'panels-divider' })
		result.push({ type: 'heading', id: 'panels', label: formatMessage(messages.panels) })
		for (const panel of workbench.panels.value) {
			result.push({
				id: `panel-${panel.id}`,
				label: panel.title,
				icon: panel.icon,
				selected: props.group.panels.some((entry) => entry.id === panel.id),
				action: () =>
					workbench.movePanel(panel.id, host, { referenceGroup: props.group, direction: 'within' }),
			})
		}
	}
	return result
})
</script>
