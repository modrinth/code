<template>
	<section
		class="flex flex-col gap-4 rounded-[20px] border border-solid border-surface-4 bg-surface-3 p-5"
	>
		<div class="flex flex-col gap-8 md:flex-row md:items-center">
			<div class="flex min-w-0 flex-1 flex-col gap-0.5">
				<h2 class="m-0 text-xl font-semibold text-contrast">
					{{ formatMessage(messages.playWithAppTitle) }}
				</h2>
				<p class="m-0 text-base leading-6 text-primary">
					{{ formatMessage(messages.playWithAppDescription) }}
				</p>
			</div>
			<div class="flex shrink-0 flex-col items-start gap-2">
				<Button type="colored" color="brand" size="lg" :disabled="disabled" @click="emit('play')">
					<SpinnerIcon v-if="pendingAction === 'play'" class="animate-spin" aria-hidden="true" />
					<PlayIcon v-else aria-hidden="true" />
					{{ formatMessage(messages.playServerButton) }}
				</Button>
			</div>
		</div>

		<div class="flex flex-col gap-3">
			<div class="h-px w-full bg-surface-4"></div>
			<div class="flex flex-col gap-0.5">
				<h2 class="m-0 text-xl font-semibold text-contrast">
					{{ formatMessage(messages.differentLauncherTitle) }}
				</h2>
				<p class="m-0 text-base leading-6 text-primary">
					{{ formatMessage(messages.differentLauncherDescription) }}
				</p>
			</div>
			<div class="flex flex-col gap-2 sm:flex-row sm:items-center">
				<Button size="lg" :disabled="disabled" @click="emit('download')">
					<SpinnerIcon
						v-if="pendingAction === 'download'"
						class="animate-spin"
						aria-hidden="true"
					/>
					<DownloadIcon v-else aria-hidden="true" />
					{{ formatMessage(messages.downloadModpackButton) }}
				</Button>
				<Button
					v-tooltip="formatMessage(copied ? messages.copiedAddress : messages.copyAddress)"
					native-type="button"
					size="lg"
					class="w-full !justify-between text-left sm:w-[313px]"
					:aria-label="formatMessage(copied ? messages.copiedAddress : messages.copyAddress)"
					@click="copyAddress"
				>
					<span class="min-w-0 truncate text-base font-semibold text-primary">{{ address }}</span>
					<CheckIcon v-if="copied" class="size-5 shrink-0 text-brand" aria-hidden="true" />
					<ClipboardCopyIcon v-else class="size-5 shrink-0 text-secondary" aria-hidden="true" />
				</Button>
			</div>
		</div>
	</section>
</template>

<script setup lang="ts">
import { CheckIcon, ClipboardCopyIcon, DownloadIcon, PlayIcon, SpinnerIcon } from '@modrinth/assets'
import { ref } from 'vue'

import { Button } from '#ui/components/base/buttons'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

const props = defineProps<{
	address: string
	disabled?: boolean
	pendingAction?: string
}>()

const emit = defineEmits<{
	play: []
	download: []
}>()

const { formatMessage } = useVIntl()
const copied = ref(false)

const messages = defineMessages({
	playWithAppTitle: {
		id: 'servers.play.card.app.title',
		defaultMessage: 'Play with Modrinth App',
	},
	playWithAppDescription: {
		id: 'servers.play.card.app.description',
		defaultMessage:
			'The easiest way to play is with the Modrinth App, which automatically installs all the required content for you and the players on your server.',
	},
	playServerButton: {
		id: 'servers.play.card.app.play-button',
		defaultMessage: 'Play server',
	},
	differentLauncherTitle: {
		id: 'servers.play.card.launcher.title',
		defaultMessage: 'Using a different launcher?',
	},
	differentLauncherDescription: {
		id: 'servers.play.card.launcher.description',
		defaultMessage:
			'Install the required content using your preferred launcher, then add the server address in-game to join.',
	},
	downloadModpackButton: {
		id: 'servers.play.card.launcher.download-button',
		defaultMessage: 'Download modpack',
	},
	copyAddress: {
		id: 'servers.play.card.address.copy',
		defaultMessage: 'Copy server address',
	},
	copiedAddress: {
		id: 'servers.play.card.address.copied',
		defaultMessage: 'Server address copied',
	},
})

async function copyAddress() {
	await navigator.clipboard.writeText(props.address)
	copied.value = true
	setTimeout(() => {
		copied.value = false
	}, 2000)
}
</script>
