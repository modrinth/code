<template>
	<component :is="tag" v-if="html" :class="bodyClass" v-html="html" />
	<component :is="tag" v-else-if="fallback" :class="bodyClass">{{ fallback }}</component>
</template>

<script setup lang="ts">
import { computedAsync } from '@vueuse/core'
import { computed } from 'vue'

import { renderHighlightedMarkdown, renderMarkdown } from '../../utils/markdown'

const props = withDefaults(
	defineProps<{
		markdown?: string | null
		inline?: boolean
		highlight?: boolean
		fallback?: string
		bodyClass?: unknown
	}>(),
	{
		markdown: '',
		inline: false,
		highlight: false,
		fallback: '',
		bodyClass: undefined,
	},
)

const rendered = computedAsync(async () => {
	const markdown = props.markdown ?? ''
	const html = props.highlight ? await renderHighlightedMarkdown(markdown) : await renderMarkdown(markdown)

	if (!props.inline) return html

	const match = html.match(/^<p>([\s\S]*)<\/p>\s*$/)
	return match ? match[1] : html
}, '')

const html = computed(() => rendered.value)
const tag = computed(() => (props.inline ? 'span' : 'div'))
const bodyClass = computed(() => (props.inline ? props.bodyClass : ['markdown-body', props.bodyClass]))
</script>
