<script setup lang="ts">
import BrowseToolbar from './components/BrowseToolbar.vue'
import BrowseResults from './components/results/index.vue'
import SelectedProjectsFloatingBar from './components/SelectedProjectsFloatingBar.vue'
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
