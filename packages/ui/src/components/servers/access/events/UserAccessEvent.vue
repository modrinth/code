<template>
	<BaseEvent>
		<IntlFormatted :message-id="message" :values="{ permissions: permissionLabel }">
			<template #target-user>
				<EventEntityLink :entity="targetUser" />
			</template>
			<template #permission-label="{ children }">
				<span v-if="permissionRole" class="font-semibold text-contrast">
					<component :is="() => children" />
				</span>
				<component :is="() => children" v-else />
			</template>
		</IntlFormatted>
	</BaseEvent>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { computed } from 'vue'

import { defineMessages, type MessageDescriptor, useVIntl } from '../../../../composables/i18n'
import IntlFormatted from '../../../base/IntlFormatted.vue'
import { apiPermissionsToAccessRole } from '../permissions'
import type { ServerAccessRole } from '../types'
import BaseEvent from './BaseEvent.vue'
import EventEntityLink from './EventEntityLink.vue'
import type { EventEntity } from './types'

const props = defineProps<{
	kind: 'invited' | 'invite_revoked' | 'permission_modified' | 'removed'
	targetUser: EventEntity
	permissions?: Archon.ServerUsers.v1.UserScope | null
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	invited: {
		id: 'servers.audit-log.event.user-invited',
		defaultMessage: 'Invited <target-user></target-user>',
	},
	invitedWithPermissions: {
		id: 'servers.audit-log.event.user-invited-with-permissions',
		defaultMessage:
			'Invited <target-user></target-user> with <permission-label>{permissions}</permission-label>',
	},
	inviteRevoked: {
		id: 'servers.audit-log.event.user-invite-revoked',
		defaultMessage: 'Revoked invite for <target-user></target-user>',
	},
	permissionModified: {
		id: 'servers.audit-log.event.user-permission-modified',
		defaultMessage: 'Changed permissions for <target-user></target-user>',
	},
	permissionModifiedWithPermissions: {
		id: 'servers.audit-log.event.user-permission-modified-with-permissions',
		defaultMessage:
			'Changed permissions for <target-user></target-user> to <permission-label>{permissions}</permission-label>',
	},
	removed: {
		id: 'servers.audit-log.event.user-removed',
		defaultMessage: 'Removed <target-user></target-user>',
	},
	ownerRole: {
		id: 'servers.access-role.owner',
		defaultMessage: 'Owner',
	},
	editorRole: {
		id: 'servers.access-role.editor',
		defaultMessage: 'Editor',
	},
	viewerRole: {
		id: 'servers.access-role.viewer',
		defaultMessage: 'Limited',
	},
})

const roleMessages: Record<ServerAccessRole, MessageDescriptor> = {
	owner: messages.ownerRole,
	editor: messages.editorRole,
	viewer: messages.viewerRole,
}

const message = computed(() => {
	if (props.kind === 'invited') {
		return permissionLabel.value ? messages.invitedWithPermissions : messages.invited
	}
	if (props.kind === 'invite_revoked') return messages.inviteRevoked
	if (props.kind === 'permission_modified') {
		return permissionLabel.value
			? messages.permissionModifiedWithPermissions
			: messages.permissionModified
	}
	return messages.removed
})

const permissionLabel = computed(() => {
	const role = permissionRole.value
	if (role) return formatMessage(roleMessages[role])
	return ''
})

const permissionRole = computed(() =>
	props.permissions == null || props.permissions === ''
		? null
		: apiPermissionsToAccessRole(props.permissions),
)
</script>
