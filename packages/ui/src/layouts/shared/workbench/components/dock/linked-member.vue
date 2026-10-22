<template>
	<section
		class="workbench-panel"
		:class="[member.scroll === false ? '' : 'overflow-auto', { 'p-3': member.padded }]"
		:aria-label="member.title"
	>
		<SlotOutlet v-if="workbench.slots[slotName]" :slots="workbench.slots" :name="slotName" />
	</section>
</template>

<script setup lang="ts">
import {
	injectWorkbench,
	injectWorkbenchPanel,
	provideWorkbenchPanel,
} from '../../providers/workbench'
import type { WorkbenchPanel } from '../../types'
import { SlotOutlet } from '../slot-outlet'

/**
 * One panel of a linked tab, shown as its own panel would show it: within the tab's panel, it's
 * shown whenever the tab is.
 */
const props = defineProps<{ member: WorkbenchPanel }>()

const workbench = injectWorkbench()
const tab = injectWorkbenchPanel()
const slotName = `panel-${props.member.id}` as const

provideWorkbenchPanel({ id: props.member.id, visible: tab.visible, window: tab.window })
</script>
