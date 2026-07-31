<template>
	<NewModal
		ref="modal"
		:header="formatMessage(authenticating ? messages.signingInHeader : messages.header)"
		:on-hide="handleHide"
		no-padding
		max-width="548px"
		width="100%"
	>
		<div v-if="authenticating === null" class="flex w-full flex-col gap-6 p-6">
			<div class="flex flex-col gap-2 px-3">
				<h2 class="m-0 text-xl font-semibold leading-7 text-contrast">
					{{ formatMessage(messages.signInHeading) }}
				</h2>
				<p class="m-0 text-base leading-6 text-primary">
					{{ formatMessage(messages.description) }}
				</p>
			</div>

			<div class="flex flex-col gap-6">
				<div class="grid grid-cols-1 gap-2 sm:grid-cols-2">
					<ButtonStyled>
						<button class="w-full !shadow-none" type="button" @click="authenticate('sign-up')">
							<UserPlusIcon aria-hidden="true" />
							{{ formatMessage(messages.createAccountButton) }}
						</button>
					</ButtonStyled>
					<ButtonStyled color="brand">
						<button class="w-full" type="button" @click="authenticate('sign-in')">
							<LogInIcon aria-hidden="true" />
							{{ formatMessage(messages.signInButton) }}
						</button>
					</ButtonStyled>
				</div>

				<p class="m-0 text-center text-base font-medium leading-6 text-primary">
					<IntlFormatted :message-id="messages.supportPrompt">
						<template #support="{ children }">
							<button
								type="button"
								class="inline cursor-pointer border-0 bg-transparent p-0 text-base font-medium leading-6 text-blue hover:underline"
								@click="openSupport"
							>
								<component :is="() => children" />
							</button>
						</template>
					</IntlFormatted>
				</p>
			</div>
		</div>

		<div v-else class="flex w-full flex-col gap-6 p-6">
			<div class="flex flex-col gap-2.5 px-3">
				<div class="flex flex-col gap-2">
					<h2 class="m-0 text-xl font-semibold leading-7 text-contrast">
						{{ formatMessage(messages.continueInBrowserHeading) }}
					</h2>
					<p class="m-0 text-base leading-6 text-primary">
						{{ formatMessage(messages.browserDescription) }}
					</p>
				</div>
				<div class="flex items-center gap-1.5 text-primary">
					<SpinnerIcon aria-hidden="true" class="h-5 w-5 shrink-0 animate-spin" />
					<span class="text-base leading-6">
						{{ formatMessage(messages.waitingForBrowser) }}
					</span>
				</div>
			</div>

			<div class="flex flex-col gap-6">
				<div class="grid grid-cols-1 gap-2 sm:grid-cols-2">
					<ButtonStyled type="outlined">
						<button class="w-full" type="button" @click="modal?.hide()">
							<XIcon aria-hidden="true" />
							{{ formatMessage(messages.cancelButton) }}
						</button>
					</ButtonStyled>
					<ButtonStyled>
						<button
							class="w-full !shadow-none"
							type="button"
							:disabled="reopeningBrowser"
							@click="reopenBrowser"
						>
							<RefreshCwIcon aria-hidden="true" />
							{{ formatMessage(messages.openBrowserAgainButton) }}
						</button>
					</ButtonStyled>
				</div>

				<p class="m-0 text-center text-base font-medium leading-6 text-primary">
					<IntlFormatted :message-id="messages.supportPrompt">
						<template #support="{ children }">
							<button
								type="button"
								class="inline cursor-pointer border-0 bg-transparent p-0 text-base font-medium leading-6 text-blue hover:underline"
								@click="openSupport"
							>
								<component :is="() => children" />
							</button>
						</template>
					</IntlFormatted>
				</p>
			</div>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import { LogInIcon, RefreshCwIcon, SpinnerIcon, UserPlusIcon, XIcon } from '@modrinth/assets'
import { ButtonStyled, defineMessages, IntlFormatted, NewModal, useVIntl } from '@modrinth/ui'
import { openUrl } from '@tauri-apps/plugin-opener'
import { ref } from 'vue'

import { cancelLogin, type ModrinthAuthFlow } from '@/helpers/mr_auth'

const props = defineProps<{
	requestAuth: (flow: ModrinthAuthFlow) => Promise<boolean>
}>()

