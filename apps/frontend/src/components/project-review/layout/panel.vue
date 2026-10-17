<template>
	<component :is="content" v-if="content" />
	<template v-else>
		<h2 class="m-0 text-lg font-semibold text-contrast">
			{{ reviewPanelLabels[slotName] }}
		</h2>
		<p class="m-0 text-secondary">No project selected</p>
	</template>
</template>

<script setup lang="ts">
import { injectWorkbenchPanel } from '@modrinth/ui'
import { computed } from 'vue'

import { injectProjectReviewContext, provideReviewPanelVisible, provideReviewSlot } from './context'
import { reviewPanelLabels } from './labels'
import type { ProjectReviewSlot } from './types'

/** A review slot's content within its workbench panel, which tells it where and whether it's shown. */
const props = defineProps<{ slotName: ProjectReviewSlot }>()

const { slots } = injectProjectReviewContext()
const panel = injectWorkbenchPanel()

provideReviewSlot(computed(() => props.slotName))
provideReviewPanelVisible(panel.visible)

const content = computed(() => slots[props.slotName])
</script>
