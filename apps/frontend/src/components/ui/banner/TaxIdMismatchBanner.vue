<script setup lang="ts">
import { MessageIcon } from '@modrinth/assets'
import { ButtonLink, defineMessages, PagewideBanner, useVIntl } from '@modrinth/ui'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'layout.banner.tin-mismatch.title',
		defaultMessage: 'Tax form failed',
	},
	description: {
		id: 'layout.banner.tin-mismatch.description',
		defaultMessage:
			"Your withdrawals are temporarily locked because your TIN or SSN didn't match IRS records. Please contact support to reset and resubmit your tax form.",
	},
	action: {
		id: 'layout.banner.tin-mismatch.action',
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
			<span>{{ formatMessage(messages.description) }}</span>
		</template>
		<template #actions_right>
			<div class="flex w-fit flex-row">
				<ButtonLink
					href="https://support.modrinth.com"
					target="_blank"
					rel="noopener"
					type="colored"
					color="red"
				>
					<MessageIcon aria-hidden="true" />
					{{ formatMessage(messages.action) }}
				</ButtonLink>
			</div>
		</template>
	</PagewideBanner>
</template>
