<template>
	<ServerSharingUpdateModal
		ref="modal"
		:site-url="config.public.siteUrl as string"
		:description="formatMessage(messages.description)"
		video-src="https://cdn.modrinth.com/media/server-play-demo.webm"
		poster-src="https://cdn.modrinth.com/media/server-play-demo.webp"
	/>
</template>

<script setup lang="ts">
import {
	defineMessages,
	injectModrinthClient,
	injectPopupNotificationManager,
	resolveServerSharingUpdateTarget,
	ServerSharingUpdateModal,
	useVIntl,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'

const SHOWN_KEY = 'server-sharing-update-shown'

const config = useRuntimeConfig()
const auth = useAuthState()
const route = useNativeRoute()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const popupNotificationManager = injectPopupNotificationManager()
const { formatMessage } = useVIntl()
const modal = useTemplateRef<InstanceType<typeof ServerSharingUpdateModal>>('modal')

const messages = defineMessages({
	notificationTitle: {
		id: 'servers.sharing-update.notification.title',
		defaultMessage: 'Share servers with friends',
	},
	notificationDescription: {
		id: 'servers.sharing-update.notification.description',
		defaultMessage: 'Invite friends to play with an instance managed by your server.',
	},
	notificationPurchaseDescription: {
		id: 'servers.sharing-update.notification.purchase-description',
		defaultMessage: 'Get a server and invite friends to play with an instance managed by it.',
	},
	viewUpdate: {
		id: 'servers.sharing-update.notification.view-update',
		defaultMessage: 'View update',
	},
	dismiss: {
		id: 'servers.sharing-update.notification.dismiss',
		defaultMessage: 'Dismiss',
	},
	description: {
		id: 'servers.sharing-update.web-description',
		defaultMessage:
			'You can now share your server directly with friends. When they accept in the Modrinth App, we’ll create an instance for them that is managed by your server.',
	},
})

function alreadyShown() {
	try {
		return localStorage.getItem(SHOWN_KEY) === 'true'
	} catch {
		return true
	}
}

function currentServerId() {
	const id = route.params.id
	return route.path.startsWith('/hosting/manage/') && typeof id === 'string' ? id : undefined
}

async function showIfEligible(userId: string | undefined) {
	if (!userId || alreadyShown()) return
	const target = await resolveServerSharingUpdateTarget(
		client,
		queryClient,
		userId,
		currentServerId(),
	).catch(() => null)
	if (!target || !modal.value || auth.value.user?.id !== userId || alreadyShown()) return
	try {
		localStorage.setItem(SHOWN_KEY, 'true')
	} catch {
		return
	}
	popupNotificationManager.addPopupNotification({
		contentType: 'standard',
		title: formatMessage(messages.notificationTitle),
		text: formatMessage(
			target.kind === 'purchase'
				? messages.notificationPurchaseDescription
				: messages.notificationDescription,
		),
		type: 'info',
		hideIcon: true,
		autoCloseMs: null,
		buttons: [
			{
				label: formatMessage(messages.dismiss),
				color: 'standard',
				action: () => {},
			},
			{
				label: formatMessage(messages.viewUpdate),
				color: 'brand',
				action: () => modal.value?.show(target),
			},
		],
	})
}

onMounted(() => {
	watch(() => auth.value.user?.id, showIfEligible, { immediate: true })
})
</script>
