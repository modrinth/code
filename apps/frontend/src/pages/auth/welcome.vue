<template>
	<div class="relative flex flex-col gap-4 pt-12 sm:pt-10">
		<img
			:src="WavingRinthbot"
			alt="Waving Modrinth Bot"
			class="absolute right-8 top-0 h-[70px] w-auto sm:right-20 sm:h-[112px]"
		/>

		<h1 class="m-0 pr-24 text-3xl leading-tight">
			{{ formatMessage(messages.welcomeLongTitle) }}
		</h1>

		<p class="text-base leading-snug">
			<IntlFormatted :message-id="messages.welcomeDescription">
				<template #bold="{ children }">
					<strong>
						<component :is="() => normalizeChildren(children)" />
					</strong>
				</template>
			</IntlFormatted>
		</p>

		<Checkbox
			v-model="subscribe"
			class="subscribe-btn"
			:label="formatMessage(messages.subscribeCheckbox)"
			:description="formatMessage(messages.subscribeCheckbox)"
		/>

		<button class="btn btn-primary centered-btn" @click="continueSignUp">
			{{ formatMessage(commonMessages.continueButton) }}
			<RightArrowIcon />
		</button>

		<p class="text-sm leading-normal">
			<IntlFormatted :message-id="messages.tosLabel">
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
	</div>
</template>

<script setup>
import { RightArrowIcon, WavingRinthbot } from '@modrinth/assets'
import {
	Checkbox,
	commonMessages,
	defineMessages,
	IntlFormatted,
	normalizeChildren,
	useVIntl,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'

const queryClient = useQueryClient()
const route = useRoute()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	subscribeCheckbox: {
		id: 'auth.welcome.checkbox.subscribe',
		defaultMessage: 'Subscribe to updates about Modrinth',
	},
	tosLabel: {
		id: 'auth.welcome.label.tos',
		defaultMessage:
			"By creating an account, you have agreed to Modrinth's <terms-link>Terms</terms-link> and <privacy-policy-link>Privacy Policy</privacy-policy-link>.",
	},
	welcomeDescription: {
		id: 'auth.welcome.description',
		defaultMessage:
			'You’re now part of the awesome community of creators & explorers already building, downloading, and staying up-to-date with amazing mods.',
	},
	welcomeLongTitle: {
		id: 'auth.welcome.long-title',
		defaultMessage: 'Welcome to Modrinth!',
	},
	welcomeTitle: {
		id: 'auth.welcome.title',
		defaultMessage: 'Welcome',
	},
})

useHead({
	title: () => `${formatMessage(messages.welcomeTitle)} - Modrinth`,
})

const subscribe = ref(true)

onMounted(async () => {
	await useAuth(route.query.authToken)
	await useUser()
	queryClient.clear()
})

async function continueSignUp() {
	if (subscribe.value) {
		try {
			await useBaseFetch('auth/email/subscribe', {
				method: 'POST',
			})
		} catch {
			// Ignored
		}
	}

	if (route.query.redirect) {
		await navigateTo(route.query.redirect)
	} else {
		await navigateTo('/dashboard')
	}
}
</script>
