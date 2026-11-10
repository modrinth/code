<template>
	<div class="flex flex-col items-center gap-6 text-center" role="status">
		<div class="flex flex-col gap-2">
			<h2 class="m-0 text-2xl font-semibold text-contrast">
				{{ formatMessage(messages.openingTitle, { provider }) }}
			</h2>
			<p class="m-0 text-primary">
				{{ formatMessage(messages.openingDescription, { provider }) }}
			</p>
		</div>
		<Button class="!w-full !justify-center" @click="emit('cancel')">
			{{ formatMessage(messages.signInAnotherWay) }}
		</Button>
	</div>
</template>

<script setup lang="ts">
import { Button, defineMessages, useVIntl } from '@modrinth/ui'

defineProps<{
	provider: string
}>()

const emit = defineEmits<{
	cancel: []
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	openingTitle: {
		id: 'auth.oauth.opening.title',
		defaultMessage: 'Opening {provider}...',
	},
	openingDescription: {
		id: 'auth.oauth.opening.description',
		defaultMessage: 'Complete your sign in on {provider}',
	},
	signInAnotherWay: {
		id: 'auth.reauthenticate.sign-in-another-way',
		defaultMessage: 'Sign in another way',
	},
})
</script>
