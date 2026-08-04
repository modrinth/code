<template>
	<Admonition type="warning" :header="formatMessage(headerMessage)">
		<span class="flex flex-wrap items-center gap-x-1.5 gap-y-1">
			<span>{{ formatMessage(messages.sharedInstanceWrongAccountSignInAs) }}</span>
			<span
				v-if="sharedInstanceExpectedUser"
				class="inline-flex max-w-full min-w-0 items-center gap-1.5 align-middle font-semibold text-contrast"
			>
				<Avatar
					:src="sharedInstanceExpectedUser.avatarUrl"
					:alt="sharedInstanceExpectedUsername"
					:tint-by="sharedInstanceExpectedUser.tintBy"
					size="20px"
					circle
					no-shadow
				/>
				<span class="min-w-0 truncate">{{ sharedInstanceExpectedUsername }}</span>
			</span>
			<span v-else class="font-semibold">{{ sharedInstanceExpectedUsername }}</span>
			<span>{{ formatMessage(bodyMessage) }}</span>
		</span>
	</Admonition>
</template>

<script setup lang="ts">
import { Admonition, Avatar, useVIntl } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'

import { get_user } from '@/helpers/cache'

import { instanceAdmonitionsMessages as messages } from './instance-admonitions-messages'
import type { SharedInstanceRole } from './types'

const props = defineProps<{
	expectedUserId?: string | null
	role?: SharedInstanceRole | null
	signedOut?: boolean
}>()

const { formatMessage } = useVIntl()
const expectedUserId = computed(() => props.expectedUserId ?? null)
const expectedUserQuery = useQuery({
	queryKey: computed(() => ['user', expectedUserId.value]),
	queryFn: async () => {
		if (!expectedUserId.value) return null

		return await get_user(expectedUserId.value, 'bypass').catch(() => null)
	},
	enabled: () => !!expectedUserId.value,
	staleTime: 30_000,
})
const sharedInstanceExpectedUser = computed(() => {
	const user = expectedUserQuery.data.value
	if (!user) return null

	return {
		username: user.username,
		avatarUrl: user.avatar_url ?? undefined,
		tintBy: user.id,
	}
})
const sharedInstanceExpectedUsername = computed(
	() =>
		sharedInstanceExpectedUser.value?.username ||
		formatMessage(messages.sharedInstanceWrongAccountFallbackUsername),
)
const headerMessage = computed(() =>
	props.signedOut
		? messages.sharedInstanceSignedOutHeader
		: messages.sharedInstanceWrongAccountHeader,
)
const bodyMessage = computed(() =>
	props.role === 'owner'
		? messages.sharedInstanceWrongAccountOwnerBody
		: messages.sharedInstanceWrongAccountUserBody,
)
</script>
