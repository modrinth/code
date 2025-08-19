<script setup lang="ts">
import { CheckIcon, MailIcon } from '@modrinth/assets'
import { ButtonStyled } from '@modrinth/ui'
import { ref } from 'vue'

import { useBaseFetch } from '~/composables/fetch.js'

const auth = await useAuth()
const showSubscriptionConfirmation = ref(false)
const showSubscribeButton = useAsyncData(
	async () => {
		if (auth.value?.user) {
			try {
				const { subscribed } = await useBaseFetch('auth/email/subscribe', {
					method: 'GET',
				})
				return !subscribed
			} catch {
				return true
			}
		} else {
			return false
		}
	},
	{ watch: [auth], server: false },
)

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
			showSubscribeButton.status.value = 'success'
			showSubscribeButton.data.value = false
		}, 2500)
	}
}
</script>

<template>
	<ButtonStyled
		v-if="showSubscribeButton.status.value === 'success' && showSubscribeButton.data.value"
		color="brand"
		type="outlined"
	>
		<button v-tooltip="`Subscribe to the Modrinth newsletter`" @click="subscribe">
			<template v-if="!showSubscriptionConfirmation"> <MailIcon /> Subscribe </template>
			<template v-else> <CheckIcon /> Subscribed! </template>
		</button>
	</ButtonStyled>
</template>
