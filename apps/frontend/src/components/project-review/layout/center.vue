<template>
	<div class="flex h-full min-h-0 min-w-0 flex-col">
		<div class="min-h-0 flex-1">
			<ProjectReviewTabs />
		</div>
		<component :is="slots.footer" v-if="slots.footer" />
	</div>
</template>

<script setup lang="ts">
import { injectProjectReviewContext } from './context'
import ProjectReviewTabs from './tabs/tabs.vue'

defineOptions({ inheritAttrs: false })

const { slots } = injectProjectReviewContext()
</script>
