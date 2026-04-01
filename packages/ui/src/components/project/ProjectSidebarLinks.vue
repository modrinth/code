<template>
	<div
		v-if="
			project.issues_url ||
			project.source_url ||
			project.wiki_url ||
			project.discord_url ||
			project.site_url ||
			projectV3?.link_urls.store?.url ||
			project.donation_urls.length > 0
		"
		class="flex flex-col gap-1 mt-4"
	>
		<div class="flex flex-col gap-1.5">
			<a
				v-if="project.issues_url"
				:href="project.issues_url"
				:target="linkTarget"
				rel="noopener nofollow ugc"
				class="text-link"
			>
				{{ formatMessage(messages.issues) }}
			</a>
			<a
				v-if="project.source_url"
				:href="project.source_url"
				:target="linkTarget"
				rel="noopener nofollow ugc"
				class="text-link"
			>
				{{ formatMessage(messages.source) }}
			</a>
			<a
				v-if="project.wiki_url"
				:href="project.wiki_url"
				:target="linkTarget"
				rel="noopener nofollow ugc"
				class="text-link"
			>
				{{ formatMessage(messages.wiki) }}
			</a>
			<a
				v-if="project.discord_url"
				:href="project.discord_url"
				:target="linkTarget"
				rel="noopener nofollow ugc"
				class="text-link"
			>
				{{ formatMessage(messages.discord) }}
			</a>
			<a
				v-if="projectV3?.link_urls.site?.url"
				:href="projectV3?.link_urls.site?.url"
				:target="linkTarget"
				rel="noopener nofollow ugc"
				class="text-link"
			>
				{{ formatMessage(messages.site) }}
			</a>
			<a
				v-if="projectV3?.link_urls.store?.url"
				:href="projectV3?.link_urls.store?.url"
				:target="linkTarget"
				rel="noopener nofollow ugc"
				class="text-link"
			>
				{{ formatMessage(messages.store) }}
			</a>
			<a
				v-for="(donation, index) in project.donation_urls"
				:key="index"
				:href="donation.url"
				:target="linkTarget"
				rel="noopener nofollow ugc"
				class="text-link"
			>
				<span v-if="donation.id === 'bmac'">{{ formatMessage(messages.donateBmac) }}</span>
				<span v-else-if="donation.id === 'patreon'">{{
					formatMessage(messages.donatePatreon)
				}}</span>
				<span v-else-if="donation.id === 'paypal'">{{ formatMessage(messages.donatePayPal) }}</span>
				<span v-else-if="donation.id === 'ko-fi'">{{ formatMessage(messages.donateKoFi) }}</span>
				<span v-else-if="donation.id === 'github'">{{ formatMessage(messages.donateGithub) }}</span>
				<span v-else>{{ formatMessage(messages.donateGeneric) }}</span>
			</a>
		</div>
	</div>
</template>
<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'

import { defineMessages, useVIntl } from '../../composables/i18n'

const { formatMessage } = useVIntl()

defineProps<{
	project: {
		issues_url: string
		source_url: string
		wiki_url: string
		discord_url: string
		site_url?: string
		store_url?: string
		donation_urls: {
			id: string
			url: string
		}[]
	}
	projectV3?: Labrinth.Projects.v3.Project
	linkTarget: string
}>()

const messages = defineMessages({
	title: {
		id: 'project.about.links.title',
		defaultMessage: 'Links',
	},
	issues: {
		id: 'project.about.links.issues',
		defaultMessage: 'Report issues',
	},
	source: {
		id: 'project.about.links.source',
		defaultMessage: 'View source',
	},
	wiki: {
		id: 'project.about.links.wiki',
		defaultMessage: 'Visit wiki',
	},
	discord: {
		id: 'project.about.links.discord',
		defaultMessage: 'Join Discord server',
	},
	site: {
		id: 'project.about.links.site',
		defaultMessage: 'Visit website',
	},
	store: {
		id: 'project.about.links.store',
		defaultMessage: 'Visit store page',
	},
	donateGeneric: {
		id: 'project.about.links.donate.generic',
		defaultMessage: 'Donate',
	},
	donateGitHub: {
		id: 'project.about.links.donate.github',
		defaultMessage: 'Sponsor on GitHub',
	},
	donateBmac: {
		id: 'project.about.links.donate.bmac',
		defaultMessage: 'Buy Me a Coffee',
	},
	donatePatreon: {
		id: 'project.about.links.donate.patreon',
		defaultMessage: 'Donate on Patreon',
	},
	donatePayPal: {
		id: 'project.about.links.donate.paypal',
		defaultMessage: 'Donate on PayPal',
	},
	donateKoFi: {
		id: 'project.about.links.donate.kofi',
		defaultMessage: 'Donate on Ko-fi',
	},
	donateGithub: {
		id: 'project.about.links.donate.github',
		defaultMessage: 'Sponsor on GitHub',
	},
})
</script>
