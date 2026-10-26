<script setup lang="ts">
import { Avatar, defineMessages, useVIntl } from '@modrinth/ui'

import type { SharedInstanceOwner } from '@/components/ui/library/use-library'

defineProps<{
	owner: SharedInstanceOwner
	compact: boolean
}>()

const { formatMessage } = useVIntl()
const messages = defineMessages({
	sharedBy: {
		id: 'app.library.instance.shared-by',
		defaultMessage: 'Shared by {name}',
	},
})
</script>

<template>
	<span
		v-tooltip="formatMessage(messages.sharedBy, { name: owner.name })"
		class="absolute z-[2] flex rounded-full bg-surface-3"
		:class="compact ? 'bottom-0 left-0 p-px' : 'bottom-1.5 left-1.5 p-0.5'"
		:aria-label="formatMessage(messages.sharedBy, { name: owner.name })"
		role="img"
	>
		<Avatar
			:src="owner.avatarUrl"
			cache-icon
			:tint-by="owner.name"
			:size="compact ? '14px' : '28px'"
			alt=""
			circle
			no-shadow
		/>
	</span>
</template>
