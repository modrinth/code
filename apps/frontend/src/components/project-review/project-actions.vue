<template>
	<div v-if="visibleTargets.length" class="flex shrink-0 flex-col gap-4 pb-4">
		<Tabs
			:value="selectedTarget"
			:tabs="tabs"
			class="max-w-full"
			@update:value="selectedTarget = $event as ActionTarget"
		/>
		<ReviewPanel
			v-if="visibleTargets.includes(selectedTarget)"
			:key="selectedTarget"
			mode="inline"
			:target="{ kind: selectedTarget }"
		/>
	</div>
</template>

<script setup lang="ts">
import { Tabs } from '@modrinth/ui'
import { computed, ref, watch } from 'vue'

import { injectReviewPanels } from '~/providers/project-review/review-panels'

import ReviewPanel from './review-panel/index.vue'

const { resolve } = injectReviewPanels()
const targets = ['re-review', 'reupload', 'rules', 'post-approval', 'status-alerts'] as const
type ActionTarget = (typeof targets)[number]
const selectedTarget = ref<ActionTarget>('re-review')
const visibleTargets = computed(() => targets.filter((kind) => resolve({ kind })))
const tabs = computed(() =>
	visibleTargets.value.map((kind) => ({
		value: kind,
		label: resolve({ kind })!.panel.title,
	})),
)

watch(
	visibleTargets,
	(visible, previous = []) => {
		if (visible.includes('re-review') && !previous.includes('re-review')) {
			selectedTarget.value = 're-review'
		} else if (!visible.includes(selectedTarget.value) && visible[0]) {
			selectedTarget.value = visible[0]
		}
	},
	{ immediate: true },
)
</script>
