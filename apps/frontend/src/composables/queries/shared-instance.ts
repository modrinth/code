import {
	type AbstractModrinthClient,
	type Labrinth,
	ModrinthApiError,
	type SharedInstances,
} from '@modrinth/api-client'
import type { ContentItem } from '@modrinth/ui'
import type { QueryClient } from '@tanstack/vue-query'

import { createSharedInstanceContentLoader } from '~/helpers/shared-instance-content'

type InstanceMembership = SharedInstances.Instances.v1.InstanceUser
type InstanceVersion = SharedInstances.Instances.v1.InstanceVersion

export interface SharedInstanceModerationUser {
	id: string
	username: string
	avatar_url?: string | null
	joined_at: string | null
	last_played: string | null
	join_type: SharedInstances.Instances.v1.JoinType
}

export interface SharedInstanceVersionSummary {
	version: number
	game_version?: string
	loader?: string
	loader_version?: string
}

export interface SharedInstanceModerationDetails {
	id: string
	name: string
	icon_url: string | null
	quarantine: boolean
	quarantine_details: SharedInstances.Instances.v1.QuarantineDetails | null
	owner: SharedInstanceModerationUser
	/** Members who joined, excluding the owner. */
	members: SharedInstanceModerationUser[]
	/** Newest version first. Only the newest includes game version and loader details. */
	versions: SharedInstanceVersionSummary[]
	/** How many shared instances the owner owns or belongs to, or null when it could not be loaded. */
	owner_instance_count: number | null
}

export interface UserSharedInstance {
	id: string
	name: string
	iconUrl: string | null
	quarantine: boolean
	quarantineScope: SharedInstances.Instances.v1.QuarantineScope | null
	memberCount: number
	membership?: InstanceMembership
	latestVersion: InstanceVersion | null
}

export const sharedInstanceQueryKeys = {
	allModerationContexts: ['shared-instance', 'moderation-context'] as const,
	moderationContext: (instanceId: string) =>
		[...sharedInstanceQueryKeys.allModerationContexts, 'v1', instanceId] as const,
	version: (instanceId: string, version: number) =>
		['shared-instance', 'version', 'v1', instanceId, version] as const,
	versionContent: (instanceId: string, version: number) =>
		['shared-instance', 'version-content', 'v1', instanceId, version] as const,
	blacklist: (userId: string) => ['shared-instance-blacklist', 'v1', userId] as const,
	allUserInstances: ['shared-instances', 'user'] as const,
	userInstances: (userId: string) => [...sharedInstanceQueryKeys.allUserInstances, userId] as const,
	openReports: (instanceIds: string[]) =>
		['shared-instance', 'open-reports', 'v3', instanceIds] as const,
	moderator: (userId: string) => ['user', 'v2', userId] as const,
}

const STALE_TIME = 30_000
const OPEN_REPORTS_PAGE_SIZE = 100

export const sharedInstanceQueryOptions = {
	version: (instanceId: string, version: number, client: AbstractModrinthClient) => ({
		queryKey: sharedInstanceQueryKeys.version(instanceId, version),
		queryFn: () => client.sharedinstances.instances_v1.getVersion(instanceId, version),
		staleTime: STALE_TIME,
	}),

	versionContent: (
		instanceId: string,
		version: number,
		client: AbstractModrinthClient,
		queryClient: QueryClient,
	) => ({
		queryKey: sharedInstanceQueryKeys.versionContent(instanceId, version),
		queryFn: (): Promise<ContentItem[]> =>
			createSharedInstanceContentLoader(client, (id, number) =>
				queryClient.fetchQuery(sharedInstanceQueryOptions.version(id, number, client)),
			).loadVersionContent(instanceId, version),
		staleTime: STALE_TIME,
	}),

	moderationContext: (
		instanceId: string,
		client: AbstractModrinthClient,
		queryClient: QueryClient,
	) => ({
		queryKey: sharedInstanceQueryKeys.moderationContext(instanceId),
		queryFn: () => loadModerationContext(instanceId, client, queryClient),
		staleTime: STALE_TIME,
	}),

	blacklist: (userId: string, client: AbstractModrinthClient) => ({
		queryKey: sharedInstanceQueryKeys.blacklist(userId),
		queryFn: () => client.sharedinstances.users_v1.getBlacklistStatus(userId),
		staleTime: STALE_TIME,
		retry: false,
	}),

	userInstances: (userId: string, client: AbstractModrinthClient) => ({
		queryKey: sharedInstanceQueryKeys.userInstances(userId),
		queryFn: () => loadUserInstances(userId, client),
		staleTime: STALE_TIME,
	}),

	openReports: (instanceIds: string[], client: AbstractModrinthClient) => ({
		queryKey: sharedInstanceQueryKeys.openReports(instanceIds),
		queryFn: async () =>
			(
				await Promise.all(
					instanceIds.map((instanceId) => loadOpenSharedInstanceReports(instanceId, client)),
				)
			).flat(),
		staleTime: STALE_TIME,
	}),

	moderator: (userId: string, client: AbstractModrinthClient) => ({
		queryKey: sharedInstanceQueryKeys.moderator(userId),
		queryFn: () => client.labrinth.users_v2.get(userId),
		staleTime: STALE_TIME,
		retry: false,
	}),
}

