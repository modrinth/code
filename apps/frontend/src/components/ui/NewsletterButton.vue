<script setup lang="ts">
import { CheckIcon, MailIcon } from '@modrinth/assets'
import { defineMessages, injectModrinthClient, useVIntl } from '@modrinth/ui'
import Button from '@modrinth/ui/src/components/base/buttons/Button.vue'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	tooltipSubscribe: {
		id: 'ui.newsletter-button.tooltip',
		defaultMessage: 'Subscribe to the Modrinth newsletter',
	},
	subscribe: {
		id: 'ui.newsletter-button.subscribe',
		defaultMessage: 'Subscribe',
	},
	subscribed: {
		id: 'ui.newsletter-button.subscribed',
		defaultMessage: 'Subscribed!',
	},
})

const auth = (await useAuth()) as unknown as {
	value: { user: { id: string; username: string; email: string; created: string } }
}
const client = injectModrinthClient()
const queryClient = useQueryClient()
const showSubscriptionConfirmation = ref(false)

const { data: showSubscribeButton, isSuccess } = useQuery({
	queryKey: computed(() => ['newsletter', 'subscribed', auth.value?.user?.id]),
	queryFn: async () => {
		if (auth.value?.user) {
			try {
				const { subscribed } = await client.labrinth.auth_internal.getNewsletterStatus()
				return !subscribed
			} catch {
				return true
			}
		} else {
			return false
		}
	},
	enabled: computed(() => !!auth.value?.user),
})

async function subscribe() {
	try {
		await client.labrinth.auth_internal.subscribeNewsletter()
		showSubscriptionConfirmation.value = true
	} catch {
		// Ignored
	} finally {
		setTimeout(() => {
			showSubscriptionConfirmation.value = false
			queryClient.setQueryData(['newsletter', 'subscribed', auth.value?.user?.id], false)
		}, 2500)
	}
}
</script>

<template>
	<Button
		v-if="isSuccess && showSubscribeButton"
		v-tooltip="formatMessage(messages.tooltipSubscribe)"
		type="outlined"
		@click="subscribe"
	>
		<template v-if="!showSubscriptionConfirmation">
			<MailIcon aria-hidden="true" />
			{{ formatMessage(messages.subscribe) }}
		</template>
		<template v-else>
			<CheckIcon aria-hidden="true" />
			{{ formatMessage(messages.subscribed) }}
		</template>
	</Button>
</template>
