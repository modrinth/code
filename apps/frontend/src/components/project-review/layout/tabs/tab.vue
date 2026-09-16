<template>
	<span
		class="project-review-tab box-border flex h-full w-full items-center whitespace-nowrap rounded-lg border border-solid border-transparent px-2.5 text-[13px] font-semibold leading-none transition-colors duration-150 hover:text-contrast motion-reduce:transition-none"
		:class="isActive ? 'project-review-tab-active text-contrast' : 'text-secondary'"
	>
		{{ formatMessage(projectReviewMessages[params.params.tab]) }}
	</span>
</template>

<script setup lang="ts">
import { useVIntl } from '@modrinth/ui'
import type { IDockviewPanelHeaderProps } from 'dockview-vue'
import { ref, watchEffect } from 'vue'

import { projectReviewMessages } from '../../messages'
import type { ProjectReviewTab } from '../types'

const props = defineProps<{ params: IDockviewPanelHeaderProps<{ tab: ProjectReviewTab }> }>()
const { formatMessage } = useVIntl()
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

<style scoped>
.project-review-tab:not(.project-review-tab-active):hover {
	background-color: var(--surface-3);
}

.project-review-tab-active {
	background-color: var(--surface-4);
}
</style>
