<template>
	<div v-if="loaderData?.icon" v-html="loaderData.icon" class="loader-icon-wrapper" />
	<LoaderIcon v-else />
</template>

<script setup lang="ts">
import { LoaderIcon } from '@modrinth/assets'
import { computed } from 'vue'

import { useGeneratedState } from '~/composables/generated'

const props = defineProps<{
	loader: string
}>()

const tags = useGeneratedState()

// Find the loader by name (case-insensitive comparison)
const loaderData = computed(() =>
	tags.value.loaders.find((l) => l.name.toLowerCase() === props.loader.toLowerCase()),
)
</script>

<style scoped>
.loader-icon-wrapper :deep(svg) {
	width: 100%;
	height: 100%;
}
</style>
