<template>
	<div>
		<h2 class="m-0 mb-4 text-2xl font-semibold">User lookup</h2>
		<form
			v-if="isAdmin(auth.user)"
			class="card flex flex-col gap-3"
			@submit.prevent="lookupUser('email')"
		>
			<div class="flex flex-col gap-2">
				<label for="user-email">
					<span class="text-lg font-semibold text-contrast">
						User email
						<span class="text-brand-red">*</span>
					</span>
				</label>
				<Input
					id="user-email"
					v-model="userEmail"
					type="email"
					:maxlength="64"
					placeholder="Enter user email..."
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
					Get user account
				</Button>
			</div>
		</form>
		<form class="card flex flex-col gap-3" @submit.prevent="lookupUser('discord')">
			<div class="flex flex-col gap-2">
				<label for="discord-id">
					<span class="text-lg font-semibold text-contrast">
						Discord ID
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
					placeholder="Enter Discord ID..."
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
					Get user account
				</Button>
			</div>
		</form>
	</div>
</template>
<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { DiscordIcon, MailIcon } from '@modrinth/assets'
import { Button, injectNotificationManager, Input } from '@modrinth/ui'
import { isAdmin } from '@modrinth/utils'
import { useQuery } from '@tanstack/vue-query'

const { addNotification } = injectNotificationManager()
const auth = await useAuth()

const userEmail = ref('')
const discordId = ref('')

const { refetch: lookupEmail, isFetching: isFetchingEmail } = useQuery({
	queryKey: computed(() => ['users', 'lookup', 'email', userEmail.value.trim()]),
	queryFn: async () =>
		(await useBaseFetch('user_email', {
			apiVersion: 3,
			query: { email: userEmail.value.trim() },
		})) as Labrinth.Users.v3.User,
	enabled: false,
	retry: false,
})

const { refetch: lookupDiscord, isFetching: isFetchingDiscord } = useQuery({
	queryKey: computed(() => ['users', 'lookup', 'discord', discordId.value.trim()]),
	queryFn: async () =>
		(await useBaseFetch('user_discord', {
			apiVersion: 3,
			query: { discord_id: discordId.value.trim() },
		})) as Labrinth.Users.v3.User,
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
			title: 'User lookup failed',
			text: err instanceof Error ? err.message : 'User lookup failed',
			type: 'error',
		})
	} finally {
		stopLoading()
	}
}
</script>