const { formatMessage } = useVIntl()
const modal = ref<InstanceType<typeof NewModal>>()
const authenticating = ref<ModrinthAuthFlow | null>(null)
const reopeningBrowser = ref(false)
let resolveShow: ((signedIn: boolean) => void) | undefined
let authenticationId = 0
let activeAuthentication: Promise<void> | undefined

function show(event?: MouseEvent) {
	resetAuthentication(true)
	resolveShow?.(false)
	const modalInstance = modal.value
	if (!modalInstance) return Promise.resolve(false)

	return new Promise<boolean>((resolve) => {
		resolveShow = resolve
		modalInstance.show(event)
	})
}

function showSigningIn(flow: ModrinthAuthFlow = 'sign-in', event?: MouseEvent) {
	const result = show(event)
	authenticate(flow)
	return result
}

function finish(signedIn: boolean) {
	resolveShow?.(signedIn)
	resolveShow = undefined
}

function authenticate(flow: ModrinthAuthFlow) {
	const id = ++authenticationId
	authenticating.value = flow

	const authentication = (async () => {
		try {
			if ((await props.requestAuth(flow)) && authenticationId === id) {
				authenticating.value = null
				activeAuthentication = undefined
				finish(true)
				modal.value?.hide()
			}
		} finally {
			if (authenticationId === id) {
				authenticating.value = null
				activeAuthentication = undefined
			}
		}
	})()

	activeAuthentication = authentication
}

async function reopenBrowser() {
	const flow = authenticating.value
	if (!flow || reopeningBrowser.value) return

	reopeningBrowser.value = true
	const previousAuthentication = activeAuthentication
	++authenticationId

	try {
		await cancelLogin()
		await previousAuthentication?.catch(() => undefined)
		if (authenticating.value === flow) authenticate(flow)
	} finally {
		reopeningBrowser.value = false
	}
}

function resetAuthentication(cancelActive: boolean) {
	const wasAuthenticating = authenticating.value !== null
	++authenticationId
	activeAuthentication = undefined
	authenticating.value = null
	reopeningBrowser.value = false

	if (cancelActive && wasAuthenticating) void cancelLogin()
}

function handleHide() {
	resetAuthentication(true)
	finish(false)
}

function openSupport() {
	openUrl('https://support.modrinth.com')
}

const messages = defineMessages({
	header: {
		id: 'modal.modrinth-account-required.header',
		defaultMessage: 'Account required',
	},
	signingInHeader: {
		id: 'modal.modrinth-account-required.signing-in-header',
		defaultMessage: 'Signing in',
	},
	signInHeading: {
		id: 'modal.modrinth-account-required.sign-in-heading',
		defaultMessage: 'Sign in to a Modrinth account',
	},
	description: {
		id: 'modal.modrinth-account-required.description',
		defaultMessage:
			"You'll need to sign into your Modrinth account before you can use this feature.",
	},
	createAccountButton: {
		id: 'modal.modrinth-account-required.create-account-button',
		defaultMessage: 'Create an account',
	},
	signInButton: {
		id: 'modal.modrinth-account-required.sign-in-button',
		defaultMessage: 'Sign in to Modrinth',
	},
	continueInBrowserHeading: {
		id: 'modal.modrinth-account-required.continue-in-browser-heading',
		defaultMessage: 'Continue in your browser',
	},
	browserDescription: {
		id: 'modal.modrinth-account-required.browser-description',
		defaultMessage:
			'A new tab opened to sign in. Complete the sign in there, then return to the app.',
	},
	waitingForBrowser: {
		id: 'modal.modrinth-account-required.waiting-for-browser',
		defaultMessage: 'Waiting for browser confirmation...',
	},
	cancelButton: {
		id: 'modal.modrinth-account-required.cancel-button',
		defaultMessage: 'Cancel',
	},
	openBrowserAgainButton: {
		id: 'modal.modrinth-account-required.open-browser-again-button',
		defaultMessage: 'Open browser again',
	},
	supportPrompt: {
		id: 'modal.modrinth-account-required.support-prompt',
		defaultMessage: 'Having trouble signing in? <support>Get support</support>',
	},
})

defineExpose({ show, showSigningIn })
</script>
