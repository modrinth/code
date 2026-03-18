<template>
	<div
		v-tooltip="showCustomModpackTooltip ? formatMessage(messages.customModpackTooltip) : name"
		class="flex gap-1.5 items-center flex-shrink overflow-hidden smart-clickable:allow-pointer-events"
		:class="[onclick ? 'hover:underline cursor-pointer' : '']"
		@click="onclick"
	>
		<Avatar :src="icon" size="24px" />
		<span class="truncate font-medium">
			{{ name }}
		</span>
	</div>
</template>

<script setup lang="ts">
import { defineMessages, useVIntl } from '../../../composables'
import Avatar from '../../base/Avatar.vue'

defineProps<{
	name: string
	icon?: string
	onclick?: () => void
	showCustomModpackTooltip?: boolean
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	customModpackTooltip: {
		id: `project.server.customModpackTooltip`,
		defaultMessage: 'This project uses a custom modpack',
	},
})
</script>
