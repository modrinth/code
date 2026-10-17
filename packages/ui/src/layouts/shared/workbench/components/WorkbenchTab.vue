<template>
	<div
		:data-workbench-panel="id"
		class="group flex h-full items-center gap-1.5 text-sm"
		:class="
			rounded
				? [
						'box-border w-full justify-center whitespace-nowrap rounded-[10px] border border-solid px-2.5 font-medium transition-all active:scale-[0.97] motion-reduce:transition-none',
						isActive
							? 'border-green bg-highlight-green text-green'
							: 'border-transparent bg-transparent text-primary hover:bg-surface-4',
						{ 'ring-1 ring-inset ring-primary': isSelected },
					]
				: ['max-w-[14rem] pl-3', canClose ? 'pr-1' : 'pr-3', { 'bg-surface-4': isSelected }]
		"
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
				:class="isActive && !rounded ? 'text-brand' : rounded ? '' : 'text-secondary'"
				aria-hidden="true"
			/>
			<span
				class="truncate"
				:class="rounded ? '' : isActive ? 'font-semibold text-contrast' : 'text-secondary'"
			>
				{{ panel?.title }}
			</span>
		</template>
		<button
			v-if="canClose"
			type="button"
			class="flex size-6 shrink-0 items-center justify-center rounded-md border-none bg-transparent p-0 text-secondary opacity-0 hover:bg-surface-5 hover:text-contrast focus-visible:opacity-100 focus-visible:outline focus-visible:outline-2 focus-visible:outline-brand group-hover:opacity-100"
			:class="{ 'opacity-100': isActive }"
			:aria-label="formatMessage(messages.close, { name: panel?.title ?? id })"
			@pointerdown.stop
			@mousedown.stop
			@click.stop="close"
		>
			<XIcon class="size-4" />
		</button>
	</div>
</template>

<script setup lang="ts">
import { XIcon } from '@modrinth/assets'
import type { DockviewPanelApi } from 'dockview-vue'
import { computed, onBeforeUnmount, ref } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables/i18n'

import type { WorkbenchPanelParams } from '../composables/workbench-layout'
import { injectWorkbench } from '../providers/workbench'

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
const visibility = props.params.api.onDidVisibilityChange((event) => (isActive.value = event.isVisible))
onBeforeUnmount(() => visibility.dispose())

function close() {
	workbench.closePanel(id)
}
</script>
