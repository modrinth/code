<script setup lang="ts">
import { TagCategoryCloudIcon } from '@modrinth/assets'
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
	<Tooltip placement="top" class="absolute z-[2]" :class="compact ? 'left-0 top-0' : 'left-1.5 top-1.5'">
		<span
			class="flex items-center justify-center rounded-full bg-white/70 text-surface-1"
			:class="compact ? 'size-4' : 'size-6'"
			role="img"
			:aria-label="
				isOwner || !owner
					? formatMessage(messages.sharedByYou)
					: `${formatMessage(messages.sharedBy)} ${owner.name}`
			"
		>
			<TagCategoryCloudIcon :class="compact ? 'size-2.5' : 'size-4'" class="[stroke-width:2.5]" />
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
