<script setup lang="ts">
import { type Component, computed } from 'vue'

import PrideCollectionWidget from './PrideCollectionWidget.vue'
import SparkLiveWidget from './SparkLiveWidget.vue'
import SparkLiveWidgetEmbed from './SparkLiveWidgetEmbed.vue'

const ARTICLE_WIDGETS: Record<string, Component> = {
	'spark-live-widget': SparkLiveWidget,
	'spark-live-widget-embed': SparkLiveWidgetEmbed,
	'pride-collection-widget': PrideCollectionWidget,
}

type ArticleBodyPart = { type: 'html'; content: string } | { type: 'widget'; id: string }

function parseArticleHtml(html: string): ArticleBodyPart[] {
	const widgetIds = Object.keys(ARTICLE_WIDGETS)
	if (widgetIds.length === 0) {
		return [{ type: 'html', content: html }]
	}

	const pattern = new RegExp(`<div id="(${widgetIds.join('|')})"></div>`, 'g')
	const parts: ArticleBodyPart[] = []
	let lastIndex = 0
	let match: RegExpExecArray | null

	while ((match = pattern.exec(html)) !== null) {
		if (match.index > lastIndex) {
			parts.push({ type: 'html', content: html.slice(lastIndex, match.index) })
		}
		parts.push({ type: 'widget', id: match[1] })
		lastIndex = pattern.lastIndex
	}

	if (lastIndex < html.length) {
		parts.push({ type: 'html', content: html.slice(lastIndex) })
	}

	return parts.length > 0 ? parts : [{ type: 'html', content: html }]
}

const props = defineProps<{
	html: string
}>()

const parts = computed(() => parseArticleHtml(props.html))
</script>

<template>
	<div
		v-if="parts.length === 1 && parts[0].type === 'html'"
		class="markdown-body"
		v-html="parts[0]?.content"
	/>
	<div v-else class="flex flex-col gap-4">
		<template v-for="(part, index) in parts" :key="index">
			<div v-if="part.type === 'html'" class="markdown-body" v-html="part.content" />
			<component :is="ARTICLE_WIDGETS[part.id]" v-else />
		</template>
	</div>
</template>
