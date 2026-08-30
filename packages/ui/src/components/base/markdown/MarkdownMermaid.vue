<template>
	<div
		class="mermaid"
		style="display: flex; justify-content: center; width: 100%; height: auto"
		:data-error="result.error"
		v-html="result.svg"
	/>
</template>

<script setup lang="ts">
import { renderMermaidSVG, THEMES } from 'beautiful-mermaid'
import type { DiagramColors } from 'beautiful-mermaid'
import type { ThemeNames } from 'comark/plugins/mermaid'
import { computed, onMounted, ref } from 'vue'

const modrinthMermaidTheme: DiagramColors = {
	bg: 'var(--surface-2)',
	fg: 'var(--color-contrast)',
	line: 'var(--color-secondary)',
	accent: 'var(--color-brand)',
	muted: 'var(--color-secondary)',
	surface: 'var(--color-button-bg)',
	border: 'var(--color-divider)',
}

const props = defineProps<{
	content: string
	theme?: ThemeNames
	themeDark?: ThemeNames
}>()

const isDark = ref(false)

onMounted(() => {
	const htmlEl = document.querySelector('html')
	if (!htmlEl) return
	isDark.value = htmlEl.classList.contains('dark')
	const observer = new MutationObserver(() => {
		isDark.value = htmlEl.classList.contains('dark')
	})
	observer.observe(htmlEl, { attributes: true, attributeFilter: ['class'] })
})

const resolvedTheme = computed(() => {
	const themeProp = isDark.value ? (props.themeDark ?? modrinthMermaidTheme) : (props.theme ?? modrinthMermaidTheme)
	return typeof themeProp === 'string' ? THEMES[themeProp] : themeProp
})

const result = computed(() => {
	try {
		return { svg: renderMermaidSVG(props.content, resolvedTheme.value), error: null as string | null }
	} catch (err) {
		return { svg: '', error: err instanceof Error ? err.message : 'Failed to render diagram' }
	}
})
</script>
