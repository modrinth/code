<script setup lang="ts">
import { CheckIcon, MailIcon } from '@modrinth/assets'
import { ButtonStyled, defineMessages, useVIntl } from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import { useBaseFetch } from '~/composables/fetch.js'

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
const queryClient = useQueryClient()
const showSubscriptionConfirmation = ref(false)

const { data: showSubscribeButton, isSuccess } = useQuery({
	queryKey: computed(() => ['newsletter', 'subscribed', auth.value.user.id]),
	queryFn: async () => {
		if (auth.value?.user) {
			try {
				const { subscribed } = (await useBaseFetch('auth/email/subscribe', {
					method: 'GET',
				})) as { subscribed: boolean }
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
		await useBaseFetch('auth/email/subscribe', {
			method: 'POST',
		})
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
	<ButtonStyled v-if="isSuccess && showSubscribeButton" color="brand" type="outlined">
		<button v-tooltip="formatMessage(messages.tooltipSubscribe)" @click="subscribe">
			<template v-if="!showSubscriptionConfirmation">
				<MailIcon /> {{ formatMessage(messages.subscribe) }}
			</template>
			<template v-else> <CheckIcon /> {{ formatMessage(messages.subscribed) }} </template>
		</button>
	</ButtonStyled>
</template>
