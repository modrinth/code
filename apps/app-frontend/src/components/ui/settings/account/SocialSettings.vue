<template>
	<div v-if="!owyx.isSignedIn.value" class="flex flex-col items-center gap-4 py-8 text-center">
		<p class="m-0 text-lg font-semibold text-contrast">{{ formatMessage(messages.signInTitle) }}</p>
		<p class="m-0 max-w-md text-secondary">{{ formatMessage(messages.signInBody) }}</p>
		<Button type="colored" color="brand" size="xl" @click="owyx.signIn()">
			{{ formatMessage(messages.signIn) }}
		</Button>
	</div>

	<div v-else class="flex flex-col gap-5 max-w-lg">
		<div class="flex flex-wrap items-start justify-between gap-3">
			<div>
				<p class="m-0 text-lg font-semibold text-contrast">{{ formatMessage(messages.title) }}</p>
				<p class="m-0 mt-1 text-sm text-secondary">
					{{
						formatMessage(messages.signedInAs, {
							nick:
								owyx.session.value?.user?.displayNickname ||
								owyx.session.value?.user?.nickname ||
								'—',
						})
					}}
				</p>
			</div>
			<Button size="sm" @click="openFriends()">
				{{ formatMessage(messages.openFriends) }}
			</Button>
		</div>

		<div class="grid grid-cols-3 gap-2">
			<button
				type="button"
				class="rounded-xl border border-solid border-surface-5 bg-surface-2 px-3 py-3 text-center cursor-pointer button-base hover:border-[var(--color-brand)]"
				@click="openFriends()"
			>
				<p class="m-0 text-2xl font-semibold text-contrast tabular-nums">{{ stats.friends }}</p>
				<p class="m-0 mt-1 text-[11px] uppercase tracking-wide text-secondary">
					{{ formatMessage(messages.statFriends) }}
				</p>
			</button>
			<button
				type="button"
				class="rounded-xl border border-solid border-surface-5 bg-surface-2 px-3 py-3 text-center cursor-pointer button-base hover:border-[var(--color-brand)]"
				@click="openFriends()"
			>
				<p class="m-0 text-2xl font-semibold text-contrast tabular-nums">{{ stats.online }}</p>
				<p class="m-0 mt-1 text-[11px] uppercase tracking-wide text-secondary">
					{{ formatMessage(messages.statOnline) }}
				</p>
			</button>
			<button
				type="button"
				class="rounded-xl border border-solid border-surface-5 bg-surface-2 px-3 py-3 text-center cursor-pointer button-base hover:border-[var(--color-brand)]"
				@click="openFriends('incoming')"
			>
				<p class="m-0 text-2xl font-semibold text-contrast tabular-nums">{{ stats.pending }}</p>
				<p class="m-0 mt-1 text-[11px] uppercase tracking-wide text-secondary">
					{{ formatMessage(messages.statPending) }}
				</p>
			</button>
		</div>

		<div
			class="flex items-center justify-between gap-3 rounded-xl border border-solid border-surface-5 bg-surface-2 px-3 py-3"
		>
			<div class="min-w-0">
				<p class="m-0 font-medium text-contrast">{{ formatMessage(messages.presenceTitle) }}</p>
				<p class="m-0 mt-0.5 text-xs text-secondary">{{ formatMessage(messages.presenceBody) }}</p>
			</div>
			<span
				class="shrink-0 rounded-full px-2.5 py-1 text-[11px] font-semibold uppercase tracking-wide"
				:class="presenceLive ? 'bg-green/20 text-green' : 'bg-button-bg text-secondary'"
			>
				{{
					presenceLive
						? formatMessage(messages.presenceOn)
						: formatMessage(messages.presenceOff)
				}}
			</span>
		</div>

		<label
			class="flex items-start justify-between gap-3 rounded-xl border border-solid border-surface-5 bg-surface-2 px-3 py-3 text-sm text-primary cursor-pointer"
		>
			<span class="flex flex-col gap-1 min-w-0">
				<span class="font-medium text-contrast">{{ formatMessage(messages.allowRequests) }}</span>
				<span class="text-xs text-secondary">{{ formatMessage(messages.allowRequestsHint) }}</span>
			</span>
			<input
				v-model="allowRequests"
				type="checkbox"
				class="mt-1 accent-[var(--color-brand)] shrink-0"
				:disabled="saving"
				@change="saveAllowRequests"
			/>
		</label>

		<div
			class="rounded-xl border border-solid border-surface-5 bg-surface-2 px-3 py-3 text-sm text-secondary"
		>
			<p class="m-0 font-medium text-contrast">{{ formatMessage(messages.skinsTitle) }}</p>
			<p class="m-0 mt-1 leading-relaxed">{{ formatMessage(messages.skinsBody) }}</p>
			<div class="mt-3 flex flex-wrap gap-2">
				<Button size="sm" type="colored" color="brand" :disabled="installingCsl" @click="installCsl">
					{{ formatMessage(messages.installCsl) }}
				</Button>
				<a
					class="btn-like inline-flex items-center rounded-lg border border-solid border-surface-5 bg-button-bg px-3 py-1.5 text-xs font-medium text-primary no-underline hover:border-[var(--color-brand)]"
					href="https://owyx.site/profile"
					target="_blank"
					rel="noopener noreferrer"
				>
					{{ formatMessage(messages.openProfile) }}
				</a>
				<a
					class="inline-flex items-center text-xs text-link"
					:href="CUSTOM_SKIN_LOADER_MODRINTH"
					target="_blank"
					rel="noopener noreferrer"
				>
					{{ formatMessage(messages.skinsModLink) }}
				</a>
			</div>
		</div>

		<p v-if="saveError" class="m-0 text-sm text-red">{{ saveError }}</p>
		<p v-else-if="savedFlash" class="m-0 text-sm text-green">{{ formatMessage(messages.saved) }}</p>
		<p v-else-if="loadError" class="m-0 text-sm text-secondary">{{ loadError }}</p>
	</div>
