<template>
	<div v-if="!owyx.isSignedIn.value" class="flex flex-col items-center gap-4 py-8 text-center">
		<p class="m-0 text-lg font-semibold text-contrast">{{ formatMessage(messages.signInTitle) }}</p>
		<p class="m-0 max-w-md text-secondary">{{ formatMessage(messages.signInBody) }}</p>
		<Button type="colored" color="brand" size="xl" @click="owyx.signIn()">
			{{ formatMessage(messages.signIn) }}
		</Button>
	</div>

	<div v-else class="flex flex-col gap-5 max-w-lg">
		<div>
			<p class="m-0 text-lg font-semibold text-contrast">{{ formatMessage(messages.title) }}</p>
			<p class="m-0 mt-1 text-secondary leading-relaxed">{{ formatMessage(messages.body) }}</p>
		</div>

		<label class="flex items-start gap-3 text-sm text-primary cursor-pointer">
			<input
				v-model="allowRequests"
				type="checkbox"
				class="mt-1 accent-[var(--color-brand)]"
				:disabled="saving"
				@change="saveAllowRequests"
			/>
			<span class="flex flex-col gap-1">
				<span class="font-medium text-contrast">{{ formatMessage(messages.allowRequests) }}</span>
				<span class="text-xs text-secondary">{{ formatMessage(messages.allowRequestsHint) }}</span>
			</span>
		</label>

		<div
			class="rounded-xl border border-solid border-surface-5 bg-surface-2 px-3 py-3 text-sm text-secondary"
		>
			<p class="m-0 font-medium text-contrast">{{ formatMessage(messages.presenceTitle) }}</p>
			<p class="m-0 mt-1 leading-relaxed">{{ formatMessage(messages.presenceBody) }}</p>
		</div>

		<div
			class="rounded-xl border border-solid border-surface-5 bg-surface-2 px-3 py-3 text-sm text-secondary"
		>
			<p class="m-0 font-medium text-contrast">{{ formatMessage(messages.skinsTitle) }}</p>
			<p class="m-0 mt-1 leading-relaxed">{{ formatMessage(messages.skinsBody) }}</p>
			<a
				class="mt-2 inline-block text-link"
				:href="CUSTOM_SKIN_LOADER_MODRINTH"
				target="_blank"
				rel="noopener noreferrer"
			>
				{{ formatMessage(messages.skinsModLink) }}
			</a>
		</div>

		<p v-if="saveError" class="m-0 text-sm text-red">{{ saveError }}</p>
		<p v-else-if="savedFlash" class="m-0 text-sm text-green">{{ formatMessage(messages.saved) }}</p>

		<label class="flex items-start gap-3 text-sm text-primary cursor-pointer">
			<input
				v-model="uiSounds"
				type="checkbox"
				class="mt-1 accent-[var(--color-brand)]"
				@change="onUiSoundsChange"
			/>
			<span class="flex flex-col gap-1">
				<span class="font-medium text-contrast">{{ formatMessage(messages.uiSounds) }}</span>
				<span class="text-xs text-secondary">{{ formatMessage(messages.uiSoundsHint) }}</span>
			</span>
		</label>

		<p class="m-0 text-sm text-secondary">{{ formatMessage(messages.friendsHint) }}</p>
	</div>
</template>

<script setup lang="ts">
import { Button, defineMessages, useVIntl } from '@modrinth/ui'
import { onMounted, ref } from 'vue'

import {
	getOwyxSocialSettings,
	patchOwyxSocialSettings,
} from '@/helpers/owyx-friends'
import { CUSTOM_SKIN_LOADER_MODRINTH } from '@/helpers/owyx-csl'
import { injectOwyxSiteSession } from '@/providers/owyx-site-session'
import {
	getOwyxUiSoundsEnabled,
	playOwyxUiSound,
	setOwyxUiSoundsEnabled,
} from '@/helpers/owyx-ui-sound'

const { formatMessage } = useVIntl()
const owyx = injectOwyxSiteSession()
const allowRequests = ref(true)
const saving = ref(false)
const saveError = ref('')
const savedFlash = ref(false)
const uiSounds = ref(getOwyxUiSoundsEnabled())

function onUiSoundsChange() {
	setOwyxUiSoundsEnabled(uiSounds.value)
	if (uiSounds.value) playOwyxUiSound('toggle')
}

async function loadSettings() {
	if (!owyx.isSignedIn.value) return
	try {
		const s = await getOwyxSocialSettings()
		allowRequests.value = s.allowFriendRequests
	} catch (e) {
		saveError.value = e instanceof Error ? e.message : String(e)
	}
}

async function saveAllowRequests() {
	saving.value = true
	saveError.value = ''
	savedFlash.value = false
	try {
		const s = await patchOwyxSocialSettings({ allowFriendRequests: allowRequests.value })
		allowRequests.value = s.allowFriendRequests
		savedFlash.value = true
		playOwyxUiSound('toggle')
		setTimeout(() => {
			savedFlash.value = false
		}, 2000)
	} catch (e) {
		saveError.value = e instanceof Error ? e.message : String(e)
		// revert optimistic checkbox by reloading
		await loadSettings()
	} finally {
		saving.value = false
	}
}

onMounted(() => {
	void loadSettings()
})

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
	allowRequests: {
		id: 'owyx.settings.social.allow-requests',
		defaultMessage: 'Allow friend requests from anyone',
	},
	allowRequestsHint: {
		id: 'owyx.settings.social.allow-requests-hint',
		defaultMessage:
			'When off, other players cannot send you new requests. Existing friends stay connected.',
	},
	presenceTitle: {
		id: 'owyx.settings.social.presence-title',
		defaultMessage: 'Presence',
	},
	presenceBody: {
		id: 'owyx.settings.social.presence-body',
		defaultMessage:
			'While signed in, the launcher sends a short heartbeat (~30s). Friends see you as online or playing. After about 90 seconds without a heartbeat you appear offline.',
	},
	skinsTitle: {
		id: 'owyx.settings.social.skins-title',
		defaultMessage: 'In-world skins',
	},
	skinsBody: {
		id: 'owyx.settings.social.skins-body',
		defaultMessage:
			'Upload a skin on owyx.site. Install CustomSkinLoader in your instance (Owyx writes the config when you enable skins from instance options or Admin). Others with the same mod see your Owyx skin.',
	},
	skinsModLink: {
		id: 'owyx.settings.social.skins-mod-link',
		defaultMessage: 'Open CustomSkinLoader on Modrinth',
	},
	friendsHint: {
		id: 'owyx.settings.social.friends-hint',
		defaultMessage: 'Tip: open the Friends panel from the home sidebar to manage requests.',
	},
	saved: {
		id: 'owyx.settings.social.saved',
		defaultMessage: 'Saved',
	},
	uiSounds: {
		id: 'owyx.settings.social.ui-sounds',
		defaultMessage: 'Soft UI sounds',
	},
	uiSoundsHint: {
		id: 'owyx.settings.social.ui-sounds-hint',
		defaultMessage:
			'Synthetic clicks generated in the app (no sample packs). Muted automatically when prefers-reduced-motion is on.',
	},
})
</script>
