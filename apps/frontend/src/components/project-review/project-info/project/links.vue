<template>
	<section
		v-if="links.length"
		class="flex flex-col gap-4 rounded-xl border border-solid border-orange p-2.5"
		:aria-label="formatMessage(messages.links)"
	>
		<div v-for="link in links" :key="link.key" class="min-w-0">
			<h2 class="!mb-1">{{ link.label }}</h2>
			<a
				:href="link.url"
				target="_blank"
				rel="noopener noreferrer"
				class="break-all font-mono text-xs text-secondary hover:text-contrast"
				>{{ link.url.replace(/^https?:\/\//, '') }}</a
			>
		</div>
	</section>
</template>

<script setup lang="ts">
import { useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'
import { reviewExternalUrl } from '~/providers/project-review/project-links'

import { projectReviewMessages as messages } from '../../messages'

const { project } = injectProjectReviewPageContext()
const { formatMessage } = useVIntl()
const links = computed(() =>
	Object.entries(project.value?.link_urls ?? {})
		.sort(([a], [b]) => {
			const order = ['source', 'issues', 'discord', 'wiki']
			return (
				(order.includes(a) ? order.indexOf(a) : order.length) -
				(order.includes(b) ? order.indexOf(b) : order.length)
			)
		})
		.flatMap(([key, link]) => {
			const url = reviewExternalUrl(link.url)
			const labels = {
				source: messages.source,
				issues: messages.issues,
				discord: messages.discord,
				wiki: messages.wiki,
			}
			const message = labels[key as keyof typeof labels]
			return url
				? [
						{
							key,
							url,
							label: message ? formatMessage(message) : link.platform,
						},
					]
				: []
		}),
)
</script>
