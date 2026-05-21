import type { Archon } from '@modrinth/api-client'
import { computed } from 'vue'

import { useVIntl } from '#ui/composables/i18n'
import { injectModrinthServerContext } from '#ui/providers'
import { commonMessages } from '#ui/utils/common-messages'

export type ServerPermissionName = keyof typeof Archon.ServerUsers.v1.UserScope

type ServerPermissionValue = Archon.Servers.v0.UserScope | Archon.ServerUsers.v1.UserScope

const U64_SIZE = 64n
const U64_MODULUS = 1n << U64_SIZE

export const serverPermissionBits = {
	NONE: 0n,
	BASE_READ: 1n << 63n,
	POWER_ACTIONS: 1n << 62n,
	FILES_WRITE: 1n << 61n,
	SETUP: 1n << 60n,
	BACKUPS: 1n << 59n,
	ADVANCED: 1n << 58n,
	RESET_SERVER: 1n << 57n,
	MANAGE_USERS: 1n << 56n,
	SUPPORT_AGENT: 1n,
	INFRA_MANAGER: 1n << 1n,
	INFRA_MANAGER_READ: 1n << 2n,
	INFRA_SERVERS_XFER: 1n << 3n,
	SERVER_ADMIN: ((1n << 64n) - 1n) ^ ((1n << 15n) - 1n),
} as const satisfies Record<ServerPermissionName, bigint>

function parsePermissionNumber(value: number) {
	const bigintValue = BigInt(value)
	return bigintValue < 0n ? bigintValue + U64_MODULUS : bigintValue
}

function parsePermissionString(value: string) {
	const numericValue = Number(value)
	if (value.trim() !== '' && Number.isFinite(numericValue)) {
		return parsePermissionNumber(numericValue)
	}

	const permissions = value
		.split('|')
		.map((permission) => permission.trim())
		.filter((permission): permission is ServerPermissionName => permission in serverPermissionBits)

	if (permissions.length === 0) return 0n

	return permissions.reduce((mask, permission) => mask | serverPermissionBits[permission], 0n)
}

function parsePermissions(permissions: ServerPermissionValue) {
	return typeof permissions === 'number'
		? parsePermissionNumber(permissions)
		: parsePermissionString(permissions)
}

function hasPermissionBit(permissions: ServerPermissionValue, scope: ServerPermissionName) {
	const permission = serverPermissionBits[scope]
	if (permission === 0n) return true

	const permissionsMask = parsePermissions(permissions)
	return (permissionsMask & permission) === permission
}

export function hasServerPermission(
	permissions: ServerPermissionValue,
	scope: ServerPermissionName,
) {
	if (
		scope !== 'NONE' &&
		scope !== 'SERVER_ADMIN' &&
		hasPermissionBit(permissions, 'SERVER_ADMIN')
	) {
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
