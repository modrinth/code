import { defineMessages } from '@modrinth/ui'
import { TeamMemberPermission } from '@modrinth/utils'

export const projectPermissions = {
	uploadVersion: TeamMemberPermission.UPLOAD_VERSION,
	deleteVersion: TeamMemberPermission.DELETE_VERSION,
	editDetails: TeamMemberPermission.EDIT_DETAILS,
	editBody: TeamMemberPermission.EDIT_BODY,
	manageInvites: TeamMemberPermission.MANAGE_INVITES,
	removeMember: TeamMemberPermission.REMOVE_MEMBER,
	editMember: TeamMemberPermission.EDIT_MEMBER,
	deleteProject: TeamMemberPermission.DELETE_PROJECT,
	viewAnalytics: TeamMemberPermission.VIEW_ANALYTICS,
	viewPayouts: TeamMemberPermission.VIEW_PAYOUTS,
}

export const permissionMessages = defineMessages({
	uploadVersion: {
		id: 'project.settings.members.permission.upload-version',
		defaultMessage: 'Upload version',
	},
	deleteVersion: {
		id: 'project.settings.members.permission.delete-version',
		defaultMessage: 'Delete version',
	},
	editDetails: {
		id: 'project.settings.members.permission.edit-details',
		defaultMessage: 'Edit details',
	},
	editBody: { id: 'project.settings.members.permission.edit-body', defaultMessage: 'Edit body' },
	manageInvites: {
		id: 'project.settings.members.permission.manage-invites',
		defaultMessage: 'Manage invites',
	},
	removeMember: {
		id: 'project.settings.members.permission.remove-member',
		defaultMessage: 'Remove member',
	},
	editMember: {
		id: 'project.settings.members.permission.edit-member',
		defaultMessage: 'Edit member',
	},
	deleteProject: {
		id: 'project.settings.members.permission.delete-project',
		defaultMessage: 'Delete project',
	},
	viewAnalytics: {
		id: 'project.settings.members.permission.view-analytics',
		defaultMessage: 'View analytics',
	},
	viewPayouts: {
		id: 'project.settings.members.permission.view-revenue',
		defaultMessage: 'View revenue',
	},
})

export function hasPermission(permissions: number | null | undefined, permission: number) {
	return ((permissions ?? 0) & permission) === permission
}
