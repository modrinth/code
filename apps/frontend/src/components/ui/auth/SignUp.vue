<template>
	<div
		class="universal-card flex w-full max-w-[28rem] flex-col gap-6 border border-solid border-surface-5"
	>
		<div class="mx-auto text-center text-2xl font-semibold text-contrast">
			{{ formatMessage(messages.signUpWithTitle) }}
		</div>
		<section class="flex flex-col gap-2.5">
			<ButtonStyled>
				<a
					class="!shadow-none"
					:href="getAuthUrl('google', redirectTarget)"
					@click="onOAuthProviderClick('google')"
				>
					<GoogleColorIcon />
					<span>{{ formatMessage(messages.continueWithProvider, { provider: 'Google' }) }}</span>
				</a>
			</ButtonStyled>
			<ButtonStyled>
				<a
					class="!shadow-none"
					:href="getAuthUrl('microsoft', redirectTarget)"
					@click="onOAuthProviderClick('microsoft')"
				>
					<MicrosoftColorIcon />
					<span>{{ formatMessage(messages.continueWithProvider, { provider: 'Microsoft' }) }}</span>
				</a>
			</ButtonStyled>
			<ButtonStyled>
				<a
					class="!shadow-none"
					:href="getAuthUrl('discord', redirectTarget)"
					@click="onOAuthProviderClick('discord')"
				>
					<DiscordColorIcon />
					<span>{{ formatMessage(messages.continueWithProvider, { provider: 'Discord' }) }}</span>
				</a>
			</ButtonStyled>
			<template v-if="showOtherOptions">
				<ButtonStyled>
					<a
						class="!shadow-none"
						:href="getAuthUrl('github', redirectTarget)"
						@click="onOAuthProviderClick('github')"
					>
						<GitHubColorIcon />
						<span>{{ formatMessage(messages.continueWithProvider, { provider: 'GitHub' }) }}</span>
					</a>
				</ButtonStyled>
				<ButtonStyled>
					<a
						class="!shadow-none"
						:href="getAuthUrl('gitlab', redirectTarget)"
						@click="onOAuthProviderClick('gitlab')"
					>
						<GitLabColorIcon />
						<span>{{ formatMessage(messages.continueWithProvider, { provider: 'GitLab' }) }}</span>
					</a>
				</ButtonStyled>
				<ButtonStyled>
					<a
						class="!shadow-none"
						:href="getAuthUrl('steam', redirectTarget)"
						@click="onOAuthProviderClick('steam')"
					>
						<SteamColorIcon />
						<span>{{ formatMessage(messages.continueWithProvider, { provider: 'Steam' }) }}</span>
					</a>
				</ButtonStyled>
			</template>
			<button
				class="mx-auto -mb-3 bg-transparent pt-1 text-center text-base font-semibold text-secondary transition-all hover:text-primary"
				@click="onToggleOtherOptions()"
			>
				{{
					showOtherOptions
						? formatMessage(messages.showFewerOptions)
						: formatMessage(messages.showOtherOptions)
				}}
			</button>
		</section>

		<div class="h-px w-full bg-surface-5"></div>

		<section class="flex flex-col gap-2.5">
			<label for="email" hidden>{{ formatMessage(commonMessages.emailLabel) }}</label>
			<StyledInput
				id="email"
				v-model="emailModel"
				:icon="MailIcon"
				type="email"
				autocomplete="email"
				:placeholder="formatMessage(commonMessages.emailLabel)"
				wrapper-class="w-full"
			/>

			<label for="password" hidden>{{ formatMessage(commonMessages.passwordLabel) }}</label>
			<StyledInput
				id="password"
				v-model="passwordModel"
				:icon="KeyIcon"
				type="password"
				autocomplete="new-password"
				:placeholder="formatMessage(commonMessages.passwordLabel)"
				wrapper-class="w-full"
			/>

			<ButtonStyled color="brand">
				<button
					class="!w-full"
					:disabled="!emailModel || !passwordModel"
					@click="onContinueWithEmail()"
				>
					{{ formatMessage(messages.continueWithEmail) }} <RightArrowIcon />
				</button>
			</ButtonStyled>

			<p v-if="!routeQuery.launcher" class="m-0 text-center">
				<IntlFormatted :message-id="messages.legalDisclaimer">
					<template #terms-link="{ children }">
						<NuxtLink to="/legal/terms" class="text-link">
							<component :is="() => children" />
						</NuxtLink>
					</template>
					<template #privacy-policy-link="{ children }">
						<NuxtLink to="/legal/privacy" class="text-link">
							<component :is="() => children" />
						</NuxtLink>
					</template>
				</IntlFormatted>
			</p>

			<div class="mx-auto flex flex-wrap items-center justify-start gap-2 text-center">
				{{ formatMessage(messages.alreadyHaveAccountLabel) }}
				<NuxtLink
					class="mr-1 text-link"
					:to="{
						path: '/auth/sign-in',
						query: routeQuery,
					}"
				>
					{{ formatMessage(commonMessages.signInButton) }}
				</NuxtLink>
			</div>
		</section>
	</div>
