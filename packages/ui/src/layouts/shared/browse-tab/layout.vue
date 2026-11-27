<script setup lang="ts">
import BrowseResults from './components/results/index.vue'
import SelectedProjectsFloatingBar from './components/selection/floating-bar.vue'
import BrowseToolbar from './components/toolbar.vue'
import { injectBrowseManager } from './providers/browse-manager'

defineProps<{
	dense?: boolean
	stacked?: boolean
}>()

const ctx = injectBrowseManager()
</script>

<template>
	<SelectedProjectsFloatingBar v-if="ctx.selection" :selection="ctx.selection" />
	<BrowseToolbar :stacked="stacked" />
	<BrowseResults :dense="dense" />
	<slot name="after" />
</template>
