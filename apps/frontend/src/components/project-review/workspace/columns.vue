<template>
	<SplitviewVue
		:orientation="Orientation.HORIZONTAL"
		:proportional-layout="false"
		@ready="onColumnsReady"
	/>
</template>

<script setup lang="ts">
import { Orientation, SplitviewVue } from 'dockview-vue'

import { injectProjectReviewContext } from '../context'
import ProjectReviewCenter from './center.vue'
import ProjectReviewPanel from './panel.vue'

defineOptions({ components: { ProjectReviewCenter, ProjectReviewPanel } })

const { onColumnsReady } = injectProjectReviewContext()
</script>
