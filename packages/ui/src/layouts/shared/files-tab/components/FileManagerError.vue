<template>
	<div class="flex h-full w-full items-center justify-center gap-6 p-20">
		<FileIcon class="size-28" />
		<div class="flex flex-col gap-2">
			<h3 class="m-0 text-2xl font-bold text-red">{{ title }}</h3>
			<p class="m-0 text-sm text-secondary">
				{{ message }}
			</p>
			<div class="flex gap-2">
				<Button size="sm" @click="$emit('refetch')">
					<RefreshCwIcon class="h-5 w-5" />
					{{ formatMessage(messages.tryAgain) }}
				</Button>
				<Button size="sm" @click="$emit('home')">
					<HomeIcon class="h-5 w-5" />
					{{ formatMessage(messages.goToHome) }}
				</Button>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { FileIcon, HomeIcon, RefreshCwIcon } from '@modrinth/assets'

import { Button } from '#ui/components/base/buttons'
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
