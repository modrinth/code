<template>
	<component :is="icon" v-if="icon" />
	<LoaderIcon v-else />
</template>

<script setup lang="ts">
import { getLoaderIcon, LoaderIcon } from '@modrinth/assets'
import { computed } from 'vue'

const props = defineProps<{
	loader: string
}>()

const icon = computed(() => getLoaderIcon(props.loader))
</script>
