<template>
	<ProjectReviewSidebarControl side="left" :params="params" />
</template>

<script setup lang="ts">
import type { IDockviewHeaderActionsProps } from 'dockview-vue'

import ProjectReviewSidebarControl from './sidebar-control.vue'

defineOptions({ inheritAttrs: false })
defineProps<{ params: IDockviewHeaderActionsProps }>()
</script>
