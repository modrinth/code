<template>
	<div v-if="!owyx.isSignedIn.value" class="flex flex-col items-center gap-4 py-8 text-center">
		<p class="m-0 text-lg font-semibold text-contrast">{{ formatMessage(messages.signInTitle) }}</p>
		<p class="m-0 max-w-md text-secondary">{{ formatMessage(messages.signInBody) }}</p>
		<Button type="colored" color="brand" size="xl" @click="owyx.signIn()">
			{{ formatMessage(messages.signIn) }}
		</Button>
	</div>

	<div v-else class="flex flex-col gap-6 max-w-xl">
		<section class="flex items-center gap-4">
			<img
				:src="avatarSrc"
				alt=""
				class="h-16 w-16 rounded-full object-cover border border-solid border-surface-5 bg-surface-3"
			/>
			<div class="min-w-0">
				<p class="m-0 truncate text-lg font-semibold text-contrast">{{ nickname }}</p>
				<p v-if="email" class="m-0 mt-0.5 truncate text-sm text-secondary">{{ email }}</p>
				<p class="m-0 mt-1 text-xs text-secondary">{{ formatMessage(messages.roleHint, { role }) }}</p>
			</div>
		</section>

		<section class="flex flex-col gap-2">
			<label class="text-sm text-secondary">{{ formatMessage(messages.nickLabel) }}</label>
			<input
				v-model="editNick"
				class="rounded-lg border border-solid border-surface-5 bg-surface-3 px-3 py-2 text-primary"
				maxlength="16"
			/>
			<label class="text-sm text-secondary">{{ formatMessage(messages.discordLabel) }}</label>
			<input
				v-model="editDiscord"
				class="rounded-lg border border-solid border-surface-5 bg-surface-3 px-3 py-2 text-primary"
				placeholder="username"
			/>
			<label class="text-sm text-secondary">{{ formatMessage(messages.currentPasswordLabel) }}</label>
			<input
				v-model="currentPassword"
				type="password"
				autocomplete="current-password"
				class="rounded-lg border border-solid border-surface-5 bg-surface-3 px-3 py-2 text-primary"
			/>
			<label class="text-sm text-secondary">{{ formatMessage(messages.passwordLabel) }}</label>
			<input
				v-model="editPassword"
				type="password"
				autocomplete="new-password"
				class="rounded-lg border border-solid border-surface-5 bg-surface-3 px-3 py-2 text-primary"
				:placeholder="formatMessage(messages.passwordPlaceholder)"
			/>
			<div class="flex flex-wrap gap-2 mt-1">
				<Button type="colored" color="brand" :disabled="saving" @click="saveProfile">
					{{ saving ? '…' : formatMessage(messages.save) }}
				</Button>
				<label class="inline-flex cursor-pointer">
					<span class="sr-only">{{ formatMessage(messages.uploadAvatar) }}</span>
					<input
						type="file"
						accept="image/png,image/jpeg,image/webp"
						class="hidden"
						@change="onAvatar"
					/>
					<span
						class="inline-flex items-center rounded-xl bg-button-bg px-3 py-2 text-sm text-primary hover:brightness-110"
					>
						{{ formatMessage(messages.uploadAvatar) }}
					</span>
				</label>
				<Button class="!bg-button-bg" @click="openProfileSite">
					{{ formatMessage(messages.openProfile) }}
				</Button>
				<Button class="!bg-button-bg" @click="owyx.signOut()">
					{{ formatMessage(messages.signOut) }}
				</Button>
			</div>
			<p v-if="statusMsg" class="m-0 text-sm" :class="statusOk ? 'text-green' : 'text-red'">
				{{ statusMsg }}
			</p>
		</section>
	</div>
</template>

<script setup lang="ts">
import { Button, defineMessages, useVIntl } from '@modrinth/ui'
import { computed, ref, watch } from 'vue'

import { fetch as tauriFetch } from '@tauri-apps/plugin-http'

import {
	DEFAULT_OWYX_API_BASE,
	getOwyxClientKey,
	getStoredOwyxApiBase,
	sanitizeOwyxApiBase,
} from '@/helpers/owyx-api'
import { resolveOwyxAvatarUrl } from '@/helpers/owyx-avatar'
import {
	getStoredOwyxSiteSession,
	OWYX_SITE_PROFILE_URL,
} from '@/helpers/owyx-site-auth'
import { injectOwyxSiteSession } from '@/providers/owyx-site-session'

const { formatMessage } = useVIntl()
const owyx = injectOwyxSiteSession()

const nickname = computed(() => owyx.session.value?.user?.nickname ?? 'Owyx')
const email = computed(() => owyx.session.value?.user?.email ?? '')
const role = computed(() => owyx.session.value?.user?.role ?? 'user')
const avatarSrc = computed(() => resolveOwyxAvatarUrl(owyx.session.value?.user?.avatarUrl))

const editNick = ref(nickname.value)
const editDiscord = ref('')
const editPassword = ref('')
const currentPassword = ref('')
const saving = ref(false)
const statusMsg = ref('')
const statusOk = ref(true)

watch(nickname, (n) => {
	editNick.value = n
})

function apiBase() {
	return sanitizeOwyxApiBase(getStoredOwyxApiBase() || DEFAULT_OWYX_API_BASE)
}

