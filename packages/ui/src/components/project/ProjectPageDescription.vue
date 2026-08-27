<template>
	<MarkdownBody highlight :source="description ?? ''" />
</template>
<script setup lang="ts">
import MarkdownBody from '../base/MarkdownBody.vue'

withDefaults(
	defineProps<{
		description: string
	}>(),
	{},
)
</script>
