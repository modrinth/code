<script setup lang="ts">
import { getCategoryIcon, getLoaderIcon, getTagIcon } from '@modrinth/assets'
import { computed } from 'vue'

const props = defineProps<{
	tag: string
	enforceType?: 'loader' | 'category'
}>()

const icon = computed(() =>
	props.enforceType === 'loader'
		? getLoaderIcon(props.tag)
		: props.enforceType === 'category'
			? getCategoryIcon(props.tag)
			: getTagIcon(props.tag),
)
</script>

<template>
	<component :is="icon" v-if="icon" />
</template>
