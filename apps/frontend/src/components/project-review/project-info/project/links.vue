<template>
	<div v-if="links.length" class="-mt-1.5 flex flex-col">
		<ReviewPanel
			v-for="link in links"
			:key="link.key"
			mode="anchored"
			:target="{ kind: 'link', key: link.key }"
			class="min-w-0"
		>
			<div class="flex flex-col gap-1 py-1.5">
				<h4 class="m-0 text-sm font-normal text-secondary">
					{{ link.label }}
				</h4>
				<a
					v-tooltip="link.url"
					:href="link.url"
					target="_blank"
					rel="noopener noreferrer"
					class="inline-flex w-fit min-w-0 max-w-full items-center gap-1 font-mono text-xs !transition-colors hover:text-contrast"
				>
					<span class="min-w-0 truncate">{{ link.url.replace(/^https?:\/\//, '') }}</span>
					<ExternalIcon class="mb-0.5 size-3.5 shrink-0" aria-hidden="true" />
				</a>
			</div>
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
const links = computed(() => {
	const isServerProject = project.value?.minecraft_server != null
	const order = isServerProject
		? ['site', 'store', 'wiki', 'discord']
		: [
				'issues',
				'source',
				'wiki',
				'discord',
				'patreon',
				'bmac',
				'paypal',
				'github',
				'ko-fi',
				'other',
			]
	const labels = {
		site: messages.website,
		store: messages.store,
		source: messages.source,
		issues: messages.issues,
		discord: isServerProject ? messages.serverDiscord : messages.discord,
		wiki: messages.wiki,
		patreon: messages.donationPatreon,
		bmac: messages.donationBmac,
		paypal: messages.donationPaypal,
		github: messages.donationGithub,
		'ko-fi': messages.donationKoFi,
		other: messages.donationOther,
	}

	return Object.entries(project.value?.link_urls ?? {})
		.sort(([a], [b]) => {
			return (
				(order.includes(a) ? order.indexOf(a) : order.length) -
				(order.includes(b) ? order.indexOf(b) : order.length)
			)
		})
		.flatMap(([key, link]) => {
			const url = reviewExternalUrl(link.url)
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
		})
})
</script>
