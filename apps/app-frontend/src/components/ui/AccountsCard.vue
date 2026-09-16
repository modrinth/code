<template>
	<div
		v-if="accounts.length === 0"
		class="flex flex-col gap-3 bg-button-bg border border-solid border-surface-5 rounded-xl p-3 mt-2"
	>
		<span class="font-medium text-contrast">{{ formatMessage(messages.notSignedIn) }}</span>
		<p class="m-0 text-xs text-secondary leading-relaxed">{{ formatMessage(messages.pathsHint) }}</p>
		<div class="flex flex-col gap-2">
			<Button
				type="colored"
				color="brand"
				:disabled="loginDisabled"
				@click="signInOwyxSite()"
			>
				{{ formatMessage(messages.signInOwyxSite) }}
			</Button>
			<Button
				class="!bg-button-bg !text-primary ![box-shadow:var(--shadow-button)]"
				:disabled="loginDisabled"
				@click="showOfflineForm = !showOfflineForm"
			>
				{{ formatMessage(messages.offlineNickname) }}
			</Button>
			<Button
				class="!bg-button-bg !text-primary ![box-shadow:var(--shadow-button)]"
				:disabled="loginDisabled"
				@click="loginMicrosoft()"
			>
				<LogInIcon v-if="!loginDisabled" />
				<SpinnerIcon v-else class="animate-spin" />
				{{ formatMessage(messages.signInMicrosoft) }}
			</Button>
		</div>
		<p class="m-0 text-[11px] text-secondary">{{ formatMessage(messages.microsoftStubHint) }}</p>
		<div v-if="showOfflineForm" class="flex flex-col gap-2 pt-1">
			<p class="m-0 text-xs text-secondary">{{ formatMessage(messages.offlineWarning) }}</p>
			<input
				v-model="offlineNickname"
				class="w-full rounded-lg border border-solid border-surface-5 bg-surface-3 px-3 py-2 text-primary"
				type="text"
				maxlength="16"
				:placeholder="formatMessage(messages.nicknamePlaceholder)"
				@keydown.enter="loginOffline()"
			/>
			<Button type="colored" color="brand" :disabled="loginDisabled || !offlineNickname.trim()" @click="loginOffline()">
				{{ formatMessage(messages.playOffline) }}
			</Button>
		</div>
	</div>
	<Accordion
		v-else
		class="w-full mt-2 bg-button-bg border border-solid border-surface-5 rounded-xl overflow-clip"
		button-class="button-base w-full bg-transparent px-3 py-2 border-0 cursor-pointer"
		:open-by-default="false"
	>
		<template #title>
			<div class="flex gap-2 w-full min-w-0">
				<Avatar
					size="36px"
					:src="
						selectedAccount
							? avatarUrl
							: 'https://launcher-files.modrinth.com/assets/steve_head.png'
					"
				/>
				<div class="flex flex-col items-start w-full min-w-0">
					<span class="truncate w-full text-left">{{
						selectedAccount ? selectedAccount.profile.name : formatMessage(messages.selectAccount)
					}}</span>
					<span class="text-secondary text-xs">{{ accountTypeLabel(selectedAccount) }}</span>
					<span
						v-if="owyxSite.isSignedIn.value"
						class="mt-0.5 text-[10px] font-medium uppercase tracking-wide text-brand"
					>
						{{ formatMessage(messages.owyxSessionBadge) }}
					</span>
				</div>
			</div>
		</template>
		<div class="bg-button-bg pt-1 pb-2 border-0 border-t border-solid border-surface-5">
			<template v-if="accounts.length > 0">
				<div v-for="account in accounts" :key="account.profile.id" class="flex gap-1 items-center">
					<button
						class="flex items-center flex-shrink flex-grow overflow-clip gap-2 p-2 border-0 bg-transparent cursor-pointer button-base min-w-0"
						@click="setAccount(account)"
					>
						<RadioButtonCheckedIcon
							v-if="selectedAccount && selectedAccount.profile.id === account.profile.id"
							class="w-5 h-5 text-brand shrink-0"
						/>
						<RadioButtonIcon v-else class="w-5 h-5 text-secondary shrink-0" />
						<Avatar :src="getAccountAvatarUrl(account)" size="24px" />
						<div class="flex flex-col items-start min-w-0">
							<p
								class="m-0 truncate min-w-0"
								:class="
									selectedAccount && selectedAccount.profile.id === account.profile.id
										? 'text-contrast font-semibold'
										: 'text-primary'
								"
							>
								{{ account.profile.name }}
							</p>
							<span class="text-xs text-secondary">{{ accountTypeLabel(account) }}</span>
						</div>
					</button>
					<IconButton
						v-tooltip="formatMessage(messages.removeAccount)"
						type="quiet"
						color="red"
						:label="formatMessage(messages.removeAccount)"
						class="mr-2 !bg-button-bg !text-primary ![box-shadow:var(--shadow-button)] hover:!bg-red focus-visible:!bg-red hover:!text-[var(--color-accent-contrast)] focus-visible:!text-[var(--color-accent-contrast)]"
						@click="logout(account.profile.id)"
					>
						<TrashIcon />
					</IconButton>
				</div>
			</template>
			<div class="flex flex-col gap-2 px-2 pt-2">
				<Button
					class="w-full !bg-button-bg !text-primary ![box-shadow:var(--shadow-button)]"
					:disabled="loginDisabled"
					@click="signInOwyxSite()"
				>
					<PlusIcon />
					{{ formatMessage(messages.signInOwyxSite) }}
				</Button>
				<Button
					class="w-full !bg-button-bg !text-primary ![box-shadow:var(--shadow-button)]"
					:disabled="loginDisabled"
					@click="showOfflineForm = !showOfflineForm"
				>
					<PlusIcon />
					{{ formatMessage(messages.addOffline) }}
				</Button>
				<Button
					class="w-full !bg-button-bg !text-primary ![box-shadow:var(--shadow-button)]"
					:disabled="loginDisabled"
					@click="loginMicrosoft()"
				>
					<PlusIcon />
					{{ formatMessage(messages.addMicrosoft) }}
				</Button>
				<p class="m-0 px-1 text-[11px] text-secondary">{{ formatMessage(messages.microsoftStubHint) }}</p>
				<div v-if="showOfflineForm" class="flex flex-col gap-2 pb-1">
					<p class="m-0 text-xs text-secondary">{{ formatMessage(messages.offlineWarning) }}</p>
					<input
						v-model="offlineNickname"
						class="w-full rounded-lg border border-solid border-surface-5 bg-surface-3 px-3 py-2 text-primary"
						type="text"
						maxlength="16"
						:placeholder="formatMessage(messages.nicknamePlaceholder)"
						@keydown.enter="loginOffline()"
					/>
					<Button
						type="colored"
						color="brand"
						:disabled="loginDisabled || !offlineNickname.trim()"
						@click="loginOffline()"
					>
						{{ formatMessage(messages.playOffline) }}
					</Button>
				</div>
			</div>
		</div>
	</Accordion>
