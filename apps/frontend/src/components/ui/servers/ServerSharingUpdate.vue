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

const props = defineProps<{
	serverId?: string
}>()

const config = useRuntimeConfig()
const auth = useAuthState()
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

onMounted(async () => {
	const userId = auth.value.user?.id
	if (!userId || alreadyShown()) return
	const target = await resolveServerSharingUpdateTarget(
		client,
		queryClient,
		userId,
		props.serverId,
	).catch(() => null)
	if (!target || !modal.value || alreadyShown()) return
	try {
		localStorage.setItem(SHOWN_KEY, 'true')
	} catch {
		return
	}
	modal.value.show(target)
})
</script>
