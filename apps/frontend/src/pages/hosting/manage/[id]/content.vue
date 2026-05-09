<script setup lang="ts">
import {
	commonMessages,
	defineMessages,
	injectModrinthClient,
	injectModrinthServerContext,
	ServersManageContentPage,
	useVIntl,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'

const client = injectModrinthClient()
const { server, serverId, worldId } = injectModrinthServerContext()
const queryClient = useQueryClient()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'servers.manage.content.title',
		defaultMessage: 'Content - {serverName} - Modrinth',
	},
})

async function getContentWorldId() {
	if (worldId.value) return worldId.value

	const serverFull = await queryClient.ensureQueryData({
		queryKey: ['servers', 'v1', 'detail', serverId],
		queryFn: () => client.archon.servers_v1.get(serverId),
		staleTime: 30_000,
	})
	const activeWorld = serverFull.worlds.find((world) => world.is_active)
	return activeWorld?.id ?? serverFull.worlds[0]?.id ?? null
}

const contentWorldId = await getContentWorldId()

if (contentWorldId) {
	try {
		const content = await queryClient.ensureQueryData({
			queryKey: ['content', 'list', 'v1', serverId],
			queryFn: () =>
				client.archon.content_v1.getAddons(serverId, contentWorldId, { from_modpack: false }),
			staleTime: 30_000,
		})

		const modpackProjectId =
			content.modpack?.spec.platform === 'modrinth' ? content.modpack.spec.project_id : null

		if (modpackProjectId) {
			await queryClient.ensureQueryData({
				queryKey: ['labrinth', 'project', modpackProjectId],
				queryFn: () => client.labrinth.projects_v2.get(modpackProjectId),
				staleTime: 30_000,
			})
		}
	} catch {
		// Let mounted layouts' useQuery surface errors; do not fail route setup.
	}
}

useHead({
	title: () =>
		formatMessage(messages.title, {
			serverName: server.value?.name ?? formatMessage(commonMessages.serverLabel),
		}),
})
</script>

<template>
	<ServersManageContentPage :owner-avatar-url-base="''" />
</template>
