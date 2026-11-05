<template>
	<div class="flex flex-col gap-4">
		<img
			:src="ServerInviteFriendsIllustration"
			alt=""
			class="-mt-6 mx-auto mb-2 h-[172px] w-[250px] object-contain"
		/>
		<div class="flex flex-col gap-1.5">
			<h2 class="m-0 text-2xl font-semibold text-contrast">
				{{ formatMessage(messages.title) }}
			</h2>
			<p class="m-0 text-base text-primary">
				{{ formatMessage(messages.description) }}
			</p>
		</div>
		<div class="flex flex-col gap-2">
			<Button
				native-type="button"
				size="lg"
				class="self-stretch !justify-between text-left"
				:disabled="!link"
				:aria-label="recentlyCopied ? formatMessage(messages.copied) : undefined"
				@click="copyInviteLink"
			>
				<span class="min-w-0 truncate text-base font-semibold text-primary">
					{{ link ?? formatMessage(loading ? messages.preparing : messages.unavailable) }}
				</span>
				<SpinnerIcon v-if="loading" class="size-5 shrink-0 animate-spin" />
				<CheckIcon
					v-else-if="recentlyCopied"
					class="size-5 shrink-0 text-brand"
					aria-hidden="true"
				/>
				<ClipboardCopyIcon v-else class="size-5 shrink-0 text-secondary" aria-hidden="true" />
			</Button>
			<p class="m-0 text-base text-secondary">{{ formatMessage(messages.inviteLater) }}</p>
		</div>
		<div v-if="error" class="flex flex-wrap items-center gap-2 text-sm text-red">
			<span>{{ error }}</span>
			<Button size="sm" @click="emit('retry')">{{ formatMessage(messages.retry) }}</Button>
		</div>
	</div>
</template>

<script setup lang="ts">
import {
	CheckIcon,
	ClipboardCopyIcon,
	ServerInviteFriendsIllustration,
	SpinnerIcon,
} from '@modrinth/assets'
import { useTimeoutFn } from '@vueuse/core'
import { ref, watch } from 'vue'

import { Button } from '#ui/components/base/buttons'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { injectNotificationManager } from '#ui/providers'

const props = defineProps<{
	link: string | null
	loading?: boolean
	error?: string | null
}>()

const emit = defineEmits<{
	retry: []
	copied: []
}>()

const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()
const recentlyCopied = ref(false)
const { start: resetCopiedTimer, stop: stopCopiedTimer } = useTimeoutFn(
	() => {
		recentlyCopied.value = false
	},
	1500,
	{ immediate: false },
)

watch(
	() => props.link,
	() => {
		stopCopiedTimer()
		recentlyCopied.value = false
	},
)

const messages = defineMessages({
	title: {
		id: 'servers.setup.onboarding.invite.title',
		defaultMessage: 'Invite friends to join',
	},
	description: {
		id: 'servers.setup.onboarding.invite.description',
		defaultMessage:
			'Share this link and they can create a shared instance with everything they need to play, managed right from your server!',
	},
	inviteLater: {
		id: 'servers.setup.onboarding.invite.invite-later',
		defaultMessage: 'Invite more players anytime from your server’s Play page.',
	},
	preparing: {
		id: 'servers.setup.onboarding.invite.preparing',
		defaultMessage: 'Preparing your invite link...',
	},
	unavailable: {
		id: 'servers.setup.onboarding.invite.unavailable',
		defaultMessage: 'Invite link unavailable',
	},
	retry: {
		id: 'servers.setup.onboarding.invite.retry',
		defaultMessage: 'Retry',
	},
	copied: {
		id: 'servers.setup.onboarding.invite.copied',
		defaultMessage: 'Link copied',
	},
	copyFailed: {
		id: 'servers.setup.onboarding.invite.copy-failed',
		defaultMessage: 'Failed to copy link',
	},
})

async function copyInviteLink() {
	const link = props.link
	if (!link) return
	try {
		await navigator.clipboard.writeText(link)
		if (props.link === link) {
			emit('copied')
			recentlyCopied.value = true
			resetCopiedTimer()
		}
	} catch (error) {
		addNotification({
			type: 'error',
			title: formatMessage(messages.copyFailed),
			text: error instanceof Error ? error.message : String(error),
		})
	}
}
</script>
