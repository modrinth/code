<template>
	<Section :heading="formatMessage(messages.links)">
		<template #right>
			<EditButton :section="formatMessage(messages.links)" @click="editModal?.show()" />
		</template>
		<EditModal :key="project?.id" ref="editModal" />
		<div v-if="links.length" class="flex flex-col gap-3">
			<ReviewPanel
				v-for="link in links"
				:key="link.key"
				mode="anchored"
				:target="{ kind: 'link', key: link.key }"
				class="min-w-0"
			>
				<div class="flex flex-col gap-1">
					<h4 class="m-0 text-sm font-medium text-secondary">
						{{ link.label }}
					</h4>
					<span v-if="link.unavailable && !link.removed" class="text-xs text-secondary">
						{{ formatMessage(messages.linkUnavailable) }}
					</span>
					<a
						v-if="link.url"
						v-tooltip="
							link.removed
								? formatMessage(messages.linkRemoved, { link: link.url })
								: link.unavailable
									? formatMessage(messages.previousLink, { url: link.url })
									: link.url
						"
						:href="link.url"
						:aria-label="
							link.removed
								? formatMessage(messages.linkRemoved, { link: link.url })
								: link.unavailable
									? formatMessage(messages.previousLink, { url: link.url })
									: undefined
						"
						target="_blank"
						rel="noopener noreferrer"
						class="inline-flex w-fit min-w-0 max-w-full items-center gap-1 text-xs !transition-colors hover:text-contrast"
						:class="{ 'text-secondary line-through': link.unavailable }"
					>
						<component :is="link.icon" class="mt-px size-3.5 shrink-0" aria-hidden="true" />
						<span class="min-w-0 truncate">{{ link.url.replace(/^https?:\/\//, '') }}</span>
						<ExternalIcon class="mb-0.5 size-3.5 shrink-0" aria-hidden="true" />
					</a>
				</div>
			</ReviewPanel>
		</div>
		<span v-else>{{ formatMessage(messages.emptyLinks) }}</span>
	</Section>
</template>

<script setup lang="ts">
import {
	BuyMeACoffeeIcon,
	CodeIcon,
	CurrencyIcon,
	DiscordIcon,
	ExternalIcon,
	GlobeIcon,
	HeartIcon,
	IssuesIcon,
	KoFiIcon,
	OpenCollectiveIcon,
	PatreonIcon,
	PayPalIcon,
	StoreIcon,
	WikiIcon,
} from '@modrinth/assets'
import { useVIntl } from '@modrinth/ui'
import { computed, useTemplateRef } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'
import { reviewExternalUrl } from '~/providers/project-review/project-links'
import { injectReviewPanels } from '~/providers/project-review/review-panels'

import { projectReviewMessages as messages } from '../../messages'
import ReviewPanel from '../../review-panel/index.vue'
import EditButton from '../edit/button.vue'
import EditModal from '../edit/links.vue'
import Section from '../section.vue'

const { project } = injectProjectReviewPageContext()
const { previousLinks } = injectReviewPanels()
const { formatMessage } = useVIntl()
const editModal = useTemplateRef<InstanceType<typeof EditModal>>('editModal')
const linkIcons = {
	site: GlobeIcon,
	store: StoreIcon,
	source: CodeIcon,
	issues: IssuesIcon,
	discord: DiscordIcon,
	wiki: WikiIcon,
	patreon: PatreonIcon,
	bmac: BuyMeACoffeeIcon,
	paypal: PayPalIcon,
	github: HeartIcon,
	'ko-fi': KoFiIcon,
	'open-collective': OpenCollectiveIcon,
}
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

	const currentLinks = project.value?.link_urls ?? {}
	return [...new Set([...Object.keys(currentLinks), ...previousLinks.value.keys()])]
		.sort((a, b) => {
			return (
				(order.includes(a) ? order.indexOf(a) : order.length) -
				(order.includes(b) ? order.indexOf(b) : order.length)
			)
		})
		.flatMap((key) => {
			const link = currentLinks[key]
			const url = reviewExternalUrl(link?.url)
			const message = labels[key as keyof typeof labels]
			return url || previousLinks.value.has(key)
				? [
						{
							key,
							url: url ?? reviewExternalUrl(previousLinks.value.get(key)),
							removed: !link?.url,
							unavailable: !url,
							icon: linkIcons[key as keyof typeof linkIcons] ?? CurrencyIcon,
							label: message ? formatMessage(message) : (link?.platform ?? key),
						},
					]
				: []
		})
})
</script>
