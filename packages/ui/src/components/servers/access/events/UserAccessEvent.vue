<template>
	<BaseEvent>
		<IntlFormatted :message-id="message" :values="{ permissions: permissionLabel }">
			<template #target-user>
				<EventEntityLink :entity="targetUser" />
			</template>
		</IntlFormatted>
	</BaseEvent>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import {
	defineMessages,
	type MessageDescriptor,
	useVIntl,
} from '../../../../composables/i18n'
import IntlFormatted from '../../../base/IntlFormatted.vue'
import BaseEvent from './BaseEvent.vue'
import EventEntityLink from './EventEntityLink.vue'
import type { EventEntity } from './types'

const props = defineProps<{
	kind: 'invited' | 'invite_revoked' | 'permission_modified' | 'removed'
	targetUser: EventEntity
	permissions?: string[] | null
}>()

const { formatMessage, locale } = useVIntl()

const messages = defineMessages({
	invited: {
		id: 'servers.audit-log.event.user-invited',
		defaultMessage: 'Invited <target-user></target-user>',
	},
	invitedWithPermissions: {
		id: 'servers.audit-log.event.user-invited-with-permissions',
		defaultMessage: 'Invited <target-user></target-user> with {permissions}',
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
		defaultMessage: 'Changed permissions for <target-user></target-user> to {permissions}',
	},
	removed: {
		id: 'servers.audit-log.event.user-removed',
		defaultMessage: 'Removed <target-user></target-user>',
	},
	serverAdmin: {
		id: 'servers.audit-log.permission.server-admin',
		defaultMessage: 'server admin',
	},
	baseRead: {
		id: 'servers.audit-log.permission.base-read',
		defaultMessage: 'base read',
	},
	powerActions: {
		id: 'servers.audit-log.permission.power-actions',
		defaultMessage: 'power actions',
	},
	filesWrite: {
		id: 'servers.audit-log.permission.files-write',
		defaultMessage: 'file writes',
	},
	setup: {
		id: 'servers.audit-log.permission.setup',
		defaultMessage: 'setup',
	},
	backups: {
		id: 'servers.audit-log.permission.backups',
		defaultMessage: 'backups',
	},
	advanced: {
		id: 'servers.audit-log.permission.advanced',
		defaultMessage: 'advanced settings',
	},
	resetServer: {
		id: 'servers.audit-log.permission.reset-server',
		defaultMessage: 'server reset',
	},
	manageUsers: {
		id: 'servers.audit-log.permission.manage-users',
		defaultMessage: 'manage users',
	},
	supportAgent: {
		id: 'servers.audit-log.permission.support-agent',
		defaultMessage: 'support agent',
	},
	infraManager: {
		id: 'servers.audit-log.permission.infra-manager',
		defaultMessage: 'infrastructure manager',
	},
	infraManagerRead: {
		id: 'servers.audit-log.permission.infra-manager-read',
		defaultMessage: 'infrastructure read',
	},
	infraServersTransfer: {
		id: 'servers.audit-log.permission.infra-servers-transfer',
		defaultMessage: 'server transfer',
	},
})

const permissionMessages: Record<string, MessageDescriptor> = {
	SERVER_ADMIN: messages.serverAdmin,
	BASE_READ: messages.baseRead,
	POWER_ACTIONS: messages.powerActions,
	FILES_WRITE: messages.filesWrite,
	SETUP: messages.setup,
	BACKUPS: messages.backups,
	ADVANCED: messages.advanced,
	RESET_SERVER: messages.resetServer,
	MANAGE_USERS: messages.manageUsers,
	SUPPORT_AGENT: messages.supportAgent,
	INFRA_MANAGER: messages.infraManager,
	INFRA_MANAGER_READ: messages.infraManagerRead,
	INFRA_SERVERS_XFER: messages.infraServersTransfer,
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
	if (!props.permissions || props.permissions.length === 0) return ''
	void locale.value

	const labels = props.permissions.map(formatPermission).filter(Boolean)
	return new Intl.ListFormat(locale.value, {
		style: 'long',
		type: 'conjunction',
	}).format(labels)
})

function formatPermission(permission: string): string {
	const descriptor = permissionMessages[permission]
	return descriptor ? formatMessage(descriptor) : permission
}
</script>
