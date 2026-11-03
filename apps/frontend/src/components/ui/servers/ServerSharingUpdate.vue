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
const { formatMessage } = useVIntl()
const modal = useTemplateRef<InstanceType<typeof ServerSharingUpdateModal>>('modal')

const messages = defineMessages({
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
	modal.value.show(target)
}

onMounted(() => {
	watch(() => auth.value.user?.id, showIfEligible, { immediate: true })
})
</script>