</template>

<script setup lang="ts">
import {
	DiscordColorIcon,
	GitHubColorIcon,
	GitLabColorIcon,
	GoogleColorIcon,
	KeyIcon,
	MailIcon,
	MicrosoftColorIcon,
	RightArrowIcon,
	SteamColorIcon,
} from '@modrinth/assets'
import {
	ButtonStyled,
	commonMessages,
	defineMessages,
	IntlFormatted,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'
import { useStorage } from '@vueuse/core'
import type { LocationQuery } from 'vue-router'

import { getAuthUrl, PENDING_SIGN_IN_OAUTH_PROVIDER_STORAGE_KEY } from '@/composables/auth.ts'

type AuthProvider = 'discord' | 'google' | 'github' | 'gitlab' | 'steam' | 'microsoft'

interface Props {
	redirectTarget?: string
	showOtherOptions?: boolean
	routeQuery?: LocationQuery
	onToggleOtherOptions?: () => void
	onContinueWithEmail?: () => void
}

const {
	redirectTarget = '',
	showOtherOptions = false,
	routeQuery = {},
	onToggleOtherOptions = () => {},
	onContinueWithEmail = () => {},
} = defineProps<Props>()

const emailModel = defineModel<string>('email', { default: '' })
const passwordModel = defineModel<string>('password', { default: '' })

const pendingSignInOAuthProvider = useStorage<AuthProvider | null>(
	PENDING_SIGN_IN_OAUTH_PROVIDER_STORAGE_KEY,
	null,
)
const onOAuthProviderClick = (provider: AuthProvider) => {
	pendingSignInOAuthProvider.value = provider
}

const { formatMessage } = useVIntl()

const messages = defineMessages({
	signUpWithTitle: {
		id: 'auth.sign-up.title.sign-up-with',
		defaultMessage: 'Create an Account',
	},
	continueWithProvider: {
		id: 'auth.continue-with-provider',
		defaultMessage: 'Continue with {provider}',
	},
	legalDisclaimer: {
		id: 'auth.sign-up.legal-dislaimer',
		defaultMessage:
			"By creating an account, you agree to Modrinth's <terms-link>Terms</terms-link> and <privacy-policy-link>Privacy Policy</privacy-policy-link>.",
	},
	alreadyHaveAccountLabel: {
		id: 'auth.sign-up.sign-in-option.title',
		defaultMessage: 'Already have an account?',
	},
	continueWithEmail: {
		id: 'auth.sign-up.continue-with-email',
		defaultMessage: 'Continue with Email',
	},
	showFewerOptions: {
		id: 'auth.sign-up.show-fewer-options',
		defaultMessage: 'Show fewer options',
	},
	showOtherOptions: {
		id: 'auth.sign-up.show-other-options',
		defaultMessage: 'Show other options',
	},
})
</script>
