<template>
	<div>
		<h2 class="m-0 mb-4 text-2xl font-semibold">{{ formatMessage(messages.title) }}</h2>
		<form
			v-if="isAdmin(auth.user)"
			class="card flex flex-col gap-3"
			@submit.prevent="lookupUser('email')"
		>
			<div class="flex flex-col gap-2">
				<label for="user-email">
					<span class="text-lg font-semibold text-contrast">
						{{ formatMessage(messages.email) }}
						<span class="text-brand-red">*</span>
					</span>
				</label>
				<Input
					id="user-email"
					v-model="userEmail"
					type="email"
					:maxlength="64"
					:placeholder="formatMessage(messages.emailPlaceholder)"
					:disabled="isLookingUp"
					autocomplete="off"
					required
				/>
			</div>
			<div class="flex gap-2">
				<Button
					type="colored"
					color="brand"
					native-type="submit"
					:disabled="!userEmail.trim() || isLookingUp"
					:loading="isFetchingEmail"
				>
					<MailIcon aria-hidden="true" />
					{{ formatMessage(messages.getAccount) }}
				</Button>
			</div>
		</form>
		<form class="card flex flex-col gap-3" @submit.prevent="lookupUser('discord')">
			<div class="flex flex-col gap-2">
				<label for="discord-id">
					<span class="text-lg font-semibold text-contrast">
						{{ formatMessage(messages.discordId) }}
						<span class="text-brand-red">*</span>
					</span>
				</label>
				<Input
					id="discord-id"
					v-model="discordId"
					type="text"
					inputmode="numeric"
					pattern="[0-9]+"
					:maxlength="19"
					:placeholder="formatMessage(messages.discordPlaceholder)"
					:disabled="isLookingUp"
					autocomplete="off"
					required
				/>
			</div>
			<div class="flex gap-2">
				<Button
					type="colored"
					color="brand"
					native-type="submit"
					:disabled="!discordId.trim() || isLookingUp"
					:loading="isFetchingDiscord"
				>
					<DiscordIcon aria-hidden="true" />
					{{ formatMessage(messages.getAccount) }}
				</Button>
			</div>
		</form>
	</div>
</template>
<script setup lang="ts">
import { DiscordIcon, MailIcon } from '@modrinth/assets'
import {
	Button,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	Input,
	useVIntl,
} from '@modrinth/ui'
import { isAdmin } from '@modrinth/utils'
import { useQuery } from '@tanstack/vue-query'

const { labrinth } = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()
const auth = await useAuth()

const userEmail = ref('')
const discordId = ref('')

const { refetch: lookupEmail, isFetching: isFetchingEmail } = useQuery({
	queryKey: computed(() => ['users', 'lookup', 'email', userEmail.value.trim()]),
	queryFn: () => labrinth.users_v3.getByEmail(userEmail.value.trim()),
	enabled: false,
	retry: false,
})

const { refetch: lookupDiscord, isFetching: isFetchingDiscord } = useQuery({
	queryKey: computed(() => ['users', 'lookup', 'discord', discordId.value.trim()]),
	queryFn: () => labrinth.users_v3.getByDiscordId(discordId.value.trim()),
	enabled: false,
	retry: false,
})

const isLookingUp = computed(() => isFetchingEmail.value || isFetchingDiscord.value)

async function lookupUser(kind: 'email' | 'discord') {
	if (isLookingUp.value || (kind === 'email' && !isAdmin(auth.value.user))) return

	startLoading()

	try {
		const lookup = kind === 'email' ? lookupEmail : lookupDiscord
		const { data } = await lookup({ throwOnError: true })

		if (data) await navigateTo(`/user/${encodeURIComponent(data.username)}`)
	} catch (err) {
		console.error(err)
		addNotification({
			title: formatMessage(messages.lookupFailed),
			text: err instanceof Error ? err.message : formatMessage(messages.lookupFailed),
			type: 'error',
		})
	} finally {
		stopLoading()
	}
}

const messages = defineMessages({
	title: { id: 'admin.user-lookup.title', defaultMessage: 'User lookup' },
	email: { id: 'admin.user-lookup.email', defaultMessage: 'User email' },
	emailPlaceholder: {
		id: 'admin.user-lookup.email-placeholder',
		defaultMessage: 'Enter user email...',
	},
	discordId: { id: 'admin.user-lookup.discord-id', defaultMessage: 'Discord ID' },
	discordPlaceholder: {
		id: 'admin.user-lookup.discord-placeholder',
		defaultMessage: 'Enter Discord ID...',
	},
	getAccount: { id: 'admin.user-lookup.get-account', defaultMessage: 'Get user account' },
	lookupFailed: { id: 'admin.user-lookup.failed', defaultMessage: 'User lookup failed' },
})
</script>
