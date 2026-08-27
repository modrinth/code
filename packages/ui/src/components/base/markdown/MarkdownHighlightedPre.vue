<template>
	<pre v-if="highlighted"><code :class="`hljs language-${language}`" v-html="highlighted" /></pre>
	<pre v-else><code :class="language ? `language-${language}` : undefined">{{ codeText }}</code></pre>
</template>

<script setup lang="ts">
import { getFenceCodeText, getFenceLanguage } from '@modrinth/utils'
import { hljs } from '@modrinth/utils/highlightjs'
import type { ElementNode } from 'comark'
import { computed } from 'vue'

const props = defineProps<{
	__node: ElementNode
}>()

const codeText = computed(() => getFenceCodeText(props.__node))
const language = computed(() => getFenceLanguage(props.__node))

const highlighted = computed(() => {
	if (!language.value || !hljs.getLanguage(language.value)) return undefined
	try {
		return hljs.highlight(codeText.value, { language: language.value }).value
	} catch {
		return undefined
	}
})
</script>