</template>

<script setup lang="ts">
import { Button, defineMessages, injectNotificationManager, useVIntl } from '@modrinth/ui'
import { computed, inject, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'

import {
	CUSTOM_SKIN_LOADER_MODRINTH,
	CUSTOM_SKIN_LOADER_PROJECT_ID,
} from '@/helpers/owyx-csl'
import {
	getOwyxSocialSettings,
	listOwyxFriends,
	patchOwyxSocialSettings,
} from '@/helpers/owyx-friends'
import { owyxPresenceStatus } from '@/helpers/owyx-presence'
import { playOwyxUiSound } from '@/helpers/owyx-ui-sound'
import { injectContentInstall } from '@/providers/content-install'
import { appSettingsModalContextKey } from '@/providers/app-settings-modal'
import { injectOwyxSiteSession } from '@/providers/owyx-site-session'

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const router = useRouter()
const owyx = injectOwyxSiteSession()
const { install: installVersion } = injectContentInstall()
const settingsModal = inject(appSettingsModalContextKey, null)
const allowRequests = ref(true)
const saving = ref(false)
const saveError = ref('')
const loadError = ref('')
const savedFlash = ref(false)
const installingCsl = ref(false)
const stats = ref({ friends: 0, online: 0, pending: 0 })

const presenceLive = computed(() => {
	const status = owyxPresenceStatus.value
	return status === 'online' || status === 'playing'
})

function openFriends(focus?: 'incoming') {
	settingsModal?.close()
	void router.push('/')
	window.dispatchEvent(
		new CustomEvent('owyx:open-friends', { detail: { focus: focus || 'list' } }),
	)
}

async function installCsl() {
	if (installingCsl.value) return
	installingCsl.value = true
	try {
		settingsModal?.close()
		await installVersion(
			CUSTOM_SKIN_LOADER_PROJECT_ID,
			null,
			null,
			'SocialSettings',
			() => {
				installingCsl.value = false
			},
			(instanceId) => {
				void router.push(`/instance/${encodeURIComponent(instanceId)}`)
			},
		)
	} catch (e) {
		installingCsl.value = false
		handleError(e)
	}
}

async function loadSettings() {
	if (!owyx.isSignedIn.value) return
	loadError.value = ''
	try {
		const [s, friends] = await Promise.all([getOwyxSocialSettings(), listOwyxFriends()])
		allowRequests.value = s.allowFriendRequests
		const accepted = friends.filter((f) => f.status === 'accepted')
		stats.value = {
			friends: accepted.length,
			online: accepted.filter((f) => f.presence === 'online' || f.presence === 'playing').length,
			pending: friends.filter((f) => f.status === 'pending' && f.incoming).length,
		}
	} catch (e) {
		loadError.value = e instanceof Error ? e.message : String(e)
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
		await loadSettings()
	} finally {
		saving.value = false
	}
}

function onVisibility() {
	if (document.visibilityState === 'visible') void loadSettings()
}

onMounted(() => {
	void loadSettings()
	document.addEventListener('visibilitychange', onVisibility)
})
onUnmounted(() => {
	document.removeEventListener('visibilitychange', onVisibility)
})
watch(
	() => owyx.isSignedIn.value,
	() => {
		void loadSettings()
	},
)

const messages = defineMessages({
	signInTitle: {
		id: 'owyx.settings.social.sign-in-title',
		defaultMessage: 'Owyx account required',
	},
	signInBody: {
		id: 'owyx.settings.social.sign-in-body',
		defaultMessage: 'Sign in to manage friends, presence, and social privacy.',
	},
	signIn: {
		id: 'owyx.settings.social.sign-in',
		defaultMessage: 'Sign in',
	},
	title: {
		id: 'owyx.settings.social.title',
		defaultMessage: 'Friends & social',
	},
	signedInAs: {
		id: 'owyx.settings.social.signed-in-as',
		defaultMessage: 'Signed in as {nick}',
	},
	openFriends: {
		id: 'owyx.settings.social.open-friends',
		defaultMessage: 'Open Friends',
	},
	statFriends: {
		id: 'owyx.settings.social.stat-friends',
		defaultMessage: 'Friends',
	},
	statOnline: {
		id: 'owyx.settings.social.stat-online',
		defaultMessage: 'Online',
	},
	statPending: {
		id: 'owyx.settings.social.stat-pending',
		defaultMessage: 'Incoming',
	},
	allowRequests: {
		id: 'owyx.settings.social.allow-requests',
		defaultMessage: 'Allow friend requests from anyone',
	},
	allowRequestsHint: {
		id: 'owyx.settings.social.allow-requests-hint',
		defaultMessage: 'When off, others cannot send new requests. Existing friends stay connected.',
	},
	presenceTitle: {
		id: 'owyx.settings.social.presence-title',
		defaultMessage: 'Presence',
	},
	presenceBody: {
		id: 'owyx.settings.social.presence-body',
		defaultMessage: 'Heartbeat ~30s while signed in. Idle ~90s → appear offline to friends.',
	},
	presenceOn: {
		id: 'owyx.settings.social.presence-on',
		defaultMessage: 'Active',
	},
	presenceOff: {
		id: 'owyx.settings.social.presence-off',
		defaultMessage: 'Off',
	},
	skinsTitle: {
		id: 'owyx.settings.social.skins-title',
		defaultMessage: 'In-world skins',
	},
	skinsBody: {
		id: 'owyx.settings.social.skins-body',
		defaultMessage:
			'Upload on owyx.site. CustomSkinLoader in your instance reads https://owyx.site/api/csl/ (Owyx writes config on launch).',
	},
	installCsl: {
		id: 'owyx.settings.social.install-csl',
		defaultMessage: 'Install CustomSkinLoader to instance',
	},
	openProfile: {
		id: 'owyx.settings.social.open-profile',
		defaultMessage: 'Open profile on site',
	},
	skinsModLink: {
		id: 'owyx.settings.social.skins-mod-link',
		defaultMessage: 'CustomSkinLoader on Modrinth',
	},
	saved: {
		id: 'owyx.settings.social.saved',
		defaultMessage: 'Saved',
	},
})
</script>
