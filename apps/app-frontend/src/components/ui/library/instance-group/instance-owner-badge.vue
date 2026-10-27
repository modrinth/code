<script setup lang="ts">
import { Avatar, useVIntl } from '@modrinth/ui'

import type { SharedInstanceOwner } from '@/features/library/library-instance'
import { libraryInstanceMessages } from '@/features/library/messages'

defineProps<{
	owner: SharedInstanceOwner
	compact: boolean
}>()

const { formatMessage } = useVIntl()
</script>

<template>
	<span
		v-tooltip="formatMessage(libraryInstanceMessages.sharedBy, { name: owner.name })"
		class="absolute z-[2] flex rounded-full bg-surface-3"
		:class="compact ? 'bottom-0 left-0 p-px' : 'bottom-1.5 left-1.5 p-0.5'"
		:aria-label="formatMessage(libraryInstanceMessages.sharedBy, { name: owner.name })"
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
