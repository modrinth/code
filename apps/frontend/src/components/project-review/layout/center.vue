<template>
	<div class="flex h-full min-h-0 min-w-0 flex-col">
		<div class="min-h-0 flex-1">
			<SplitviewVue
				class="project-review-rows h-full"
				:orientation="Orientation.VERTICAL"
				:proportional-layout="false"
				@ready="onRowsReady"
			/>
		</div>
		<component :is="slots.footer" v-if="slots.footer" />
	</div>
</template>

<script setup lang="ts">
import { Orientation, SplitviewVue } from 'dockview-vue'

import { injectProjectReviewContext } from './context'
import ProjectReviewPanel from './panel.vue'
import ProjectReviewTabs from './tabs/tabs.vue'

defineOptions({
	components: { ProjectReviewPanel, ProjectReviewTabs },
	inheritAttrs: false,
})

const { onRowsReady, slots } = injectProjectReviewContext()
</script>
