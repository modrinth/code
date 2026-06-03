import type { Archon } from '@modrinth/api-client'

import { hasServerPermission } from '../../../composables/server-permissions'
import type { ServerAccessRole } from './types'

export function apiPermissionsToAccessRole(
	permissions: Archon.ServerUsers.v1.UserScope,
): ServerAccessRole {
	if (hasServerPermission(permissions, 'SERVER_ADMIN')) {
		return 'owner'
	}
	if (
		hasServerPermission(permissions, 'EXEC_COMMANDS') ||
		hasServerPermission(permissions, 'FILES_WRITE') ||
		hasServerPermission(permissions, 'SETUP') ||
		hasServerPermission(permissions, 'BACKUPS') ||
		hasServerPermission(permissions, 'ADVANCED') ||
		hasServerPermission(permissions, 'RESET_SERVER')
	) {
		return 'editor'
	}
	return 'viewer'
}
