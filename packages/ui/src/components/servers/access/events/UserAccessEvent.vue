<template>
	<BaseEvent>
		<template v-if="kind === 'invited'">
			Invited <EventEntityLink :entity="targetUser" />
			<span v-if="permissionLabel" class="text-secondary"> with {{ permissionLabel }}</span>
		</template>
		<template v-else-if="kind === 'invite_revoked'">
			Revoked invite for <EventEntityLink :entity="targetUser" />
		</template>
		<template v-else-if="kind === 'permission_modified'">
			Changed permissions for <EventEntityLink :entity="targetUser" />
			<span v-if="permissionLabel" class="text-secondary"> to {{ permissionLabel }}</span>
		</template>
		<template v-else>
			Removed <EventEntityLink :entity="targetUser" />
		</template>
	</BaseEvent>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import BaseEvent from './BaseEvent.vue'
import EventEntityLink from './EventEntityLink.vue'
import type { EventEntity } from './types'

const props = defineProps<{
	kind: 'invited' | 'invite_revoked' | 'permission_modified' | 'removed'
	targetUser: EventEntity
	permissions?: string | null
}>()

const permissionLabel = computed(() => {
	if (!props.permissions) return ''
	return props.permissions
		.split('|')
		.map((permission) => permission.trim().toLowerCase().replaceAll('_', ' '))
		.filter(Boolean)
		.join(', ')
})
</script>
