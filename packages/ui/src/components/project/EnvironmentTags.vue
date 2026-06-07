<script setup lang="ts">
import { defineMessage, useVIntl, type MessageDescriptor } from '#ui/composables/i18n.ts'
import type { Labrinth } from '@modrinth/api-client'
import { ClientIcon, MonitorSmartphoneIcon, ServerIcon, UserIcon } from '@modrinth/assets'
import { computed, type Component } from 'vue'
import TagItem from '../base/TagItem.vue'

const { formatMessage } = useVIntl()

const props = defineProps<{
	environment: Labrinth.Projects.v3.Environment
}>()

type EnvironmentTag = {
	icon: Component
	message: MessageDescriptor
	environments: Labrinth.Projects.v3.Environment[]
}

const environmentTags: EnvironmentTag[] = [
	{
		icon: ClientIcon,
		message: defineMessage({
			id: `project.about.compatibility.environments.client-side`,
			defaultMessage: 'Client-side',
		}),
		environments: [
			'client_only',
			'client_only_server_optional',
			'client_or_server',
			'client_or_server_prefers_both',
		],
	},
	{
		icon: ServerIcon,
		message: defineMessage({
			id: `project.about.compatibility.environments.server-side`,
			defaultMessage: 'Server-side',
		}),
		environments: [
			'server_only',
			'server_only_client_optional',
			'client_or_server',
			'client_or_server_prefers_both',
		],
	},
	{
		icon: ServerIcon,
		message: defineMessage({
			id: `project.about.compatibility.environments.dedicated-servers-only`,
			defaultMessage: 'Dedicated servers only',
		}),
		environments: ['dedicated_server_only'],
	},
	{
		icon: UserIcon,
		message: defineMessage({
			id: `project.about.compatibility.environments.singleplayer-only`,
			defaultMessage: 'Singleplayer only',
		}),
		environments: ['singleplayer_only'],
	},
	{
		icon: UserIcon,
		message: defineMessage({
			id: `project.about.compatibility.environments.singleplayer`,
			defaultMessage: 'Singleplayer',
		}),
		environments: ['server_only'],
	},
	{
		icon: MonitorSmartphoneIcon,
		message: defineMessage({
			id: `project.about.compatibility.environments.client-and-server`,
			defaultMessage: 'Client and server',
		}),
		environments: [
			'client_and_server',
			'client_only_server_optional',
			'server_only_client_optional',
			'client_or_server_prefers_both',
		],
	},
]

const tags = computed(() => {
	return environmentTags.filter((x) => x.environments.includes(props.environment ?? 'unknown'))
})
</script>
<template>
	<TagItem v-for="tag in tags" :key="`environment-tag-${tag.message.id}`">
		<component :is="tag.icon" />
		{{ formatMessage(tag.message) }}
	</TagItem>
</template>