</template>

<script setup lang="ts">
import {
	LogInIcon,
	PlusIcon,
	RadioButtonCheckedIcon,
	RadioButtonIcon,
	SpinnerIcon,
	TrashIcon,
} from '@modrinth/assets'
import {
	Accordion,
	Avatar,
	Button,
	defineMessages,
	IconButton,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import type { Ref } from 'vue'
import { computed, onUnmounted, ref } from 'vue'

import { useAppEvent } from '@/composables/use-app-event'
import { handleSevereError } from '@/composables/use-error.js'
import { trackEvent } from '@/helpers/analytics'
import {
	get_default_user,
	login as login_flow,
	login_offline as login_offline_flow,
	remove_user,
	set_default_user,
	users,
} from '@/helpers/auth'
import { getPlayerHeadUrl } from '@/helpers/rendering/player-head'
import { resolveOwyxAvatarUrl } from '@/helpers/owyx-avatar'
import type { Skin } from '@/helpers/skins'
import { get_available_skins } from '@/helpers/skins'
import { injectOwyxSiteSession } from '@/providers/owyx-site-session'

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const owyxSite = injectOwyxSiteSession()

const emit = defineEmits<{
	change: []
}>()

type MinecraftCredential = {
	profile: {
		id: string
		name: string
	}
	is_offline?: boolean
	refresh_token?: string
}

const accounts: Ref<MinecraftCredential[]> = ref([])
const loginDisabled = ref(false)
const defaultUser = ref<string | undefined>()
const equippedSkin = ref<Skin | null>(null)
const equippedHeadUrl = ref<string>()
const showOfflineForm = ref(false)
const offlineNickname = ref('')
let headRequest = 0

function isOfflineAccount(account?: MinecraftCredential | null) {
	if (!account) return false
	// Match Rust Credentials::is_offline: marker refresh_token, or empty refresh + empty/"0" access
	if (account.is_offline === true || account.refresh_token === 'owyx-offline') return true
	const refresh = account.refresh_token ?? ''
	const access = (account as { access_token?: string }).access_token ?? ''
	return refresh === '' && (access === '' || access === '0')
}

function accountTypeLabel(account?: MinecraftCredential | null) {
	if (!account) return formatMessage(messages.minecraftAccount)
	return isOfflineAccount(account)
		? formatMessage(messages.offlineAccount)
		: formatMessage(messages.microsoftAccount)
}

async function updateHeadUrl(skin: Skin | null) {
	const request = ++headRequest
	if (equippedHeadUrl.value) URL.revokeObjectURL(equippedHeadUrl.value)
	equippedHeadUrl.value = undefined
	if (!skin) return
	const url = await getPlayerHeadUrl(skin)
	if (request !== headRequest) URL.revokeObjectURL(url)
	else equippedHeadUrl.value = url
}

onUnmounted(() => {
	headRequest++
	if (equippedHeadUrl.value) URL.revokeObjectURL(equippedHeadUrl.value)
})

async function refreshValues() {
	defaultUser.value = await get_default_user().catch(handleError)
	const userList = await users().catch(handleError)
	accounts.value = Array.isArray(userList) ? [...userList] : []
	accounts.value.sort((a, b) => (a.profile?.name ?? '').localeCompare(b.profile?.name ?? ''))

	try {
		const skins = await get_available_skins()
		equippedSkin.value = skins.find((skin) => skin.is_equipped) ?? null

		await updateHeadUrl(equippedSkin.value)
	} catch {
		equippedSkin.value = null
		void updateHeadUrl(null)
	}
}

async function setEquippedSkin(skin: Skin) {
	equippedSkin.value = skin

	try {
		await updateHeadUrl(skin)
	} catch (error) {
		console.warn('Failed to get head render for equipped skin:', error)
	}
}

function setLoginDisabled(value: boolean) {
	loginDisabled.value = value
}

defineExpose({
	refreshValues,
	setEquippedSkin,
	setLoginDisabled,
	login: loginMicrosoft,
	loginDisabled,
})

await refreshValues()

const selectedAccount = computed(() =>
	accounts.value.find((account) => account.profile.id === defaultUser.value),
)

function getAccountAvatarUrl(account: MinecraftCredential) {
	if (isOfflineAccount(account)) {
		const site = owyxSite.session.value?.user
		if (site?.nickname && site.nickname.toLowerCase() === account.profile.name.toLowerCase()) {
			return resolveOwyxAvatarUrl(site.avatarUrl)
		}
		return resolveOwyxAvatarUrl(null)
	}
	if (
		account.profile.id === selectedAccount.value?.profile?.id &&
		equippedSkin.value?.texture_key
	) {
		const cachedUrl = equippedHeadUrl.value
		if (cachedUrl) {
			return cachedUrl
		}
	}
	return `https://mc-heads.net/avatar/${account.profile.id}/128`
}

const avatarUrl = computed(() => {
	if (selectedAccount.value && isOfflineAccount(selectedAccount.value)) {
		return getAccountAvatarUrl(selectedAccount.value)
	}
	if (equippedSkin.value?.texture_key) {
		const cachedUrl = equippedHeadUrl.value
		if (cachedUrl) {
			return cachedUrl
		}
		return `https://mc-heads.net/avatar/${equippedSkin.value.texture_key}/128`
	}
	if (selectedAccount.value?.profile?.id) {
		return `https://mc-heads.net/avatar/${selectedAccount.value.profile.id}/128`
	}
	return resolveOwyxAvatarUrl(null)
})

async function setAccount(account: MinecraftCredential) {
	defaultUser.value = account.profile.id
	await set_default_user(account.profile.id).catch(handleError)
	await refreshValues()
	emit('change')
}

async function loginMicrosoft() {
	loginDisabled.value = true
	const loggedIn = await login_flow().catch(handleSevereError)

	if (loggedIn) {
		await setAccount(loggedIn)
	}

	trackEvent('AccountLogIn')
	loginDisabled.value = false
}

async function loginOffline() {
	const name = offlineNickname.value.trim()
	if (!name) return
	loginDisabled.value = true
	try {
		const loggedIn = await login_offline_flow(name, true).catch(handleSevereError)
		if (loggedIn) {
			await setAccount(loggedIn)
			offlineNickname.value = ''
			showOfflineForm.value = false
		}
		trackEvent('AccountLogInOffline')
	} finally {
		loginDisabled.value = false
	}
}

/**
 * Sign in to Owyx site session (friends/skins). Sync nickname onto disk without
 * stealing an existing active Microsoft account when other profiles already exist.
 */
async function signInOwyxSite() {
	if (loginDisabled.value) return
	loginDisabled.value = true
	try {
		await owyxSite.signIn()
		const nick = owyxSite.session.value?.user?.nickname
		if (!nick) return
		const makeActive = accounts.value.length === 0
		const loggedIn = await login_offline_flow(nick, makeActive).catch(handleSevereError)
		if (loggedIn && makeActive) {
			await setAccount(loggedIn)
		} else {
			await refreshValues()
			emit('change')
		}
		trackEvent('AccountLogInOwyxSite')
	} catch (e) {
		handleError(e)
	} finally {
		loginDisabled.value = false
	}
}

async function logout(id: string) {
	await remove_user(id).catch(handleError)
	await refreshValues()
	if (!selectedAccount.value && accounts.value.length > 0) {
		await setAccount(accounts.value[0])
	} else {
		emit('change')
	}
	trackEvent('AccountLogOut')
}

useAppEvent('process', async (e) => {
	if (e.event === 'launched') {
		await refreshValues()
	}
})

const messages = defineMessages({
	notSignedIn: {
		id: 'minecraft-account.not-signed-in',
		defaultMessage: 'Choose how to play',
	},
	pathsHint: {
		id: 'minecraft-account.paths-hint',
		defaultMessage:
			'1) Sign in with your Owyx site account · 2) Use an offline nickname · 3) Microsoft (licensed Minecraft).',
	},
	signInOwyxSite: {
		id: 'minecraft-account.sign-in-owyx-site',
		defaultMessage: 'Sign in with Owyx account',
	},
	owyxSessionBadge: {
		id: 'minecraft-account.owyx-session-badge',
		defaultMessage: 'Owyx site session',
	},
	microsoftStubHint: {
		id: 'minecraft-account.microsoft-stub-hint',
		defaultMessage:
			'Microsoft is for a licensed Minecraft profile. Signing into Owyx does not replace an active Microsoft account. Use “Play with offline nickname” if you want the offline profile selected.',
	},
	addAccount: {
		id: 'minecraft-account.add-account',
		defaultMessage: 'Add account',
	},
	addMicrosoft: {
		id: 'minecraft-account.add-microsoft',
		defaultMessage: 'Add Microsoft account',
	},
	addOffline: {
		id: 'minecraft-account.add-offline',
		defaultMessage: 'Add offline nickname',
	},
	removeAccount: {
		id: 'minecraft-account.remove-account',
		defaultMessage: 'Remove account',
	},
	selectAccount: {
		id: 'minecraft-account.select-account',
		defaultMessage: 'Select account',
	},
	minecraftAccount: {
		id: 'minecraft-account.label',
		defaultMessage: 'Minecraft account',
	},
	microsoftAccount: {
		id: 'minecraft-account.microsoft',
		defaultMessage: 'Microsoft',
	},
	offlineAccount: {
		id: 'minecraft-account.offline',
		defaultMessage: 'Owyx / offline',
	},
	signInMicrosoft: {
		id: 'minecraft-account.sign-in-microsoft',
		defaultMessage: 'Sign in with Microsoft',
	},
	offlineNickname: {
		id: 'minecraft-account.offline-nickname',
		defaultMessage: 'Play with offline nickname',
	},
	offlineWarning: {
		id: 'minecraft-account.offline-warning',
		defaultMessage:
			'Uses a nickname for offline-mode servers. Prefer the same nick as on owyx.site so skins and friends match. Public Microsoft-authenticated servers still need a Microsoft account.',
	},
	nicknamePlaceholder: {
		id: 'minecraft-account.nickname-placeholder',
		defaultMessage: 'Nickname',
	},
	playOffline: {
		id: 'minecraft-account.play-offline',
		defaultMessage: 'Save nickname',
	},
	signInToMinecraft: {
		id: 'minecraft-account.sign-in',
		defaultMessage: 'Sign in to Minecraft',
	},
})
</script>
