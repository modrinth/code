import { Archon } from '@modrinth/api-client'
import { computed } from 'vue'

import { useVIntl } from '#ui/composables/i18n'
import { injectModrinthServerContext } from '#ui/providers'
import { commonMessages } from '#ui/utils/common-messages'

const UserScope = Archon.ServerUsers.v1.UserScope

export type ServerPermissionName = keyof typeof UserScope

export const serverPermissionBits = {
	NONE: 0,
	SERVER_ADMIN: 1 << 0,
	BASE_READ: 1 << 1,
	POWER_ACTIONS: 1 << 2,
	FILES_WRITE: 1 << 3,
	SETUP: 1 << 4,
	BACKUPS: 1 << 5,
	ADVANCED: 1 << 6,
	RESET_SERVER: 1 << 7,
	MANAGE_USERS: 1 << 8,
	SUPPORT_AGENT: 1 << 9,
	INFRA_MANAGER: 1 << 10,
	INFRA_MANAGER_READ: 1 << 11,
	INFRA_SERVERS_XFER: 1 << 12,
} as const satisfies Record<ServerPermissionName, Archon.Servers.v0.UserScope>

function hasPermissionBit(
	permissions: Archon.Servers.v0.UserScope,
	scope: ServerPermissionName,
) {
	const permission = serverPermissionBits[scope]
	return permission === 0 || (permissions & permission) === permission
}

export function hasServerPermission(
	permissions: Archon.Servers.v0.UserScope,
	scope: ServerPermissionName,
) {
	if (scope !== 'NONE' && scope !== 'SERVER_ADMIN' && hasPermissionBit(permissions, 'SERVER_ADMIN')) {
		return true
	}
	return hasPermissionBit(permissions, scope)
}

export function useServerPermissions() {
	const { formatMessage } = useVIntl()
	const { currentUserPermissions } = injectModrinthServerContext()

	const hasCurrentUserPermission = (scope: ServerPermissionName) =>
		hasServerPermission(currentUserPermissions.value, scope)

	const permissionDeniedMessage = computed(() => formatMessage(commonMessages.noPermissionAction))

	const canUsePowerActions = computed(() => hasCurrentUserPermission('POWER_ACTIONS'))
	const canWriteFiles = computed(() => hasCurrentUserPermission('FILES_WRITE'))
	const canSetup = computed(() => hasCurrentUserPermission('SETUP'))
	const canManageBackups = computed(() => hasCurrentUserPermission('BACKUPS'))
	const canUseAdvancedSettings = computed(() => hasCurrentUserPermission('ADVANCED'))
	const canResetServer = computed(() => hasCurrentUserPermission('RESET_SERVER'))
	const canManageUsers = computed(() => hasCurrentUserPermission('MANAGE_USERS'))

	const permissionTooltip = (allowed: boolean) =>
		allowed ? undefined : permissionDeniedMessage.value

	return {
		currentUserPermissions,
		permissionDeniedMessage,
		hasCurrentUserPermission,
		canUsePowerActions,
		canWriteFiles,
		canSetup,
		canManageBackups,
		canUseAdvancedSettings,
		canResetServer,
		canManageUsers,
		permissionTooltip,
	}
}
