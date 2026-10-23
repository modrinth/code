<template>
	<div
		:data-workbench-panel="id"
		class="group flex h-full items-center gap-1 text-sm"
		:class="tabClass"
		:title="rounded ? undefined : panel?.title"
		@auxclick.prevent="(event) => event.button === 1 && canClose && close()"
		@contextmenu.prevent.stop="(event) => workbench.openPanelMenu(event, id)"
	>
		<component
			:is="customContent"
			v-if="customContent && panel"
			:panel="panel"
			:active="isActive"
			:selected="isSelected"
		/>
		<template v-else>
			<component
				:is="panel.icon"
				v-if="panel?.icon"
				class="size-4 shrink-0"
				:class="iconClass"
				aria-hidden="true"
			/>
			<span class="truncate" :class="titleClass">{{ panel?.title }}</span>
		</template>
		<button
			v-if="canClose"
			type="button"
			:aria-label="formatMessage(messages.close, { name: panel?.title ?? id })"
			class="flex size-4 shrink-0 cursor-pointer items-center justify-center rounded border-0 bg-transparent p-0 text-current opacity-0 hover:bg-surface-5 focus-visible:opacity-100 group-hover:opacity-100"
			:class="{ 'opacity-100': isActive }"
			@pointerdown.stop
			@mousedown.stop
			@click.stop="close"
		>
			<XIcon class="size-3" aria-hidden="true" />
		</button>
	</div>
</template>

<script setup lang="ts">
import { XIcon } from '@modrinth/assets'
import type { DockviewPanelApi } from 'dockview-vue'
import { computed, onBeforeUnmount, ref } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables/i18n'

import type { WorkbenchPanelParams } from '../../composables/create-workbench'
import { injectWorkbench } from '../../providers/workbench'

const props = defineProps<{
	params: {
		params: WorkbenchPanelParams
		api: DockviewPanelApi
	}
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	close: {
		id: 'workbench.tab.close',
		defaultMessage: 'Close {name}',
	},
})

const workbench = injectWorkbench()
const id = props.params.params.id

const panel = computed(() => workbench.panels.value.find((candidate) => candidate.id === id))
const rounded = computed(() => workbench.variant.value === 'rounded')
const customContent = computed(() => workbench.slots.tab)
const canClose = computed(() => workbench.mode.closePanels)
const isSelected = computed(() => workbench.selectedPanels.value.includes(id))

const isActive = ref(props.params.api.isVisible)
const visibility = props.params.api.onDidVisibilityChange(
	(event) => (isActive.value = event.isVisible),
)
onBeforeUnmount(() => visibility.dispose())

/** Rounded tabs are pills, green while shown; square ones are plain, marked by the tab bar. */
const tabClass = computed(() =>
	rounded.value
		? [
				'box-border w-full justify-center whitespace-nowrap rounded-lg border border-solid px-2 font-medium transition-all active:scale-[0.97] motion-reduce:transition-none',
				isActive.value
					? 'border-green bg-highlight-green text-green'
					: 'border-transparent bg-transparent text-primary hover:bg-surface-3',
				{ 'ring-1 ring-inset ring-primary': isSelected.value },
			]
		: [
				'max-w-[14rem] pl-2.5',
				canClose.value ? 'pr-1.5' : 'pr-2.5',
				{ 'bg-surface-3': isSelected.value },
			],
)
const iconClass = computed(() => {
	if (rounded.value) return ''
	return isActive.value ? 'text-brand' : 'text-secondary'
})
const titleClass = computed(() => {
	if (rounded.value) return ''
	return isActive.value ? 'font-semibold text-contrast' : 'text-secondary'
})

function close() {
	workbench.closePanel(id)
}
</script>
