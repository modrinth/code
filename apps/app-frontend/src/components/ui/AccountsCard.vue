<template>
	<div
		v-if="mode !== 'isolated'"
		ref="button"
		class="button-base mt-2 px-3 py-2 bg-button-bg rounded-xl flex items-center gap-2"
		:class="{ expanded: mode === 'expanded' }"
		@click="toggleMenu"
	>
		<Avatar
			size="36px"
			:src="
				selectedAccount ? avatarUrl : 'https://launcher-files.modrinth.com/assets/steve_head.png'
			"
		/>
		<div class="flex flex-col w-full">
			<span>{{ selectedAccount ? selectedAccount.profile.name : 'Select account' }}</span>
			<span class="text-secondary text-xs">Minecraft account</span>
		</div>
		<DropdownIcon class="w-5 h-5 shrink-0" />
	</div>
	<transition name="fade">
		<Card
			v-if="showCard || mode === 'isolated'"
			ref="card"
			class="account-card"
			:class="{ expanded: mode === 'expanded', isolated: mode === 'isolated' }"
		>
			<div v-if="selectedAccount" class="selected account">
				<Avatar size="xs" :src="avatarUrl" />
				<div>
					<h4>{{ selectedAccount.profile.name }}</h4>
					<p>Selected</p>
				</div>
				<Button
					v-tooltip="'Log out'"
					icon-only
					color="raised"
					@click="logout(selectedAccount.profile.id)"
				>
					<TrashIcon />
				</Button>
			</div>
			<div v-else class="logged-out account">
				<h4>Not signed in</h4>
				<Button
					v-tooltip="'Log in'"
					:disabled="loginDisabled"
					icon-only
					color="primary"
					@click="login()"
				>
					<LogInIcon v-if="!loginDisabled" />
					<SpinnerIcon v-else class="animate-spin" />
				</Button>
			</div>
			<div v-if="displayAccounts.length > 0" class="account-group">
				<div v-for="account in displayAccounts" :key="account.profile.id" class="account-row">
					<Button class="option account" @click="setAccount(account)">
						<Avatar :src="getAccountAvatarUrl(account)" class="icon" />
						<p>{{ account.profile.name }}</p>
					</Button>
					<Button v-tooltip="'Log out'" icon-only @click="logout(account.profile.id)">
						<TrashIcon />
					</Button>
				</div>
			</div>
			<Button v-if="accounts.length > 0" @click="login()">
				<PlusIcon />
				Add account
			</Button>
		</Card>
	</transition>
</template>

<script setup>
import { DropdownIcon, LogInIcon, PlusIcon, SpinnerIcon, TrashIcon } from '@modrinth/assets'
import { Avatar, Button, Card, injectNotificationManager } from '@modrinth/ui'
import { computed, onBeforeUnmount, onMounted, onUnmounted, ref } from 'vue'

import { trackEvent } from '@/helpers/analytics'
import {
	get_default_user,
	login as login_flow,
	remove_user,
	set_default_user,
	users,
} from '@/helpers/auth'
import { process_listener } from '@/helpers/events'
import { getPlayerHeadUrl } from '@/helpers/rendering/batch-skin-renderer.ts'
import { get_available_skins } from '@/helpers/skins'
import { handleSevereError } from '@/store/error.js'

const { handleError } = injectNotificationManager()

defineProps({
	mode: {
		type: String,
		required: true,
		default: 'normal',
	},
})

const emit = defineEmits(['change'])

const accounts = ref({})
const loginDisabled = ref(false)
const defaultUser = ref()
const equippedSkin = ref(null)
const headUrlCache = ref(new Map())

async function refreshValues() {
	defaultUser.value = await get_default_user().catch(handleError)
	accounts.value = await users().catch(handleError)

	try {
		const skins = await get_available_skins()
		equippedSkin.value = skins.find((skin) => skin.is_equipped)

		if (equippedSkin.value) {
			try {
				const headUrl = await getPlayerHeadUrl(equippedSkin.value)
				headUrlCache.value.set(equippedSkin.value.texture_key, headUrl)
			} catch (error) {
				console.warn('Failed to get head render for equipped skin:', error)
			}
		}
	} catch {
		equippedSkin.value = null
	}
}

function setLoginDisabled(value) {
	loginDisabled.value = value
}

defineExpose({
	refreshValues,
	setLoginDisabled,
	loginDisabled,
})
await refreshValues()

const displayAccounts = computed(() =>
	accounts.value.filter((account) => defaultUser.value !== account.profile.id),
)

const avatarUrl = computed(() => {
	if (equippedSkin.value?.texture_key) {
		const cachedUrl = headUrlCache.value.get(equippedSkin.value.texture_key)
		if (cachedUrl) {
			return cachedUrl
		}
		return `https://mc-heads.net/avatar/${equippedSkin.value.texture_key}/128`
	}
	if (selectedAccount.value?.profile?.id) {
		return `https://mc-heads.net/avatar/${selectedAccount.value.profile.id}/128`
	}
	return 'https://launcher-files.modrinth.com/assets/steve_head.png'
})

function getAccountAvatarUrl(account) {
	if (
		account.profile.id === selectedAccount.value?.profile?.id &&
		equippedSkin.value?.texture_key
	) {
		const cachedUrl = headUrlCache.value.get(equippedSkin.value.texture_key)
		if (cachedUrl) {
			return cachedUrl
		}
	}
	return `https://mc-heads.net/avatar/${account.profile.id}/128`
}

const selectedAccount = computed(() =>
	accounts.value.find((account) => account.profile.id === defaultUser.value),
)

async function setAccount(account) {
	defaultUser.value = account.profile.id
	await set_default_user(account.profile.id).catch(handleError)
	emit('change')
}

