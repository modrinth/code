<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.header)"
		:on-hide="() => finish(false)"
		no-padding
		max-width="548px"
		width="100%"
	>
		<div class="flex w-full flex-col gap-6 p-6">
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
						<button
							class="w-full !shadow-none"
							type="button"
							:disabled="authenticating !== null"
							@click="authenticate('sign-up')"
						>
							<SpinnerIcon
								v-if="authenticating === 'sign-up'"
								aria-hidden="true"
								class="animate-spin"
							/>
							<UserPlusIcon v-else aria-hidden="true" />
							{{ formatMessage(messages.createAccountButton) }}
						</button>
					</ButtonStyled>
					<ButtonStyled color="brand">
						<button
							class="w-full"
							type="button"
							:disabled="authenticating !== null"
							@click="authenticate('sign-in')"
						>
							<SpinnerIcon
								v-if="authenticating === 'sign-in'"
								aria-hidden="true"
								class="animate-spin"
							/>
							<LogInIcon v-else aria-hidden="true" />
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
	</NewModal>
</template>

<script setup lang="ts">
import { LogInIcon, SpinnerIcon, UserPlusIcon } from '@modrinth/assets'
import { ButtonStyled, defineMessages, IntlFormatted, NewModal, useVIntl } from '@modrinth/ui'
import { openUrl } from '@tauri-apps/plugin-opener'
import { ref } from 'vue'

import type { ModrinthAuthFlow } from '@/helpers/mr_auth'

const props = defineProps<{
	requestAuth: (flow: ModrinthAuthFlow) => Promise<boolean>
}>()

const { formatMessage } = useVIntl()
const modal = ref<InstanceType<typeof NewModal>>()
const authenticating = ref<ModrinthAuthFlow | null>(null)
let resolveShow: ((signedIn: boolean) => void) | undefined

function show(event?: MouseEvent) {
	resolveShow?.(false)
	const modalInstance = modal.value
	if (!modalInstance) return Promise.resolve(false)

	return new Promise<boolean>((resolve) => {
		resolveShow = resolve
		modalInstance.show(event)
	})
}

function finish(signedIn: boolean) {
	resolveShow?.(signedIn)
	resolveShow = undefined
}

async function authenticate(flow: ModrinthAuthFlow) {
	authenticating.value = flow
	try {
		if (await props.requestAuth(flow)) {
			finish(true)
			modal.value?.hide()
		}
	} finally {
		authenticating.value = null
	}
}

function openSupport() {
	openUrl('https://support.modrinth.com')
}

const messages = defineMessages({
	header: {
		id: 'modal.modrinth-account-required.header',
		defaultMessage: 'Account required',
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
	supportPrompt: {
		id: 'modal.modrinth-account-required.support-prompt',
		defaultMessage: 'Having trouble signing in? <support>Get support</support>',
	},
})

defineExpose({ show })
</script>
