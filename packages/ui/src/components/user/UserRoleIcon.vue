<template>
	<ModrinthIcon
		v-if="role === 'admin'"
		v-tooltip="formatMessage(messages.modrinthTeamLabel)"
		role="img"
		:aria-label="formatMessage(messages.modrinthTeamLabel)"
		class="size-4 shrink-0 !text-green"
	/>
	<ScaleIcon
		v-else-if="role === 'moderator'"
		v-tooltip="formatMessage(messages.moderatorLabel)"
		role="img"
		:aria-label="formatMessage(messages.moderatorLabel)"
		class="size-4 shrink-0 !text-orange"
	/>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { ModrinthIcon, ScaleIcon } from '@modrinth/assets'

import { defineMessages, useVIntl } from '#ui/composables/i18n'

defineProps<{
	role?: Labrinth.Users.v2.Role | null
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	modrinthTeamLabel: {
		id: 'user.role.modrinth-team',
		defaultMessage: 'Modrinth Team',
	},
	moderatorLabel: {
		id: 'user.role.moderator',
		defaultMessage: 'Content Moderator',
	},
})
</script>
