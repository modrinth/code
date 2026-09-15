<template>
	<SplitviewVue
		class="h-full"
		:orientation="Orientation.VERTICAL"
		:proportional-layout="false"
		@ready="onRowsReady"
	/>
</template>

<script setup lang="ts">
import { Orientation, SplitviewVue } from 'dockview-vue'

import { injectProjectReviewContext } from '../context'
import ProjectReviewPanel from './panel.vue'
import ProjectReviewTabs from './tabs/tabs.vue'

defineOptions({
	components: { ProjectReviewPanel, ProjectReviewTabs },
	inheritAttrs: false,
})

const { onRowsReady } = injectProjectReviewContext()
</script>
