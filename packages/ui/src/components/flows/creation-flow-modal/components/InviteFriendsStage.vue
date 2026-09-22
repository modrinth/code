<template>
	<div class="flex flex-col gap-4">
		<img
			:src="ServerInviteFriendsIllustration"
			alt=""
			class="-mt-8 mx-auto h-[172px] w-[250px] object-contain"
		/>
		<div class="flex flex-col gap-1.5">
			<h2 class="m-0 text-2xl font-semibold text-contrast">
				{{ formatMessage(messages.title) }}
			</h2>
			<p class="m-0 text-base text-primary">
				{{ formatMessage(messages.description) }}
			</p>
		</div>
		<Button
			native-type="button"
			size="lg"
			class="-mx-2 self-stretch !justify-between text-left"
			:disabled="!ctx.inviteLink.value"
			@click="copyInviteLink"
		>
			<span class="min-w-0 truncate text-base font-semibold text-primary">
				{{
					ctx.inviteLink.value ??
					formatMessage(ctx.inviteLoading.value ? messages.preparing : messages.unavailable)
				}}
			</span>
			<SpinnerIcon v-if="ctx.inviteLoading.value" class="size-5 shrink-0 animate-spin" />
			<ClipboardCopyIcon v-else class="size-5 shrink-0 text-secondary" aria-hidden="true" />
		</Button>
		<div v-if="ctx.inviteError.value" class="flex flex-wrap items-center gap-2 text-sm text-red">
			<span>{{ ctx.inviteError.value }}</span>
			<Button size="sm" @click="ctx.retryInvite()">{{ formatMessage(messages.retry) }}</Button>
		</div>
	</div>
</template>

<script setup lang="ts">
import {
	ClipboardCopyIcon,
	ServerInviteFriendsIllustration,
	SpinnerIcon,
} from '@modrinth/assets'

import { Button } from '#ui/components/base/buttons'
import { injectCreationFlowContext } from '#ui/components/flows/creation-flow-modal/creation-flow-context'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { injectNotificationManager } from '#ui/providers'

const ctx = injectCreationFlowContext()
const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()

const messages = defineMessages({
	title: {
		id: 'servers.setup.onboarding.invite.title',
		defaultMessage: 'Invite players to join',
	},
	description: {
		id: 'servers.setup.onboarding.invite.description',
		defaultMessage:
			'Share this link and they’ll get an instance you manage from your server, with everything they need to play!',
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
	if (!ctx.inviteLink.value) return
	try {
		await navigator.clipboard.writeText(ctx.inviteLink.value)
		addNotification({ type: 'success', title: formatMessage(messages.copied) })
	} catch (error) {
		addNotification({
			type: 'error',
			title: formatMessage(messages.copyFailed),
			text: error instanceof Error ? error.message : String(error),
		})
	}
}
</script>
