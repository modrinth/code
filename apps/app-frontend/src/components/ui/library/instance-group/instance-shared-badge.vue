<script setup lang="ts">
import { CloudIcon } from '@modrinth/assets'
import { Avatar, defineMessages, Tooltip, useVIntl } from '@modrinth/ui'

import type { SharedInstanceOwner } from '@/components/ui/library/use-library'

defineProps<{
	owner: SharedInstanceOwner | null
	isOwner: boolean
	compact: boolean
}>()

const { formatMessage } = useVIntl()
const messages = defineMessages({
	sharedBy: {
		id: 'app.library.instance.shared-by-prefix',
		defaultMessage: 'Shared by',
	},
	sharedByYou: {
		id: 'app.library.instance.shared-by-you',
		defaultMessage: 'Shared by you',
	},
})
</script>

<template>
	<Tooltip placement="top" class="absolute z-[2]" :class="compact ? 'left-0 top-0' : 'left-2 top-2'">
		<span
			class="flex items-center justify-center rounded-full bg-surface-1/60 text-contrast"
			:class="compact ? 'size-4 p-px' : 'size-7 p-1'"
			role="img"
			:aria-label="
				isOwner || !owner
					? formatMessage(messages.sharedByYou)
					: `${formatMessage(messages.sharedBy)} ${owner.name}`
			"
		>
			<CloudIcon class="size-full" />
		</span>
		<template #popper>
			<span v-if="!isOwner && owner" class="flex items-center gap-1.5">
				{{ formatMessage(messages.sharedBy) }}
				<Avatar
					:src="owner.avatarUrl"
					cache-icon
					:tint-by="owner.name"
					size="18px"
					alt=""
					circle
					no-shadow
				/>
				{{ owner.name }}
			</span>
			<span v-else>{{ formatMessage(messages.sharedByYou) }}</span>
		</template>
	</Tooltip>
</template>
