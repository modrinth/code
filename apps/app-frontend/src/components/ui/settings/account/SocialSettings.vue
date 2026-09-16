<template>
	<div v-if="!owyx.isSignedIn.value" class="flex flex-col items-center gap-4 py-8 text-center">
		<p class="m-0 text-lg font-semibold text-contrast">{{ formatMessage(messages.signInTitle) }}</p>
		<p class="m-0 max-w-md text-secondary">{{ formatMessage(messages.signInBody) }}</p>
		<Button type="colored" color="brand" size="xl" @click="owyx.signIn()">
			{{ formatMessage(messages.signIn) }}
		</Button>
	</div>

	<div v-else class="flex flex-col gap-4 max-w-lg">
		<p class="m-0 text-lg font-semibold text-contrast">{{ formatMessage(messages.title) }}</p>
		<p class="m-0 text-secondary leading-relaxed">{{ formatMessage(messages.body) }}</p>
		<p class="m-0 text-sm text-secondary">{{ formatMessage(messages.privacyStub) }}</p>
		<label class="flex items-center gap-2 text-sm text-secondary">
			<input v-model="allowRequests" type="checkbox" disabled />
			{{ formatMessage(messages.allowRequests) }}
			<span class="text-xs">({{ formatMessage(messages.comingSoon) }})</span>
		</label>
		<p class="m-0 text-sm text-secondary">{{ formatMessage(messages.friendsHint) }}</p>
	</div>
</template>

<script setup lang="ts">
import { Button, defineMessages, useVIntl } from '@modrinth/ui'
import { ref } from 'vue'

import { injectOwyxSiteSession } from '@/providers/owyx-site-session'

const { formatMessage } = useVIntl()
const owyx = injectOwyxSiteSession()
const allowRequests = ref(true)

const messages = defineMessages({
	signInTitle: {
		id: 'owyx.settings.social.sign-in-title',
		defaultMessage: 'Owyx account required',
	},
	signInBody: {
		id: 'owyx.settings.social.sign-in-body',
		defaultMessage: 'Sign in to manage friends and social privacy.',
	},
	signIn: {
		id: 'owyx.settings.social.sign-in',
		defaultMessage: 'Sign in',
	},
	title: {
		id: 'owyx.settings.social.title',
		defaultMessage: 'Friends & social',
	},
	body: {
		id: 'owyx.settings.social.body',
		defaultMessage:
			'Friend requests use Owyx nicknames. Use the Friends panel in the sidebar to add, accept, or remove friends.',
	},
	privacyStub: {
		id: 'owyx.settings.social.privacy',
		defaultMessage: 'Friend-request privacy controls will land here in a later build.',
	},
	allowRequests: {
		id: 'owyx.settings.social.allow-requests',
		defaultMessage: 'Allow friend requests from anyone',
	},
	comingSoon: {
		id: 'owyx.settings.social.coming-soon',
		defaultMessage: 'coming soon',
	},
	friendsHint: {
		id: 'owyx.settings.social.friends-hint',
		defaultMessage: 'Live presence (“what they’re playing”) is not in this release.',
	},
})
</script>
