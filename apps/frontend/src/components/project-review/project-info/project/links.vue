<template>
	<div v-if="links.length" class="flex flex-col gap-3.5">
		<ReviewPanel
			mode="anchored"
			v-for="link in links"
			:key="link.key"
			:target="{ kind: 'link', key: link.key }"
			:label="link.label"
			class="min-w-0"
		>
			<h4 class="m-0 mb-1 text-sm font-normal text-secondary">
				{{ link.label }}
			</h4>
			<a
				:href="link.url"
				target="_blank"
				rel="noopener noreferrer"
				class="inline-flex max-w-full items-center gap-1 break-all font-mono text-xs !transition-colors hover:text-contrast"
			>
				<span class="min-w-0">{{ link.url.replace(/^https?:\/\//, '') }}</span>
				<ExternalIcon class="mb-0.5 size-3.5 shrink-0" aria-hidden="true" />
			</a>
		</ReviewPanel>
	</div>
</template>

<script setup lang="ts">
import { ExternalIcon } from '@modrinth/assets'
import { useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'
import { reviewExternalUrl } from '~/providers/project-review/project-links'

import { projectReviewMessages as messages } from '../../messages'
import ReviewPanel from '../../review-panel/index.vue'

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
