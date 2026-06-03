import type { MessageDescriptor } from '#ui/composables/i18n'
import { defineMessages } from '#ui/composables/i18n'

import type { ActionLogFilterActionName } from './audit-log-utils'

export const accessMessages = defineMessages({
	searchUsersPlaceholder: {
		id: 'servers.access-page.search-users-placeholder',
		defaultMessage: 'Search {count} {count, plural, one {user} other {users}}...',
	},
	inviteFriends: {
		id: 'servers.access-page.invite-friends',
		defaultMessage: 'Add user',
	},
	activityLogTitle: {
		id: 'servers.access-page.activity-log-title',
		defaultMessage: 'Activity log',
	},
	addFilter: {
		id: 'servers.access-page.activity-log-filter.add',
		defaultMessage: 'Add filter',
	},
	clearFilters: {
		id: 'servers.access-page.activity-log-filter.clear',
		defaultMessage: 'Clear filters',
	},
	emptyFilterOptions: {
		id: 'servers.access-page.activity-log-filter.empty-options',
		defaultMessage: 'No options available.',
	},
	emptyFilterSearch: {
		id: 'servers.access-page.activity-log-filter.empty-search',
		defaultMessage: 'No options found.',
	},
	userFilter: {
		id: 'servers.access-page.activity-log-filter.users',
		defaultMessage: 'User',
	},
	supportActor: {
		id: 'servers.access-page.activity-log-filter.support-actor',
		defaultMessage: 'Support',
	},
	instanceFilter: {
		id: 'servers.access-page.activity-log-filter.instances',
		defaultMessage: 'Instances',
	},
	serverScopedInstance: {
		id: 'servers.access-page.activity-log-filter.server-scoped-instance',
		defaultMessage: 'Server',
	},
	actionTypeFilter: {
		id: 'servers.access-page.activity-log-filter.action-types',
		defaultMessage: 'Actions',
	},
	actionTypeFilterSearch: {
		id: 'servers.access-page.activity-log-filter.action-types-search',
		defaultMessage: 'Search actions...',
	},
	allRoles: {
		id: 'servers.access-page.role-filter.all',
		defaultMessage: 'All',
	},
	selectedRoleFilter: {
		id: 'servers.access-page.role-filter.selected',
		defaultMessage: 'Role: {role}',
	},
	ownerRole: {
		id: 'servers.access-page.role.owner',
		defaultMessage: 'Owner',
	},
	ownerDescription: {
		id: 'servers.access-page.role.owner-description',
		defaultMessage: 'Full access including billing, members, and destructive actions.',
	},
	editorRole: {
		id: 'servers.access-page.role.editor',
		defaultMessage: 'Editor',
	},
	editorDescription: {
		id: 'servers.access-page.role.editor-description',
		defaultMessage: 'Manage instance content, files, backups, and other settings.',
	},
	viewerRole: {
		id: 'servers.access-page.role.viewer',
		defaultMessage: 'Limited',
	},
	viewerDescription: {
		id: 'servers.access-page.role.viewer-description',
		defaultMessage: 'Start, stop, and view the server without making changes.',
	},
	inviteSentTitle: {
		id: 'servers.access-page.notification.invite-sent.title',
		defaultMessage: 'Invite sent',
	},
	inviteSentText: {
		id: 'servers.access-page.notification.invite-sent.text',
		defaultMessage: 'Invited {target} as {role}.',
	},
	inviteResentTitle: {
		id: 'servers.access-page.notification.invite-resent.title',
		defaultMessage: 'Invite resent',
	},
	inviteResentText: {
		id: 'servers.access-page.notification.invite-resent.text',
		defaultMessage: 'Sent another invite to {target}.',
	},
	inviteCancelledTitle: {
		id: 'servers.access-page.notification.invite-cancelled.title',
		defaultMessage: 'Invite cancelled',
	},
	inviteCancelledText: {
		id: 'servers.access-page.notification.invite-cancelled.text',
		defaultMessage: 'Cancelled the invite for {target}.',
	},
	memberRemovedTitle: {
		id: 'servers.access-page.notification.member-removed.title',
		defaultMessage: 'Access removed',
	},
	memberRemovedText: {
		id: 'servers.access-page.notification.member-removed.text',
		defaultMessage: 'Removed {target} from this server.',
	},
	loadFailedTitle: {
		id: 'servers.access-page.notification.load-failed.title',
		defaultMessage: 'Access could not be loaded',
	},
	loadFailedText: {
		id: 'servers.access-page.notification.load-failed.text',
		defaultMessage: 'Refresh the page to try again.',
	},
	inviteFailedTitle: {
		id: 'servers.access-page.notification.invite-failed.title',
		defaultMessage: 'Invite could not be sent',
	},
	friendRequestFailedTitle: {
		id: 'servers.access-page.notification.friend-request-failed.title',
		defaultMessage: 'Friend request could not be sent',
	},
	removeFailedTitle: {
		id: 'servers.access-page.notification.remove-failed.title',
		defaultMessage: 'Access could not be removed',
	},
	roleUpdateFailedTitle: {
		id: 'servers.access-page.notification.role-update-failed.title',
		defaultMessage: 'Role could not be updated',
	},
})

