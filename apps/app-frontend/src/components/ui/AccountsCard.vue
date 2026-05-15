<template>
	<div
		v-if="accounts.length === 0"
		class="flex flex-col gap-3 bg-button-bg border border-solid border-surface-5 rounded-xl p-3 mt-2"
	>
		<span>{{ formatMessage(messages.notSignedIn) }}</span>
		<ButtonStyled color="brand">
			<button color="primary" :disabled="loginDisabled" @click="login()">
				<LogInIcon v-if="!loginDisabled" />
				<SpinnerIcon v-else class="animate-spin" />
				{{ formatMessage(messages.signInToMinecraft) }}
			</button>
		</ButtonStyled>
		<ButtonStyled>
			<button @click="addOfflineAccount()">
				<PlusIcon />
				{{ formatMessage(messages.addOfflineAccount) }}
			</button>
		</ButtonStyled>
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
					<span class="text-secondary text-xs">{{
						selectedAccountIsOffline
							? formatMessage(messages.offlineAccount)
							: formatMessage(messages.minecraftAccount)
					}}</span>
				</div>
			</div>
		</template>
		<div class="bg-button-bg pt-1 pb-2 border border-solid border-surface-5">
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
					</button>
					<ButtonStyled circular color="red" color-fill="none" hover-color-fill="background">
						<button
							v-tooltip="formatMessage(messages.removeAccount)"
							class="mr-2"
							@click="logout(account.profile.id)"
						>
							<TrashIcon />
						</button>
					</ButtonStyled>
				</div>
			</template>
			<div class="flex flex-col gap-2 px-2 pt-2">
				<ButtonStyled v-if="accounts.length > 0" class="w-full">
					<button :disabled="loginDisabled" @click="login()">
						<PlusIcon />
						{{ formatMessage(messages.addAccount) }}
					</button>
				</ButtonStyled>
				<ButtonStyled class="w-full">
					<button @click="addOfflineAccount()">
						<PlusIcon />
						{{ formatMessage(messages.addOfflineAccount) }}
					</button>
				</ButtonStyled>
			</div>
		</div>
	</Accordion>
	<NewModal
		ref="offlineAccountModal"
		:header="formatMessage(messages.addOfflineAccount)"
		width="500px"
		max-width="500px"
	>
		<form class="space-y-4 w-full" @submit.prevent="submitOfflineAccount">
			<label class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">
					{{ formatMessage(messages.offlineUsernameLabel) }}
				</span>
				<StyledInput
					v-model="offlineUsername"
					:placeholder="formatMessage(messages.offlineUsernamePlaceholder)"
					autocomplete="off"
					wrapper-class="w-full"
				/>
			</label>
			<p v-if="offlineUsernameError" class="m-0 text-sm text-red">
				{{ offlineUsernameError }}
			</p>
		</form>
		<template #actions>
			<div class="flex gap-2 justify-end">
				<ButtonStyled type="outlined">
					<button @click="hideOfflineAccountModal()">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button :disabled="offlineAccountSaving" @click="submitOfflineAccount()">
						<PlusIcon />
						{{ formatMessage(messages.addOfflineAccount) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import {
	LogInIcon,
	PlusIcon,
	RadioButtonCheckedIcon,
	RadioButtonIcon,
	SpinnerIcon,
	TrashIcon,
	XIcon,
} from '@icarus/assets'
import {
	Accordion,
	Avatar,
	ButtonStyled,
	commonMessages,
	defineMessages,
	injectNotificationManager,
	NewModal,
	StyledInput,
	useVIntl,
} from '@icarus/ui'
import type { Ref } from 'vue'
import { computed, onUnmounted, ref } from 'vue'
import {
	create_offline_user,
	get_default_user,
	login as login_flow,
	remove_user,
	set_default_user,
	users,
} from '@/helpers/auth'
import { process_listener } from '@/helpers/events'
import { getPlayerHeadUrl } from '@/helpers/rendering/batch-skin-renderer.ts'
import type { Skin } from '@/helpers/skins'
import { get_available_skins } from '@/helpers/skins'
import { handleSevereError } from '@/store/error.js'

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()

const emit = defineEmits<{
	change: []
}>()

type MinecraftCredential = {
	profile: {
		id: string
		name: string
	}
	access_token?: string
}

const accounts: Ref<MinecraftCredential[]> = ref([])
const loginDisabled = ref(false)
const offlineAccountSaving = ref(false)
const defaultUser = ref<string | undefined>()
const equippedSkin = ref<Skin | null>(null)
const headUrlCache = ref(new Map<string, string>())
const offlineAccountModal = ref<InstanceType<typeof NewModal>>()
const offlineUsername = ref('')
const offlineUsernameError = ref('')

async function refreshValues() {
	defaultUser.value = await get_default_user().catch(handleError)
	const userList = await users().catch(handleError)
	accounts.value = Array.isArray(userList) ? [...userList] : []
	accounts.value.sort((a, b) => (a.profile?.name ?? '').localeCompare(b.profile?.name ?? ''))

	try {
		const skins = await get_available_skins()
		equippedSkin.value = skins.find((skin) => skin.is_equipped) ?? null

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

function setLoginDisabled(value: boolean) {
	loginDisabled.value = value
}

defineExpose({
	refreshValues,
	setLoginDisabled,
	loginDisabled,
})

await refreshValues()

const selectedAccount = computed(() =>
	accounts.value.find((account) => account.profile.id === defaultUser.value),
)
const selectedAccountIsOffline = computed(() => selectedAccount.value?.access_token === 'OFFLINE')

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

function getAccountAvatarUrl(account: MinecraftCredential) {
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

async function setAccount(account: MinecraftCredential) {
	defaultUser.value = account.profile.id
	await set_default_user(account.profile.id).catch(handleError)
	await refreshValues()
	emit('change')
}

async function login() {
	loginDisabled.value = true
	const loggedIn = await login_flow().catch(handleSevereError)

	if (loggedIn) {
		await setAccount(loggedIn)
	}


	loginDisabled.value = false
}

async function logout(id: string) {
	await remove_user(id).catch(handleError)
	await refreshValues()
	if (!selectedAccount.value && accounts.value.length > 0) {
		await setAccount(accounts.value[0])
	} else {
		emit('change')
	}
}

async function addOfflineAccount() {
	offlineUsername.value = ''
	offlineUsernameError.value = ''
	offlineAccountModal.value?.show()
}

function hideOfflineAccountModal() {
	offlineAccountModal.value?.hide()
}

async function submitOfflineAccount() {
	const trimmed = offlineUsername.value.trim()
	if (!/^[A-Za-z0-9_]{3,16}$/.test(trimmed)) {
		offlineUsernameError.value = formatMessage(messages.offlineUsernameInvalid)
		return
	}
	offlineUsernameError.value = ''
	offlineAccountSaving.value = true

	const account = await create_offline_user(trimmed).catch(handleError)
	if (account) {
		await setAccount(account)
		hideOfflineAccountModal()
	}
	offlineAccountSaving.value = false
}

const unlisten = await process_listener(async (e) => {
	if (e.event === 'launched') {
		await refreshValues()
	}
})

onUnmounted(() => {
	unlisten()
})

const messages = defineMessages({
	notSignedIn: {
		id: 'minecraft-account.not-signed-in',
		defaultMessage: 'Not signed in',
	},
	addAccount: {
		id: 'minecraft-account.add-account',
		defaultMessage: 'Add account',
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
	offlineAccount: {
		id: 'minecraft-account.offline-label',
		defaultMessage: 'Offline account',
	},
	signInToMinecraft: {
		id: 'minecraft-account.sign-in',
		defaultMessage: 'Sign in to Minecraft',
	},
	addOfflineAccount: {
		id: 'minecraft-account.add-offline-account',
		defaultMessage: 'Add offline account',
	},
	offlineUsernameLabel: {
		id: 'minecraft-account.offline-username-label',
		defaultMessage: 'Username',
	},
	offlineUsernamePlaceholder: {
		id: 'minecraft-account.offline-username-placeholder',
		defaultMessage: 'Player123',
	},
	offlineUsernameInvalid: {
		id: 'minecraft-account.offline-username-invalid',
		defaultMessage: "Invalid username. Use 3-16 characters: letters, numbers, '_'",
	},
})
</script>