async function login() {
	loginDisabled.value = true
	const loggedIn = await login_flow().catch(handleSevereError)

	if (loggedIn) {
		await setAccount(loggedIn)
		await refreshValues()
	}

	trackEvent('AccountLogIn')
	loginDisabled.value = false
}

const logout = async (id) => {
	await remove_user(id).catch(handleError)
	await refreshValues()
	if (!selectedAccount.value && accounts.value.length > 0) {
		await setAccount(accounts.value[0])
		await refreshValues()
	} else {
		emit('change')
	}
	trackEvent('AccountLogOut')
}

const showCard = ref(false)
const card = ref(null)
const button = ref(null)
const handleClickOutside = (event) => {
	const elements = document.elementsFromPoint(event.clientX, event.clientY)
	if (
		card.value &&
		card.value.$el !== event.target &&
		!elements.includes(card.value.$el) &&
		!button.value.contains(event.target)
	) {
		toggleMenu(false)
	}
}

function toggleMenu(override = true) {
	if (showCard.value || !override) {
		showCard.value = false
	} else {
		showCard.value = true
	}
}

const unlisten = await process_listener(async (e) => {
	if (e.event === 'launched') {
		await refreshValues()
	}
})

onMounted(() => {
	window.addEventListener('click', handleClickOutside)
})

onBeforeUnmount(() => {
	window.removeEventListener('click', handleClickOutside)
})

onUnmounted(() => {
	unlisten()
})
</script>

<style scoped lang="scss">
.selected {
	background: var(--color-brand-highlight);
	border-radius: var(--radius-lg);
	color: var(--color-contrast);
	gap: 1rem;
}

.logged-out {
	background: var(--color-bg);
	border-radius: var(--radius-lg);
	gap: 1rem;
}

.account {
	width: max-content;
	display: flex;
	align-items: center;
	text-align: left;
	padding: 0.5rem 1rem;

	h4,
	p {
		margin: 0;
	}
}

.account-card {
	position: fixed;
	display: flex;
	flex-direction: column;
	margin-top: 0.5rem;
	right: 2rem;
	z-index: 11;
	gap: 0.5rem;
	padding: 1rem;
	border: 1px solid var(--color-divider);
	width: max-content;
	user-select: none;
	-ms-user-select: none;
	-webkit-user-select: none;
	max-height: calc(100vh - 300px);
	overflow-y: auto;

	&::-webkit-scrollbar-track {
		border-top-right-radius: 1rem;
		border-bottom-right-radius: 1rem;
	}

	&::-webkit-scrollbar {
		border-top-right-radius: 1rem;
		border-bottom-right-radius: 1rem;
	}

	&.hidden {
		display: none;
	}

	&.expanded {
		left: 13.5rem;
	}

	&.isolated {
		position: relative;
		left: 0;
		top: 0;
	}
}

.accounts-title {
	font-size: 1.2rem;
	font-weight: bolder;
}

.account-group {
	width: 100%;
	display: flex;
	flex-direction: column;
	gap: 0.5rem;
}

.option {
	width: calc(100% - 2.25rem);
	background: var(--color-raised-bg);
	color: var(--color-base);
	box-shadow: none;

	img {
		margin-right: 0.5rem;
	}
}

.icon {
	--size: 1.5rem !important;
}

.account-row {
	display: flex;
	flex-direction: row;
	gap: 0.5rem;
	vertical-align: center;
	justify-content: space-between;
	padding-right: 1rem;
}

.fade-enter-active,
.fade-leave-active {
	transition:
		opacity 0.25s ease,
		translate 0.25s ease,
		scale 0.25s ease;
}

.fade-enter-from,
.fade-leave-to {
	opacity: 0;
	translate: 0 -2rem;
	scale: 0.9;
}

.avatar-button {
	display: flex;
	align-items: center;
	gap: 0.5rem;
	color: var(--color-base);
	background-color: var(--color-button-bg);
	border-radius: var(--radius-md);
	width: 100%;
	padding: 0.5rem 0.75rem;
	text-align: left;

	&.expanded {
		border: 1px solid var(--color-divider);
		padding: 1rem;
	}
}

.avatar-text {
	margin: auto 0 auto 0.25rem;
	display: flex;
	flex-direction: column;
}

.text {
	width: 6rem;
	white-space: nowrap;
	overflow: hidden;
	text-overflow: ellipsis;
}

.accounts-text {
	display: flex;
	align-items: center;
	gap: 0.25rem;
	margin: 0;
}

.qr-code {
	background-color: white !important;
	border-radius: var(--radius-md);
}

.modal-body {
	display: flex;
	flex-direction: row;
	gap: var(--gap-lg);
	align-items: center;
	padding: var(--gap-xl);

	.modal-text {
		display: flex;
		flex-direction: column;
		gap: var(--gap-sm);
		width: 100%;

		h2,
		p {
			margin: 0;
		}

		.code-text {
			display: flex;
			flex-direction: row;
			gap: var(--gap-xs);
			align-items: center;

			.code {
				background-color: var(--color-bg);
				border-radius: var(--radius-md);
				border: solid 1px var(--color-button-bg);
				font-family: var(--mono-font);
				letter-spacing: var(--gap-md);
				color: var(--color-contrast);
				font-size: 2rem;
				font-weight: bold;
				padding: var(--gap-sm) 0 var(--gap-sm) var(--gap-md);
			}

			.btn {
				width: 2.5rem;
				height: 2.5rem;
			}
		}
	}
}

.button-row {
	display: flex;
	flex-direction: row;
}

.modal {
	position: absolute;
}

.code {
	color: var(--color-brand);
	padding: 0.05rem 0.1rem;
	// row not column
	display: flex;

	.card {
		background: var(--color-base);
		color: var(--color-contrast);
		padding: 0.5rem 1rem;
	}
}
</style>
