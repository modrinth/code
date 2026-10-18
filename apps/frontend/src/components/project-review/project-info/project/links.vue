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
					<h4 class="m-0 flex items-center gap-2 text-sm font-medium text-secondary">
						{{ link.label }}
						<TagItem
							v-if="link.key === 'source' && project && licenseRequiresSource(project.license.id)"
							v-tooltip="formatMessage(messages.sourceAvailabilityRequired)"
							class="!border-orange !bg-surface-4 !px-1.5 !py-0.5 !text-orange"
						>
							<TagCategoryScrollTextIcon />
						</TagItem>
					</h4>
					<span v-if="link.unavailable && !link.removed" class="text-xs text-secondary">
						{{ formatMessage(messages.linkUnavailable) }}
					</span>
					<div v-if="link.url" class="flex min-w-0 items-center gap-1.5">
						<LinkInfoTooltip
							v-slot="{ tooltipId }"
							:url="link.url"
							:validation-messages="link.validationMessages"
							@hover="setLinkHovered(link.key, $event)"
						>
							<a
								:href="link.url"
								:aria-describedby="tooltipId"
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
						</LinkInfoTooltip>
						<TriangleAlertIcon
							v-if="link.warningText"
							v-tooltip="link.warningText"
							:aria-label="link.warningText"
							class="size-4 shrink-0 text-orange"
						/>
					</div>
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
	TagCategoryScrollTextIcon,
	TriangleAlertIcon,
	WikiIcon,
} from '@modrinth/assets'
import { licenseRequiresSource } from '@modrinth/moderation'
import { injectModrinthClient, TagItem, useVIntl } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed, ref, useTemplateRef, watch } from 'vue'

import { getProjectNagMessages } from '~/composables/project-nag-validation'
import { projectQueryOptions, STALE_TIME } from '~/composables/queries/project'
import { injectProjectReviewPageContext } from '~/providers/project-review'
import { reviewExternalUrl } from '~/providers/project-review/project-links'
import { injectReviewPanels } from '~/providers/project-review/review-panels'

import LinkInfoTooltip from '../../link-info/tooltip.vue'
import { projectReviewMessages as messages } from '../../messages'
import ReviewPanel from '../../review-panel/index.vue'
import EditButton from '../edit/button.vue'
import EditModal from '../edit/links.vue'
import Section from '../section.vue'

const { project, projectId, projectV2 } = injectProjectReviewPageContext()
const { previousLinks } = injectReviewPanels()
const { formatMessage } = useVIntl()
const client = injectModrinthClient()
const hoveredLink = ref<string>()
const validation = useQuery(
	computed(() => ({
		...projectQueryOptions.validation(projectId.value, client),
		enabled: import.meta.client && !!projectId.value && hoveredLink.value !== undefined,
		staleTime: STALE_TIME,
		retry: false,
		refetchOnWindowFocus: false,
		refetchOnReconnect: false,
	})),
)

function setLinkHovered(key: string, hovered: boolean) {
	if (hovered) hoveredLink.value = key
	else if (hoveredLink.value === key) hoveredLink.value = undefined
}

watch(
	projectId,
	() => {
		hoveredLink.value = undefined
	},
	{ flush: 'sync' },
)

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
			const validationMessages =
				url && !validation.isError.value
					? getProjectNagMessages(validation.data.value?.nags ?? [], 'link-field', {
							detailField: key,
							url,
							projectType: projectV2.value?.project_type,
						})
					: []
			return url || previousLinks.value.has(key)
				? [
						{
							key,
							validationMessages,
							warningText: validationMessages
								.map((message) => formatMessage(message.message, message.values))
								.join('\n'),
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
