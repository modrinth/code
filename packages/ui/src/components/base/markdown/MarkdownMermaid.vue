<template>
	<Mermaid :content="content" :theme="resolvedTheme" :theme-dark="resolvedThemeDark" />
</template>

<script setup lang="ts">
import { Mermaid } from '@comark/vue/plugins/mermaid'
import type { ThemeNames } from 'comark/plugins/mermaid'
import type { DiagramColors } from 'beautiful-mermaid'
import { computed } from 'vue'

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

const resolvedTheme = computed(() => props.theme ?? modrinthMermaidTheme)
const resolvedThemeDark = computed(() => props.themeDark ?? modrinthMermaidTheme)
</script>
