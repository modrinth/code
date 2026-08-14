<script setup lang="ts">
import { renderBasicInlineMarkdown } from '@modrinth/utils'
import { computed } from 'vue'

const props = withDefaults(
	defineProps<{
		text: string
		target?: string
	}>(),
	{
		target: '_blank',
	},
)

const html = computed(() =>
	renderBasicInlineMarkdown(props.text, {
		target: props.target,
	}),
)
</script>

<template>
	<span class="basic-markdown-text" v-html="html" />
</template>

<style scoped>
.basic-markdown-text {
	display: block;
	max-width: 100%;
	overflow-wrap: break-word;
}

.basic-markdown-text :deep(a) {
	color: var(--color-link);
}

.basic-markdown-text :deep(a:hover) {
	text-decoration: underline;
}

.basic-markdown-text :deep(code) {
	background-color: var(--surface-4);
	font-size: 12px;
	padding: 2px 4px;
	border-radius: 4px;
}
</style>
