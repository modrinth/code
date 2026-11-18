import { defineMessage, defineMessages } from '@modrinth/ui'
import { TeamMemberPermission } from '@modrinth/utils'

export const permissionsHiddenMessage = defineMessage({
	id: 'project.settings.members.permissions-hidden',
	defaultMessage: 'Permissions hidden',
})

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

const accessMessages = defineMessages({
	viewer: { id: 'project.settings.members.access.viewer', defaultMessage: 'Viewer' },
	viewerDescription: {
		id: 'project.settings.members.access.viewer.description',
		defaultMessage: 'Can see analytics and total revenue.',
	},
	developer: { id: 'project.settings.members.access.developer', defaultMessage: 'Developer' },
	developerDescription: {
		id: 'project.settings.members.access.developer.description',
		defaultMessage: 'Can publish and delete versions and edit the description.',
	},
	serverDeveloperDescription: {
		id: 'project.settings.members.access.developer.server-description',
		defaultMessage: 'Can update content and edit the description.',
	},
	maintainer: { id: 'project.settings.members.access.maintainer', defaultMessage: 'Maintainer' },
	maintainerDescription: {
		id: 'project.settings.members.access.maintainer.description',
		defaultMessage: 'Can manage versions and edit project settings.',
	},
	serverMaintainerDescription: {
		id: 'project.settings.members.access.maintainer.server-description',
		defaultMessage: 'Can update content and edit project settings.',
	},
	admin: { id: 'project.settings.members.access.admin', defaultMessage: 'Admin' },
	adminDescription: {
		id: 'project.settings.members.access.admin.description',
		defaultMessage: 'Can manage versions, project settings, and members.',
	},
	serverAdminDescription: {
		id: 'project.settings.members.access.admin.server-description',
		defaultMessage: 'Can update content and manage project settings and members.',
	},
	analytics: { id: 'project.settings.members.access.group.analytics', defaultMessage: 'Analytics' },
	versions: { id: 'project.settings.members.access.group.versions', defaultMessage: 'Versions' },
	content: { id: 'project.settings.members.access.group.content', defaultMessage: 'Content' },
	settings: { id: 'project.settings.members.access.group.settings', defaultMessage: 'Settings' },
	members: { id: 'project.settings.members.access.group.members', defaultMessage: 'Members' },
	updateContent: {
		id: 'project.settings.members.permission.update-content',
		defaultMessage: 'Update content',
	},
	editDescription: {
		id: 'project.settings.members.permission.edit-description',
		defaultMessage: 'Edit description',
	},
	editSettings: {
		id: 'project.settings.members.permission.edit-settings',
		defaultMessage: 'Edit project settings',
	},
	viewTotalRevenue: {
		id: 'project.settings.members.permission.view-total-revenue',
		defaultMessage: 'View total revenue',
	},
})

export const viewerPermissions = projectPermissions.viewAnalytics | projectPermissions.viewPayouts
const developer =
	projectPermissions.uploadVersion | projectPermissions.deleteVersion | projectPermissions.editBody
const maintainer = viewerPermissions | developer | projectPermissions.editDetails
const members =
	projectPermissions.manageInvites | projectPermissions.editMember | projectPermissions.removeMember

export function projectAccessPresets(isServerProject: boolean) {
	const hiddenMask = isServerProject ? projectPermissions.deleteVersion : 0
	return [
		{
			id: 'viewer',
			label: accessMessages.viewer,
			description: accessMessages.viewerDescription,
			mask: viewerPermissions,
		},
		{
			id: 'developer',
			label: accessMessages.developer,
			description: isServerProject
				? accessMessages.serverDeveloperDescription
				: accessMessages.developerDescription,
			mask: developer & ~hiddenMask,
		},
		{
			id: 'maintainer',
			label: accessMessages.maintainer,
			description: isServerProject
				? accessMessages.serverMaintainerDescription
				: accessMessages.maintainerDescription,
			mask: maintainer & ~hiddenMask,
		},
		{
			id: 'admin',
			label: accessMessages.admin,
			description: isServerProject
				? accessMessages.serverAdminDescription
				: accessMessages.adminDescription,
			mask: (maintainer | members) & ~hiddenMask,
		},
	]
}

type PermissionKey = keyof typeof projectPermissions

export function projectAccessGroups(isServerProject: boolean) {
	const optionMessages = {
		...permissionMessages,
		uploadVersion: isServerProject
			? accessMessages.updateContent
			: permissionMessages.uploadVersion,
		editBody: accessMessages.editDescription,
		editDetails: accessMessages.editSettings,
		viewPayouts: accessMessages.viewTotalRevenue,
	}
	const groups: { id: string; label: typeof accessMessages.analytics; keys: PermissionKey[] }[] = [
		{ id: 'analytics', label: accessMessages.analytics, keys: ['viewAnalytics', 'viewPayouts'] },
		{
			id: 'versions',
			label: isServerProject ? accessMessages.content : accessMessages.versions,
			keys: isServerProject ? ['uploadVersion'] : ['uploadVersion', 'deleteVersion'],
		},
		{
			id: 'settings',
			label: accessMessages.settings,
			keys: ['editBody', 'editDetails', 'deleteProject'],
		},
		{
			id: 'members',
			label: accessMessages.members,
			keys: ['manageInvites', 'editMember', 'removeMember'],
		},
	]
	return groups.map(({ id, label, keys }) => ({
		id,
		label,
		options: keys.map((key) => ({
			bit: projectPermissions[key],
			label: optionMessages[key],
			destructive: key === 'deleteProject',
		})),
	}))
}
