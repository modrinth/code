<template>
	<div>
		<EmptyState :icon="FileIcon" :heading="title" :description="message" hide-pattern>
			<template #actions>
				<Button type="outlined" size="lg" @click="$emit('home')">
					<HomeIcon />
					{{ formatMessage(messages.goToHome) }}
				</Button>
				<Button type="colored" color="brand" size="lg" @click="$emit('refetch')">
					<RefreshCwIcon />
					{{ formatMessage(messages.tryAgain) }}
				</Button>
			</template>
		</EmptyState>
	</div>
</template>

<script setup lang="ts">
import { FileIcon, HomeIcon, RefreshCwIcon } from '@modrinth/assets'

import { Button } from '#ui/components/base/buttons'
import EmptyState from '#ui/components/base/EmptyState.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	tryAgain: {
		id: 'files.error.try-again',
		defaultMessage: 'Try again',
	},
	goToHome: {
		id: 'files.error.go-to-home',
		defaultMessage: 'Go to home folder',
	},
})

defineProps<{
	title: string
	message: string
}>()

defineEmits<{
	refetch: []
	home: []
}>()
</script>
