<template>
	<span
		class="box-border flex h-full w-full items-center justify-center whitespace-nowrap rounded-[10px] border border-solid px-2.5 text-sm font-medium transition-all active:scale-[0.97] motion-reduce:transition-none"
		:class="
			isActive
				? 'border-green bg-highlight-green text-green'
				: 'border-transparent bg-transparent text-primary hover:bg-surface-4'
		"
	>
		<span>
			{{ formatMessage(projectReviewMessages[params.params.tab]) }}
		</span>
		<span v-if="count !== undefined" class="relative top-px ml-1 rounded text-xs tabular-nums"
			>({{ count }})</span
		>
		<KbdChip :keybind="`review-tab-${params.params.tab}`" class="relative bottom-px ml-1" />
	</span>
</template>

<script setup lang="ts">
import { useVIntl } from '@modrinth/ui'
import type { IDockviewPanelHeaderProps } from 'dockview-vue'
import { computed, ref, watchEffect } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'

import KbdChip from '../../kdb-chip.vue'
import { projectReviewMessages } from '../../messages'
import type { ProjectReviewTab } from '../types'

const props = defineProps<{
	params: IDockviewPanelHeaderProps<{ tab: ProjectReviewTab }>
}>()
const { formatMessage } = useVIntl()
const { tabCounts } = injectProjectReviewPageContext()
const count = computed(() => {
	const tab = props.params.params.tab
	return tab in tabCounts.value ? tabCounts.value[tab as keyof typeof tabCounts.value] : undefined
})
const isActive = ref(props.params.api.isVisible)

watchEffect((onCleanup) => {
	const api = props.params.api
	isActive.value = api.isVisible
	const subscription = api.onDidVisibilityChange((event) => {
		isActive.value = event.isVisible
	})
	onCleanup(() => subscription.dispose())
})
</script>
