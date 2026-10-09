import type { Labrinth } from '@modrinth/api-client'

import { defineMessages, type MessageDescriptor } from '../composables/i18n'

export const userRestrictionMessages = defineMessages({
	createProjectsGroup: {
		id: 'user-restrictions.group.create-projects',
		defaultMessage: 'Create projects',
	},
	editProjectsGroup: {
		id: 'user-restrictions.group.edit-projects',
		defaultMessage: 'Edit projects',
	},
	versionsGroup: {
		id: 'user-restrictions.group.versions',
		defaultMessage: 'Versions',
	},
	organizationsGroup: {
		id: 'user-restrictions.group.organizations',
		defaultMessage: 'Organizations',
	},
	collectionsGroup: {
		id: 'user-restrictions.group.collections',
		defaultMessage: 'Collections',
	},
	profileGroup: {
		id: 'user-restrictions.group.profile',
		defaultMessage: 'Profile',
	},
	projectCreate: {
		id: 'user-restrictions.scope.project-create',
		defaultMessage: 'Create new projects',
	},
	projectWrite: {
		id: 'user-restrictions.scope.project-write',
		defaultMessage: 'Edit projects, manage team members, and accept invites',
	},
	projectDelete: {
		id: 'user-restrictions.scope.project-delete',
		defaultMessage: 'Delete projects',
	},
	versionCreate: {
		id: 'user-restrictions.scope.version-create',
		defaultMessage: 'Upload new versions',
	},
	versionWrite: {
		id: 'user-restrictions.scope.version-write',
		defaultMessage: 'Edit versions and add files to versions',
	},
	versionDelete: {
		id: 'user-restrictions.scope.version-delete',
		defaultMessage: 'Delete versions and version files',
	},
	organizationCreate: {
		id: 'user-restrictions.scope.organization-create',
		defaultMessage: 'Create organizations',
	},
	organizationWrite: {
		id: 'user-restrictions.scope.organization-write',
		defaultMessage: 'Edit organizations',
	},
	organizationDelete: {
		id: 'user-restrictions.scope.organization-delete',
		defaultMessage: 'Delete organizations',
	},
	collectionCreate: {
		id: 'user-restrictions.scope.collection-create',
		defaultMessage: 'Create collections',
	},
	collectionWrite: {
		id: 'user-restrictions.scope.collection-write',
		defaultMessage: 'Edit collections',
	},
	collectionDelete: {
		id: 'user-restrictions.scope.collection-delete',
		defaultMessage: 'Delete collections',
	},
	userWrite: {
		id: 'user-restrictions.scope.user-write',
		defaultMessage: 'Edit profile, follow projects, and change preferences',
	},
})

export type RemovableScope = {
	id: string
	bit: bigint
	label: MessageDescriptor
}

export type UserRestrictionGroup = {
	id: string
	label: MessageDescriptor
	scopes: RemovableScope[]
}

const scope = (id: string, shift: number, label: MessageDescriptor): RemovableScope => ({
	id,
	bit: 1n << BigInt(shift),
	label,
})

export const USER_RESTRICTION_GROUPS: UserRestrictionGroup[] = [
	{
		id: 'create-projects',
		label: userRestrictionMessages.createProjectsGroup,
		scopes: [scope('PROJECT_CREATE', 10, userRestrictionMessages.projectCreate)],
	},
	{
		id: 'edit-projects',
		label: userRestrictionMessages.editProjectsGroup,
		scopes: [
			scope('PROJECT_WRITE', 12, userRestrictionMessages.projectWrite),
			scope('PROJECT_DELETE', 13, userRestrictionMessages.projectDelete),
		],
	},
	{
		id: 'versions',
		label: userRestrictionMessages.versionsGroup,
		scopes: [
			scope('VERSION_CREATE', 14, userRestrictionMessages.versionCreate),
			scope('VERSION_WRITE', 16, userRestrictionMessages.versionWrite),
			scope('VERSION_DELETE', 17, userRestrictionMessages.versionDelete),
		],
	},
	{
		id: 'organizations',
		label: userRestrictionMessages.organizationsGroup,
		scopes: [
			scope('ORGANIZATION_CREATE', 35, userRestrictionMessages.organizationCreate),
			scope('ORGANIZATION_WRITE', 37, userRestrictionMessages.organizationWrite),
			scope('ORGANIZATION_DELETE', 38, userRestrictionMessages.organizationDelete),
		],
	},
	{
		id: 'collections',
		label: userRestrictionMessages.collectionsGroup,
		scopes: [
			scope('COLLECTION_CREATE', 31, userRestrictionMessages.collectionCreate),
			scope('COLLECTION_WRITE', 33, userRestrictionMessages.collectionWrite),
			scope('COLLECTION_DELETE', 34, userRestrictionMessages.collectionDelete),
		],
	},
	{
		id: 'profile',
		label: userRestrictionMessages.profileGroup,
		scopes: [scope('USER_WRITE', 2, userRestrictionMessages.userWrite)],
	},
]

export const REMOVABLE_SCOPES: RemovableScope[] = USER_RESTRICTION_GROUPS.flatMap(
	(group) => group.scopes,
)

export function hasRemovedScope(removedPerms: number | undefined, scopeId: string): boolean {
	if (!removedPerms) return false
	const removable = REMOVABLE_SCOPES.find((removableScope) => removableScope.id === scopeId)
	return !!removable && (BigInt(removedPerms) & removable.bit) !== 0n
}

export function isScopeRemovedForUser(
	user: Labrinth.Users.v2.User | Labrinth.Users.v3.User | null | undefined,
	scopeId: string,
): boolean {
	return (
		!!user && 'restriction' in user && hasRemovedScope(user.restriction?.removed_perms, scopeId)
	)
}

export function getRemovedScopes(removedPerms: number | undefined): RemovableScope[] {
	if (!removedPerms) return []
	const perms = BigInt(removedPerms)
	return REMOVABLE_SCOPES.filter((removableScope) => (perms & removableScope.bit) !== 0n)
}

export function encodeRemovedScopes(scopeIds: Iterable<string>): number {
	const ids = new Set(scopeIds)
	const perms = REMOVABLE_SCOPES.filter((removableScope) => ids.has(removableScope.id)).reduce(
		(acc, removableScope) => acc | removableScope.bit,
		0n,
	)
	return Number(perms)
}
