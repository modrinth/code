<script setup lang="ts">
import { defineMessages, PagewideBanner, useVIntl } from '@modrinth/ui'

const { formatMessage } = useVIntl()

defineProps<{
	reason: string
}>()

const messages = defineMessages({
	title: {
		id: 'layout.banner.account-locked.title',
		defaultMessage: 'Your account has been locked',
	},
	description: {
		id: 'layout.banner.account-locked.description',
		defaultMessage:
			'A moderator has locked your account, so you cannot create, edit, or delete anything on Modrinth. Reason: {reason}',
	},
	support: {
		id: 'layout.banner.account-locked.support',
		defaultMessage: 'Contact support',
	},
})
</script>

<template>
	<PagewideBanner variant="error">
		<template #title>
			<span>{{ formatMessage(messages.title) }}</span>
		</template>
		<template #description>
			<span>{{ formatMessage(messages.description, { reason }) }}</span>
		</template>
		<template #actions_right>
			<a
				href="https://support.modrinth.com"
				class="text-link"
				target="_blank"
				rel="noopener noreferrer"
			>
				{{ formatMessage(messages.support) }}
			</a>
		</template>
	</PagewideBanner>
</template>
