<template>
	<AuthCard>
		<OpeningAuthProvider
			v-if="openingProviderName"
			:provider="openingProviderName"
			@cancel="cancelProviderOpening"
		/>
		<div v-else class="contents">
			<div class="mx-auto text-center text-2xl font-semibold text-contrast">
				{{ formatMessage(messages.signUpWithTitle) }}
			</div>
			<section class="flex flex-col gap-2.5">
				<ButtonLink
					:href="getAuthUrl('google', redirectTarget, requestsAppSession)"
					@click="onOAuthProviderClick('google', $event)"
				>
					<GoogleColorIcon />
					<span>{{ formatMessage(messages.continueWithProvider, { provider: 'Google' }) }}</span>
				</ButtonLink>
				<ButtonLink
					:href="getAuthUrl('microsoft', redirectTarget, requestsAppSession)"
					@click="onOAuthProviderClick('microsoft', $event)"
				>
					<MicrosoftColorIcon />
					<span>{{ formatMessage(messages.continueWithProvider, { provider: 'Microsoft' }) }}</span>
				</ButtonLink>
				<ButtonLink
					:href="getAuthUrl('discord', redirectTarget, requestsAppSession)"
					@click="onOAuthProviderClick('discord', $event)"
				>
					<DiscordColorIcon />
					<span>{{ formatMessage(messages.continueWithProvider, { provider: 'Discord' }) }}</span>
				</ButtonLink>
				<template v-if="showOtherOptions">
					<ButtonLink
						:href="getAuthUrl('github', redirectTarget, requestsAppSession)"
						@click="onOAuthProviderClick('github', $event)"
					>
						<GitHubColorIcon />
						<span>{{ formatMessage(messages.continueWithProvider, { provider: 'GitHub' }) }}</span>
					</ButtonLink>
					<ButtonLink
						:href="getAuthUrl('gitlab', redirectTarget, requestsAppSession)"
						@click="onOAuthProviderClick('gitlab', $event)"
					>
						<GitLabColorIcon />
						<span>{{ formatMessage(messages.continueWithProvider, { provider: 'GitLab' }) }}</span>
					</ButtonLink>
					<ButtonLink
						:href="getAuthUrl('steam', redirectTarget, requestsAppSession)"
						@click="onOAuthProviderClick('steam', $event)"
					>
						<SteamColorIcon />
						<span>{{ formatMessage(messages.continueWithProvider, { provider: 'Steam' }) }}</span>
					</ButtonLink>
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
				<Input
					id="email"
					v-model="emailModel"
					:icon="MailIcon"
					type="email"
					autocomplete="email"
					:placeholder="formatMessage(commonMessages.emailLabel)"
					wrapper-class="w-full"
				/>

				<label for="password" hidden>{{ formatMessage(commonMessages.passwordLabel) }}</label>
				<Input
					id="password"
					v-model="passwordModel"
					:icon="KeyIcon"
					type="password"
					autocomplete="new-password"
					:placeholder="formatMessage(commonMessages.passwordLabel)"
					wrapper-class="w-full"
				/>

				<Button
					type="colored"
					color="brand"
					class="!w-full"
					:disabled="!emailModel || !passwordModel"
					@click="onContinueWithEmail()"
				>
					{{ formatMessage(messages.continueWithEmail) }} <RightArrowIcon />
				</Button>

				<p v-if="!routeQuery.launcher" class="m-0 text-center">
					<IntlFormatted :message-id="messages.legalDisclaimer">
						<template #terms-link="{ children }">
							<NuxtLink to="/legal/terms" class="text-link" target="_blank" rel="noopener">
								<component :is="() => children" />
							</NuxtLink>
						</template>
						<template #privacy-policy-link="{ children }">
							<NuxtLink to="/legal/privacy" class="text-link" target="_blank" rel="noopener">
								<component :is="() => children" />
							</NuxtLink>
						</template>
					</IntlFormatted>
				</p>

				<div class="mx-auto flex flex-wrap items-center justify-start gap-2 text-center">
					{{ formatMessage(messages.alreadyHaveAccountLabel) }}
					<button v-if="embedded" type="button" class="mr-1 text-link" @click="openSignIn">
						{{ formatMessage(commonMessages.signInButton) }}
					</button>
					<NuxtLink
						v-else
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
	</AuthCard>
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
	Button,
	ButtonLink,
	commonMessages,
	defineMessages,
	Input,
	IntlFormatted,
	useVIntl,
} from '@modrinth/ui'
import type { LocationQuery } from 'vue-router'

import AuthCard from '@/components/ui/auth/AuthCard.vue'
import OpeningAuthProvider from '@/components/ui/auth/OpeningAuthProvider.vue'
import { getAuthUrl } from '@/composables/auth.ts'
import { isAuthEmbedded, useAuthNavigation } from '@/composables/auth-modal.ts'
import { useProviderOpening } from '@/composables/external-auth.ts'
import { isLauncherProtocolV2 } from '@/composables/launcher-auth.ts'
import type { OAuthProviderId } from '@/composables/oauth-providers.ts'

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
const requestsAppSession = computed(() => isLauncherProtocolV2({ query: routeQuery }))

const { openingProviderName, beginProviderHandoff, cancelProviderOpening } = useProviderOpening()
const { go } = useAuthNavigation()
const embedded = isAuthEmbedded()

function openSignIn() {
	void go({
		path: '/auth/sign-in',
		query: routeQuery,
	})
}

const onOAuthProviderClick = (provider: OAuthProviderId, event: MouseEvent) => {
	beginProviderHandoff(
		provider,
		event,
		getAuthUrl(provider, redirectTarget, requestsAppSession.value),
		{ skip: Boolean(routeQuery.launcher || requestsAppSession.value) },
	)
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
