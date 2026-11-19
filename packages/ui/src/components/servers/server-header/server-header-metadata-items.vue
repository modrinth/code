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
			v-if="serverAddress && !serverPreferences.hideSubdomainLabel"
			:icon="LinkIcon"
			:tooltip="formatMessage(messages.copyAddress)"
			:action="copyServerAddress"
		>
			{{ serverAddress }}
		</PageHeaderMetadataItem>
		<PageHeaderMetadataItem v-if="uptime" :icon="TimerIcon">
			{{ uptime }}
		</PageHeaderMetadataItem>
		<PageHeaderMetadataItem v-if="serverProject" :to="serverProjectLink" class="!text-primary">
			<IntlFormatted :message-id="messages.linkedTo">
				<template #project>
					<Avatar :src="serverProject.icon_url" :alt="serverProject.title" size="24px" />
					{{ serverProject.title }}
				</template>
			</IntlFormatted>
		</PageHeaderMetadataItem>
	</template>
</template>

<script setup lang="ts">
import { LinkIcon, TimerIcon } from '@modrinth/assets'
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'

import Avatar from '#ui/components/base/Avatar.vue'
import IntlFormatted from '#ui/components/base/IntlFormatted.vue'
import PageHeaderMetadataItem from '#ui/components/base/page-header/metadata/page-header-metadata-item.vue'
import TagIcon from '#ui/components/base/TagIcon.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { useServerPreferences } from '#ui/composables/server-preferences'
import { serverDetailQueryOptions } from '#ui/composables/server-queries'
import { useServerProject } from '#ui/composables/use-server-project'
import { injectModrinthClient, injectNotificationManager } from '#ui/providers'
import { formatLoaderLabel } from '#ui/utils/loaders'

const props = defineProps<{
	serverId: string
	loader?: string | null
	gameVersion?: string | null
	uptime?: string | null
}>()

const messages = defineMessages({
	copyAddress: {
		id: 'servers.header.copy-address',
		defaultMessage: 'Copy server address',
	},
	addressCopiedTitle: {
		id: 'servers.header.address-copied.title',
		defaultMessage: 'Server address copied',
	},
	addressCopiedText: {
		id: 'servers.header.address-copied.text',
		defaultMessage: "Your server's address has been copied to your clipboard.",
	},
	linkedTo: {
		id: 'servers.header.linked-to',
		defaultMessage: 'Linked to {project}',
	},
})

const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()

const { data: server } = useQuery(computed(() => serverDetailQueryOptions(client, props.serverId)))
const serverPreferences = useServerPreferences(props.serverId)
const { data: serverProject } = useServerProject(computed(() => server.value?.upstream ?? null))

const serverAddress = computed(() =>
	server.value?.net?.domain ? `${server.value.net.domain}.modrinth.gg` : null,
)
const serverProjectLink = computed(() => {
	if (!serverProject.value) return ''
	return `/project/${serverProject.value.slug ?? serverProject.value.id}`
})

function copyServerAddress() {
	if (!serverAddress.value) return

	void navigator.clipboard.writeText(serverAddress.value)
	addNotification({
		title: formatMessage(messages.addressCopiedTitle),
		text: formatMessage(messages.addressCopiedText),
		type: 'success',
	})
}
</script>