async function loadModerationContext(
	instanceId: string,
	client: AbstractModrinthClient,
	queryClient: QueryClient,
): Promise<SharedInstanceModerationDetails> {
	const [instance, instanceUsers, latestVersion] = await Promise.all([
		client.sharedinstances.instances_v1.get(instanceId),
		client.sharedinstances.instances_v1.getUsers(instanceId),
		client.sharedinstances.instances_v1.getLatestVersion(instanceId).catch((error: unknown) => {
			if (error instanceof ModrinthApiError && error.statusCode === 404) return null
			throw error
		}),
	])
	if (latestVersion) {
		queryClient.setQueryData(
			sharedInstanceQueryKeys.version(instanceId, latestVersion.version),
			latestVersion,
		)
	}

	const ownerMembership = instanceUsers.users.find((membership) => membership.join_type === 'owner')
	if (!ownerMembership) throw new Error('The shared instance has no owner.')

	const userIds = [...new Set(instanceUsers.users.map((membership) => membership.id))]
	const versions: SharedInstanceVersionSummary[] = latestVersion
		? [
				{
					version: latestVersion.version,
					game_version: latestVersion.game_version,
					loader: latestVersion.loader,
					loader_version: latestVersion.loader_version,
				},
				...Array.from({ length: latestVersion.version }, (_, index) => ({
					version: latestVersion.version - index - 1,
				})),
			]
		: []

	const [users, ownerInstanceIds] = await Promise.all([
		userIds.length ? client.labrinth.users_v2.getMultiple(userIds) : Promise.resolve([]),
		client.sharedinstances.instances_v1.getForUser(ownerMembership.id).catch(() => null),
	])
	const usersById = new Map(users.map((user) => [user.id, user]))
	const toModerationUser = (membership: InstanceMembership): SharedInstanceModerationUser => {
		const user = usersById.get(membership.id)
		return {
			id: membership.id,
			username: user?.username ?? membership.id,
			avatar_url: user?.avatar_url,
			joined_at: membership.joined_at,
			last_played: membership.last_played,
			join_type: membership.join_type,
		}
	}

	return {
		id: instanceId,
		name: instance.name,
		icon_url: instance.icon,
		quarantine: instance.quarantine,
		quarantine_details: instance.quarantine_details,
		owner: toModerationUser(ownerMembership),
		members: instanceUsers.users
			.filter((membership) => membership.id !== ownerMembership.id && membership.joined_at != null)
			.map(toModerationUser),
		versions,
		owner_instance_count: ownerInstanceIds ? new Set(ownerInstanceIds).size : null,
	}
}

async function loadUserInstances(
	userId: string,
	client: AbstractModrinthClient,
): Promise<{ instances: UserSharedInstance[]; unavailableCount: number }> {
	const instanceIds = [...new Set(await client.sharedinstances.instances_v1.getForUser(userId))]
	const results = await Promise.allSettled(
		instanceIds.map(async (instanceId): Promise<UserSharedInstance> => {
			const [instance, instanceUsers, latestVersion] = await Promise.all([
				client.sharedinstances.instances_v1.get(instanceId),
				client.sharedinstances.instances_v1.getUsers(instanceId),
				client.sharedinstances.instances_v1.getLatestVersion(instanceId).catch(() => null),
			])

			return {
				id: instanceId,
				name: instance.name,
				iconUrl: instance.icon,
				quarantine: instance.quarantine,
				quarantineScope: instance.quarantine_details?.scope ?? null,
				memberCount: instanceUsers.users.filter((membership) => membership.joined_at != null)
					.length,
				membership: instanceUsers.users.find((membership) => membership.id === userId),
				latestVersion,
			}
		}),
	)
	const instances = results
		.flatMap((result) => (result.status === 'fulfilled' ? [result.value] : []))
		.sort((first, second) => first.name.localeCompare(second.name))

	if (instanceIds.length > 0 && instances.length === 0) {
		throw new Error("None of this user's shared instances could be loaded.")
	}

	return { instances, unavailableCount: results.length - instances.length }
}

async function loadOpenSharedInstanceReports(
	instanceId: string,
	client: AbstractModrinthClient,
): Promise<Labrinth.Reports.v3.Report[]> {
	const reports: Labrinth.Reports.v3.Report[] = []
	for (let offset = 0; ; offset += OPEN_REPORTS_PAGE_SIZE) {
		const page = await client.labrinth.reports_v3.list({
			all: true,
			item_id: instanceId,
			count: OPEN_REPORTS_PAGE_SIZE,
			offset,
		})
		reports.push(...page.filter((report) => report.item_type === 'shared-instance'))
		if (page.length < OPEN_REPORTS_PAGE_SIZE) break
	}
	return reports
}
