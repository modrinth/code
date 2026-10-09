<template>
	<component
		:is="icon"
		v-if="icon"
		v-tooltip="label"
		:aria-label="label"
		:class="[
			avatar ? 'h-3/5 w-3/5' : 'mb-1 ml-1 mr-0.5 inline-block size-3.5 align-middle',
			role === 'admin' ? 'text-green' : 'text-orange',
		]"
	/>
</template>

<script setup lang="ts">
import { ModrinthIcon, ScaleIcon } from '@modrinth/assets'
import { defineMessages, useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

const props = defineProps<{ role?: string; avatar?: boolean }>()

const { formatMessage } = useVIntl()
const messages = defineMessages({
	team: { id: 'thread.message.modrinth-team', defaultMessage: 'Modrinth Team' },
	moderator: { id: 'thread.message.moderator', defaultMessage: 'Moderator' },
	system: { id: 'thread.message.system', defaultMessage: 'Modrinth' },
})
const icon = computed(() =>
	props.role === 'admin' || props.role === 'system'
		? ModrinthIcon
		: props.role === 'moderator'
			? ScaleIcon
			: undefined,
)
const label = computed(() =>
	formatMessage(
		props.role === 'admin'
			? messages.team
			: props.role === 'system'
				? messages.system
				: messages.moderator,
	),
)
</script>
