<script setup lang="ts">
import { ScaleIcon } from '@modrinth/assets'
import {
	AutoLink,
	Avatar,
	defineMessages,
	IntlFormatted,
	useFormatDateTime,
	useVIntl,
} from '@modrinth/ui'
import { computed } from 'vue'

import type { DisclosureUpdatedByUser } from './types'

const props = defineProps<{
	updatedAt: string
	updatedBy?: DisclosureUpdatedByUser | null
	setByModerator?: boolean
}>()

const { formatMessage } = useVIntl()
const formatDate = useFormatDateTime({ dateStyle: 'long' })
const formatDateTime = useFormatDateTime({
	dateStyle: 'long',
	timeStyle: 'short',
})
const flags = useFeatureFlags()

const showModeratorLabel = computed(
	() => !!props.setByModerator && (!props.updatedBy || flags.value.showModeratorProjectMemberUi),
)

const messages = defineMessages({
	lastUpdated: {
		id: 'project.settings.disclosures.last-updated',
		defaultMessage: 'Last updated on {date} by {user}',
	},
	updatedByModerator: {
		id: 'project.settings.disclosures.last-updated.moderator',
		defaultMessage: 'Moderator',
	},
})
</script>

<template>
	<div class="inline-flex flex-wrap items-center gap-1 text-secondary">
		<IntlFormatted :message-id="messages.lastUpdated">
			<template #date>
				<span v-tooltip="formatDateTime(updatedAt)">{{ formatDate(updatedAt) }}</span>
			</template>
			<template #user>
				<span v-if="showModeratorLabel" class="inline-flex items-center gap-1 text-orange">
					<ScaleIcon class="size-5 shrink-0" />
					{{ formatMessage(messages.updatedByModerator) }}
				</span>
				<AutoLink
					v-else-if="updatedBy"
					:to="`/user/${updatedBy.username}`"
					class="inline-flex min-w-0 max-w-full items-center gap-1 text-contrast hover:underline"
				>
					<Avatar
						v-if="updatedBy.avatar_url"
						:src="updatedBy.avatar_url"
						:alt="updatedBy.username"
						size="20px"
						class="shrink-0"
						circle
					/>
					<span class="truncate">{{ updatedBy.username }}</span>
				</AutoLink>
			</template>
		</IntlFormatted>
	</div>
</template>