export const actionLogActionMessages = defineMessages({
	server_created: {
		id: 'servers.access-page.activity-log-filter.action.server-created',
		defaultMessage: 'Created server',
	},
	changed_server_name: {
		id: 'servers.access-page.activity-log-filter.action.changed-server-name',
		defaultMessage: 'Changed server name',
	},
	changed_server_subdomain: {
		id: 'servers.access-page.activity-log-filter.action.changed-server-subdomain',
		defaultMessage: 'Changed server subdomain',
	},
	server_reallocated: {
		id: 'servers.access-page.activity-log-filter.action.server-reallocated',
		defaultMessage: 'Reallocated server',
	},
	server_plan_changed: {
		id: 'servers.access-page.activity-log-filter.action.server-plan-changed',
		defaultMessage: 'Changed plan',
	},
	user_invited: {
		id: 'servers.access-page.activity-log-filter.action.user-invited',
		defaultMessage: 'Invited user',
	},
	user_invite_revoked: {
		id: 'servers.access-page.activity-log-filter.action.user-invite-revoked',
		defaultMessage: 'Revoked user invite',
	},
	user_permission_modified: {
		id: 'servers.access-page.activity-log-filter.action.user-permission-modified',
		defaultMessage: 'Changed user permissions',
	},
	user_removed: {
		id: 'servers.access-page.activity-log-filter.action.user-removed',
		defaultMessage: 'Removed user',
	},
	addon_added: {
		id: 'servers.access-page.activity-log-filter.action.addon-added',
		defaultMessage: 'Added content',
	},
	addon_uploaded: {
		id: 'servers.access-page.activity-log-filter.action.addon-uploaded',
		defaultMessage: 'Uploaded content',
	},
	addon_disabled: {
		id: 'servers.access-page.activity-log-filter.action.addon-disabled',
		defaultMessage: 'Disabled content',
	},
	addon_enabled: {
		id: 'servers.access-page.activity-log-filter.action.addon-enabled',
		defaultMessage: 'Enabled content',
	},
	addon_deleted: {
		id: 'servers.access-page.activity-log-filter.action.addon-deleted',
		defaultMessage: 'Deleted content',
	},
	addon_updated: {
		id: 'servers.access-page.activity-log-filter.action.addon-updated',
		defaultMessage: 'Updated content',
	},
	modpack_changed: {
		id: 'servers.access-page.activity-log-filter.action.modpack-changed',
		defaultMessage: 'Changed modpack',
	},
	modpack_unlinked: {
		id: 'servers.access-page.activity-log-filter.action.modpack-unlinked',
		defaultMessage: 'Unlinked modpack',
	},
	server_repaired: {
		id: 'servers.access-page.activity-log-filter.action.server-repaired',
		defaultMessage: 'Repaired server',
	},
	server_reset: {
		id: 'servers.access-page.activity-log-filter.action.server-reset',
		defaultMessage: 'Reset server',
	},
	server_started: {
		id: 'servers.access-page.activity-log-filter.action.server-started',
		defaultMessage: 'Started server',
	},
	server_stopped: {
		id: 'servers.access-page.activity-log-filter.action.server-stopped',
		defaultMessage: 'Stopped server',
	},
	server_restarted: {
		id: 'servers.access-page.activity-log-filter.action.server-restarted',
		defaultMessage: 'Restarted server',
	},
	server_killed: {
		id: 'servers.access-page.activity-log-filter.action.server-killed',
		defaultMessage: 'Killed server',
	},
	port_allocation_added: {
		id: 'servers.access-page.activity-log-filter.action.port-allocation-added',
		defaultMessage: 'Added port allocation',
	},
	port_allocation_removed: {
		id: 'servers.access-page.activity-log-filter.action.port-allocation-removed',
		defaultMessage: 'Removed port allocation',
	},
	loader_version_edited: {
		id: 'servers.access-page.activity-log-filter.action.loader-version-edited',
		defaultMessage: 'Changed loader version',
	},
	game_version_edited: {
		id: 'servers.access-page.activity-log-filter.action.game-version-edited',
		defaultMessage: 'Changed Minecraft version',
	},
	server_properties_modified: {
		id: 'servers.access-page.activity-log-filter.action.server-properties-modified',
		defaultMessage: 'Modified server properties',
	},
	file_uploaded: {
		id: 'servers.access-page.activity-log-filter.action.file-uploaded',
		defaultMessage: 'Uploaded file',
	},
	file_deleted: {
		id: 'servers.access-page.activity-log-filter.action.file-deleted',
		defaultMessage: 'Deleted file',
	},
	file_renamed: {
		id: 'servers.access-page.activity-log-filter.action.file-renamed',
		defaultMessage: 'Renamed file',
	},
	file_edited: {
		id: 'servers.access-page.activity-log-filter.action.file-edited',
		defaultMessage: 'Edited file',
	},
	sftp_login: {
		id: 'servers.access-page.activity-log-filter.action.sftp-login',
		defaultMessage: 'Logged in via SFTP',
	},
	console_command_executed: {
		id: 'servers.access-page.activity-log-filter.action.console-command-executed',
		defaultMessage: 'Ran console command',
	},
	console_cleared: {
		id: 'servers.access-page.activity-log-filter.action.console-cleared',
		defaultMessage: 'Cleared console',
	},
	backup_created: {
		id: 'servers.access-page.activity-log-filter.action.backup-created',
		defaultMessage: 'Created backup',
	},
	backup_renamed: {
		id: 'servers.access-page.activity-log-filter.action.backup-renamed',
		defaultMessage: 'Renamed backup',
	},
	backup_restored: {
		id: 'servers.access-page.activity-log-filter.action.backup-restored',
		defaultMessage: 'Restored backup',
	},
	backup_deleted: {
		id: 'servers.access-page.activity-log-filter.action.backup-deleted',
		defaultMessage: 'Deleted backup',
	},
	startup_command_modified: {
		id: 'servers.access-page.activity-log-filter.action.startup-command-modified',
		defaultMessage: 'Changed startup command',
	},
	java_runtime_modified: {
		id: 'servers.access-page.activity-log-filter.action.java-runtime-modified',
		defaultMessage: 'Changed Java runtime',
	},
	java_version_modified: {
		id: 'servers.access-page.activity-log-filter.action.java-version-modified',
		defaultMessage: 'Changed Java version',
	},
} satisfies Record<ActionLogFilterActionName, MessageDescriptor>)
