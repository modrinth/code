<template>
	<div
		class="group flex h-full max-w-[14rem] items-center gap-1.5 pl-3 pr-1 text-sm"
		:title="panel?.title"
		@auxclick.prevent="(event) => event.button === 1 && close()"
		@contextmenu.prevent.stop="(event) => workbench.openPanelMenu(event, id)"
	>
		<component
			:is="panel.icon"
			v-if="panel?.icon"
			class="size-4 shrink-0"
			:class="isActive ? 'text-brand' : 'text-secondary'"
			aria-hidden="true"
		/>
		<span class="truncate" :class="isActive ? 'font-semibold text-contrast' : 'text-secondary'">
			{{ panel?.title }}
		</span>
		<button
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

const isActive = ref(props.params.api.isVisible)
const visibility = props.params.api.onDidVisibilityChange((event) => (isActive.value = event.isVisible))
onBeforeUnmount(() => visibility.dispose())

function close() {
	workbench.closePanel(id)
}
</script>
