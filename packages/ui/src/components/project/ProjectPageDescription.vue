<template>
	<MarkdownBody :markdown="description ?? ''" highlight />
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