function authHeaders(json = true): Record<string, string> {
	const h: Record<string, string> = { Accept: 'application/json' }
	if (json) h['Content-Type'] = 'application/json'
	const key = getOwyxClientKey()
	if (key) h['X-Owyx-Client-Key'] = key
	const token = getStoredOwyxSiteSession()?.token
	if (token) h.Authorization = `Bearer ${token}`
	return h
}

async function owyxFetch(url: string, init?: RequestInit) {
	try {
		return await tauriFetch(url, init as Parameters<typeof tauriFetch>[1])
	} catch {
		return await fetch(url, init)
	}
}

async function saveProfile() {
	saving.value = true
	statusMsg.value = ''
	try {
		const nick = editNick.value.trim()
		if (nick && nick !== nickname.value) {
			const nickRes = await owyxFetch(`${apiBase()}/api/profile/nickname`, {
				method: 'PUT',
				headers: authHeaders(),
				body: JSON.stringify({ nickname: nick }),
			})
			const nickData = (await nickRes.json().catch(() => ({}))) as { error?: string }
			if (!nickRes.ok) throw new Error(nickData.error || `Nickname failed (${nickRes.status})`)
		}
		const body: Record<string, string> = {}
		if (editDiscord.value.trim()) body.discord_username = editDiscord.value.trim()
		if (editPassword.value) {
			if (!currentPassword.value) {
				throw new Error('Current password is required to set a new password')
			}
			body.current_password = currentPassword.value
			body.new_password = editPassword.value
		}
		if (Object.keys(body).length > 0) {
			const res = await owyxFetch(`${apiBase()}/api/profile`, {
				method: 'PUT',
				headers: authHeaders(),
				body: JSON.stringify(body),
			})
			const data = (await res.json().catch(() => ({}))) as { error?: string }
			if (!res.ok) throw new Error(data.error || `Save failed (${res.status})`)
		}
		editPassword.value = ''
		currentPassword.value = ''
		statusOk.value = true
		statusMsg.value = formatMessage(messages.saved)
		await owyx.refresh()
	} catch (e) {
		statusOk.value = false
		statusMsg.value = e instanceof Error ? e.message : String(e)
	} finally {
		saving.value = false
	}
}

async function onAvatar(e: Event) {
	const input = e.target as HTMLInputElement
	const file = input.files?.[0]
	input.value = ''
	if (!file) return
	saving.value = true
	statusMsg.value = ''
	try {
		const fd = new FormData()
		fd.append('avatar', file)
		fd.append(
			'cropData',
			JSON.stringify({
				scale: 1,
				rotation: 0,
				flipX: 1,
				offsetX: 0,
				offsetY: 0,
				cropSize: 256,
			}),
		)
		const headers = authHeaders(false)
		delete headers['Content-Type']
		const res = await owyxFetch(`${apiBase()}/api/profile/avatar`, {
			method: 'POST',
			headers,
			body: fd,
		})
		const data = (await res.json().catch(() => ({}))) as { error?: string }
		if (!res.ok) throw new Error(data.error || `Upload failed (${res.status})`)
		statusOk.value = true
		statusMsg.value = formatMessage(messages.avatarSaved)
		await owyx.refresh()
	} catch (err) {
		statusOk.value = false
		statusMsg.value = err instanceof Error ? err.message : String(err)
	} finally {
		saving.value = false
	}
}

function openProfileSite() {
	window.open(OWYX_SITE_PROFILE_URL, '_blank', 'noopener,noreferrer')
}

const messages = defineMessages({
	signInTitle: {
		id: 'owyx.settings.profile.sign-in-title',
		defaultMessage: 'Owyx account required',
	},
	signInBody: {
		id: 'owyx.settings.profile.sign-in-body',
		defaultMessage: 'Sign in with the same login as on owyx.site to manage your profile.',
	},
	signIn: {
		id: 'owyx.settings.profile.sign-in',
		defaultMessage: 'Sign in',
	},
	roleHint: {
		id: 'owyx.settings.profile.role',
		defaultMessage: 'Role: {role}',
	},
	nickLabel: {
		id: 'owyx.settings.profile.nick',
		defaultMessage: 'Nickname',
	},
	discordLabel: {
		id: 'owyx.settings.profile.discord',
		defaultMessage: 'Discord username',
	},
	passwordLabel: {
		id: 'owyx.settings.profile.password',
		defaultMessage: 'New password',
	},
	currentPasswordLabel: {
		id: 'owyx.settings.profile.current-password',
		defaultMessage: 'Current password',
	},
	passwordPlaceholder: {
		id: 'owyx.settings.profile.password-placeholder',
		defaultMessage: 'Leave blank to keep current',
	},
	save: {
		id: 'owyx.settings.profile.save',
		defaultMessage: 'Save',
	},
	saved: {
		id: 'owyx.settings.profile.saved',
		defaultMessage: 'Profile saved',
	},
	uploadAvatar: {
		id: 'owyx.settings.profile.upload-avatar',
		defaultMessage: 'Upload avatar',
	},
	avatarSaved: {
		id: 'owyx.settings.profile.avatar-saved',
		defaultMessage: 'Avatar updated',
	},
	openProfile: {
		id: 'owyx.settings.profile.open-site',
		defaultMessage: 'Open on owyx.site',
	},
	signOut: {
		id: 'owyx.settings.profile.sign-out',
		defaultMessage: 'Sign out of Owyx',
	},
})
</script>
