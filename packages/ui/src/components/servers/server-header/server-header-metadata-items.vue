<template>
	<template v-if="server">
		<PageHeaderMetadataItem
			v-if="loader"
			:icon="TagIcon"
			:icon-props="{ tag: loader, enforceType: 'loader' }"
		>
			{{ formatLoaderLabel(loader) }} {{ gameVersion }}
		</PageHeaderMetadataItem>
		<PageHeaderMetadataItem
			v-if="server.net?.domain && !serverPreferences.hideSubdomainLabel"
			:icon="LinkIcon"
			tooltip="Copy server address"
			:action="copyServerAddress"
		>
			{{ server.net.domain }}.modrinth.gg
		</PageHeaderMetadataItem>
		<PageHeaderMetadataItem v-if="uptime" :icon="TimerIcon">
			{{ uptime }}
		</PageHeaderMetadataItem>
		<PageHeaderMetadataItem v-if="serverProject" :to="serverProjectLink" class="!text-primary">
			Linked to
			<Avatar :src="serverProject.icon_url" :alt="serverProject.title" size="24px" />
			{{ serverProject.title }}
		</PageHeaderMetadataItem>
	</template>
</template>

<script setup lang="ts">
import { LinkIcon, TimerIcon } from '@modrinth/assets'
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'

import Avatar from '#ui/components/base/Avatar.vue'
import PageHeaderMetadataItem from '#ui/components/base/page-header/metadata/page-header-metadata-item.vue'
import TagIcon from '#ui/components/base/TagIcon.vue'
import { useServerPreferences } from '#ui/composables/server-preferences'
import { serverKeys } from '#ui/composables/server-queries'
import { useServerProject } from '#ui/composables/use-server-project'
import { injectModrinthClient, injectNotificationManager } from '#ui/providers'
import { formatLoaderLabel } from '#ui/utils/loaders'

const props = defineProps<{
	serverId: string
	loader?: string | null
	gameVersion?: string | null
	uptime?: string | null
}>()

const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()

const { data: server } = useQuery({
	queryKey: computed(() => serverKeys.detail(props.serverId)),
	queryFn: () => client.archon.servers_v0.get(props.serverId),
})
const serverPreferences = useServerPreferences(props.serverId)
const { data: serverProject } = useServerProject(computed(() => server.value?.upstream ?? null))

const serverProjectLink = computed(() => {
	if (!serverProject.value) return ''
	return `/project/${serverProject.value.slug ?? serverProject.value.id}`
})

function copyServerAddress() {
	const domain = server.value?.net?.domain
	if (!domain) return

	void navigator.clipboard.writeText(`${domain}.modrinth.gg`)
	addNotification({
		title: 'Server address copied',
		text: "Your server's address has been copied to your clipboard.",
		type: 'success',
	})
}
</script>
