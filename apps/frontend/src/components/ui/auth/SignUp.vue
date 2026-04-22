<template>
	<div class="flex flex-col gap-6">
		<div class="text-center text-2xl font-semibold text-contrast">
			{{ formatMessage(messages.signUpWithTitle) }}
		</div>
		<section class="flex flex-col gap-2.5">
			<ButtonStyled>
				<a class="!shadow-none" :href="getAuthUrl('google', redirectTarget)">
					<GoogleColorIcon />
					<span>{{ formatMessage(messages.continueWithProvider, { provider: 'Google' }) }}</span>
				</a>
			</ButtonStyled>
			<ButtonStyled>
				<a class="!shadow-none" :href="getAuthUrl('microsoft', redirectTarget)">
					<MicrosoftColorIcon />
					<span>{{ formatMessage(messages.continueWithProvider, { provider: 'Microsoft' }) }}</span>
				</a>
			</ButtonStyled>
			<ButtonStyled>
				<a class="!shadow-none" :href="getAuthUrl('discord', redirectTarget)">
					<DiscordColorIcon />
					<span>{{ formatMessage(messages.continueWithProvider, { provider: 'Discord' }) }}</span>
				</a>
			</ButtonStyled>
			<template v-if="showOtherOptions">
				<ButtonStyled>
					<a class="!shadow-none" :href="getAuthUrl('github', redirectTarget)">
						<GitHubColorIcon />
						<span>{{ formatMessage(messages.continueWithProvider, { provider: 'GitHub' }) }}</span>
					</a>
				</ButtonStyled>
				<ButtonStyled>
					<a class="!shadow-none" :href="getAuthUrl('gitlab', redirectTarget)">
						<GitLabColorIcon />
						<span>{{ formatMessage(messages.continueWithProvider, { provider: 'GitLab' }) }}</span>
					</a>
				</ButtonStyled>
				<ButtonStyled>
					<a class="!shadow-none" :href="getAuthUrl('steam', redirectTarget)">
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

			<p v-if="!routeQuery.launcher">
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

			<ButtonStyled color="brand">
				<button
					class="!w-full"
					:disabled="!emailModel || !passwordModel"
					@click="onContinueWithEmail()"
				>
					{{ formatMessage(messages.continueWithEmail) }} <RightArrowIcon />
				</button>
			</ButtonStyled>

			<div class="auth-form__additional-options">
				{{ formatMessage(messages.alreadyHaveAccountLabel) }}
				<NuxtLink
					class="text-link"
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

<script setup>
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
import { computed } from 'vue'

import { getAuthUrl } from '@/composables/auth.ts'

const props = defineProps({
	redirectTarget: {
		type: String,
		default: '',
	},
	showOtherOptions: {
		type: Boolean,
		default: false,
	},
	routeQuery: {
		type: Object,
		default: () => ({}),
	},
	email: {
		type: String,
		default: '',
	},
	password: {
		type: String,
		default: '',
	},
	onToggleOtherOptions: {
		type: Function,
		default: () => {},
	},
	onContinueWithEmail: {
		type: Function,
		default: () => {},
	},
})

const emit = defineEmits(['update:email', 'update:password'])

const emailModel = computed({
	get: () => props.email,
	set: (value) => emit('update:email', value),
})

const passwordModel = computed({
	get: () => props.password,
	set: (value) => emit('update:password', value),
})

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
