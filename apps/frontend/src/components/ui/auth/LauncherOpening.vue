<template>
	<div>
		<iframe
			v-if="localhostUrl"
			:src="localhostUrl"
			class="hidden"
			:title="formatMessage(messages.launcherCallbackTitle)"
		></iframe>
		<AuthCard class="text-center">
			<div class="flex flex-col gap-2">
				<h1 class="m-0 text-2xl font-semibold text-contrast">
					{{ formatMessage(messages.openingLauncherTitle) }}
				</h1>
				<p class="m-0 text-left text-primary">
					{{ formatMessage(messages.openingLauncherDescription) }}
				</p>
			</div>
			<div class="flex flex-col gap-2">
				<ButtonLink
					v-if="deeplinkUrl"
					:href="deeplinkUrl"
					type="colored"
					color="brand"
					class="!w-full !justify-center"
				>
					{{ formatMessage(messages.returnToLauncherButton) }}
					<RightArrowIcon />
				</ButtonLink>
				<Button
					v-else
					type="colored"
					color="brand"
					class="!w-full !justify-center"
					@click="sendLocalhostCallback"
				>
					{{ formatMessage(messages.returnToLauncherButton) }}
					<RightArrowIcon />
				</Button>
				<ButtonLink to="/" class="!w-full !justify-center">
					{{ formatMessage(messages.goToWebsiteButton) }}
				</ButtonLink>
			</div>
		</AuthCard>
	</div>
</template>

<script setup lang="ts">
import { RightArrowIcon } from '@modrinth/assets'
import { Button, ButtonLink, defineMessages, useVIntl } from '@modrinth/ui'

import AuthCard from '@/components/ui/auth/AuthCard.vue'

interface Props {
	localhostUrl?: string
	deeplinkUrl?: string
}

const { localhostUrl = '', deeplinkUrl = '' } = defineProps<Props>()

async function sendLocalhostCallback() {
	if (!localhostUrl) {
		return
	}
	await fetch(localhostUrl, { mode: 'no-cors' }).catch(() => undefined)
}

const { formatMessage } = useVIntl()

const messages = defineMessages({
	launcherCallbackTitle: {
		id: 'auth.sign-in.launcher.callback.title',
		defaultMessage: 'Modrinth App sign-in callback',
	},
	openingLauncherTitle: {
		id: 'auth.sign-in.launcher.opening.title',
		defaultMessage: 'Opening Modrinth App...',
	},
	openingLauncherDescription: {
		id: 'auth.sign-in.launcher.opening.description',
		defaultMessage: 'If the app doesn’t open, use the button below to finish signing in.',
	},
	returnToLauncherButton: {
		id: 'auth.sign-in.launcher.complete.return-button',
		defaultMessage: 'Open Modrinth App',
	},
	goToWebsiteButton: {
		id: 'auth.sign-in.launcher.complete.go-to-website',
		defaultMessage: 'Go to Modrinth.com',
	},
})
</script>
